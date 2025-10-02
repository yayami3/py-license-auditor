use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use anyhow::Result;
use std::fs;
use std::path::PathBuf;
use rayon::prelude::*;
use indexmap::IndexMap;
use crate::uv_lock::UvLockParser;

pub mod extractor;

// Re-export from extractor
pub use extractor::extract_all_licenses;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageLicense {
    pub name: String,
    pub version: Option<String>,
    pub license: Option<String>,
    pub license_classifiers: Vec<String>,
    pub metadata_source: String,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct LicenseTypes {
    pub osi_approved: IndexMap<String, usize>,
    pub non_osi: IndexMap<String, usize>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct LicenseSummary {
    pub total_packages: usize,
    pub with_license: usize,
    pub without_license: usize,
    pub license_types: LicenseTypes,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct LicenseReport {
    pub packages: Vec<PackageLicense>,
    pub summary: LicenseSummary,
    /// 違反情報（ポリシーチェックが有効な場合のみ）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub violations: Option<crate::policy::ViolationSummary>,
}

pub fn find_site_packages_path(path: Option<PathBuf>) -> Result<PathBuf> {
    if let Some(path) = path {
        if path.join("site-packages").exists() {
            return Ok(path.join("site-packages"));
        }
        if path.file_name().map_or(false, |name| name == "site-packages") {
            return Ok(path);
        }
        return Ok(path);
    }

    // Try to find .venv in current directory
    let current_dir = std::env::current_dir()?;
    let venv_path = current_dir.join(".venv");
    
    if venv_path.exists() {
        // Unix-like systems
        let lib_path = venv_path.join("lib");
        if lib_path.exists() {
            for entry in fs::read_dir(&lib_path)? {
                let entry = entry?;
                if entry.file_name().to_string_lossy().starts_with("python") {
                    let site_packages = entry.path().join("site-packages");
                    if site_packages.exists() {
                        return Ok(site_packages);
                    }
                }
            }
        }
        
        // Windows
        let lib_path = venv_path.join("Lib").join("site-packages");
        if lib_path.exists() {
            return Ok(lib_path);
        }
    }

    anyhow::bail!("Could not find site-packages directory. Please specify with --path")
}

/// Extract licenses from uv.lock file and corresponding site-packages
pub fn extract_licenses_from_uv_lock(uv_lock_path: Option<PathBuf>, site_packages_path: Option<PathBuf>, include_unknown: bool) -> Result<Vec<PackageLicense>> {
    // Find uv.lock file
    let lock_path = match uv_lock_path {
        Some(path) => path,
        None => UvLockParser::find_uv_lock()
            .ok_or_else(|| anyhow::anyhow!("No uv.lock file found in current directory or parent directories"))?
    };

    // Parse uv.lock
    let lock_file = UvLockParser::parse_uv_lock(&lock_path)?;
    let uv_packages = UvLockParser::extract_packages(&lock_file);

    // Find site-packages directory
    let site_packages = match site_packages_path {
        Some(path) => path,
        None => find_site_packages_path(None)?
    };

    // Extract licenses for packages found in uv.lock (parallel processing)
    let licenses: Vec<PackageLicense> = uv_packages
        .par_iter()
        .filter_map(|(package_name, package_version)| {
            if let Ok(mut license_info) = extractor::extract_license_for_package(&site_packages, package_name) {
                // Verify version matches uv.lock
                if license_info.version.as_ref() != Some(package_version) {
                    license_info.version = Some(package_version.clone());
                }
                Some(license_info)
            } else if include_unknown {
                // Package in uv.lock but not found in site-packages
                Some(PackageLicense {
                    name: package_name.clone(),
                    version: Some(package_version.clone()),
                    license: None,
                    license_classifiers: vec![],
                    metadata_source: "uv.lock (not installed)".to_string(),
                })
            } else {
                None
            }
        })
        .collect();

    Ok(licenses)
}

/// Auto-detect and extract licenses (uv.lock required)
pub fn extract_licenses_auto(path: Option<PathBuf>, include_unknown: bool) -> Result<Vec<PackageLicense>> {
    // Require uv.lock file - no fallback to site-packages
    if let Some(_) = UvLockParser::find_uv_lock() {
        eprintln!("Found uv.lock, using uv-native extraction");
        return extract_licenses_from_uv_lock(None, path, include_unknown);
    }

    // No uv.lock found - this tool is uv-only
    Err(anyhow::anyhow!(
        "No uv.lock found. This tool requires uv projects.\n\
         Please run 'uv sync' to generate uv.lock file.\n\
         \n\
         For non-uv projects, consider migrating to uv:\n\
         - uv init (new project)\n\
         - uv import (from requirements.txt/pyproject.toml)"
    ))
}

pub fn create_report(packages: Vec<PackageLicense>) -> LicenseReport {
    let total_packages = packages.len();
    let with_license = packages.iter()
        .filter(|p| get_effective_license(p).is_some())
        .count();
    let without_license = total_packages - with_license;

    let mut osi_counts = HashMap::new();
    let mut non_osi_counts = HashMap::new();

    for package in &packages {
        let license_info = get_license_info(package);
        for (license_name, is_osi) in license_info {
            if is_osi {
                *osi_counts.entry(license_name).or_insert(0) += 1;
            } else {
                *non_osi_counts.entry(license_name).or_insert(0) += 1;
            }
        }
    }

    // Convert HashMap to Vec, sort by count (descending), then create IndexMap
    let mut osi_vec: Vec<(String, usize)> = osi_counts.into_iter().collect();
    osi_vec.sort_by(|a, b| b.1.cmp(&a.1));
    let osi_approved: IndexMap<String, usize> = osi_vec.into_iter().collect();

    let mut non_osi_vec: Vec<(String, usize)> = non_osi_counts.into_iter().collect();
    non_osi_vec.sort_by(|a, b| b.1.cmp(&a.1));
    let non_osi: IndexMap<String, usize> = non_osi_vec.into_iter().collect();

    LicenseReport {
        packages,
        summary: LicenseSummary {
            total_packages,
            with_license,
            without_license,
            license_types: LicenseTypes {
                osi_approved,
                non_osi,
            },
        },
        violations: None,
    }
}

fn get_license_info(package: &PackageLicense) -> Vec<(String, bool)> {
    let mut licenses = Vec::new();

    // Prioritize classifiers (more standardized)
    for classifier in &package.license_classifiers {
        if let Some(license_name) = extract_license_from_classifier(classifier) {
            let normalized_name = normalize_license_name(&license_name);
            let is_osi = classifier.contains("OSI Approved");
            licenses.push((normalized_name, is_osi));
        }
    }

    // Fallback to License field if no classifiers
    if licenses.is_empty() {
        if let Some(license) = &package.license {
            let normalized_name = normalize_license_name(license);
            let is_osi = is_osi_approved_license(&normalized_name);
            licenses.push((normalized_name, is_osi));
        }
    }

    // If no license information found, add "Unknown"
    if licenses.is_empty() {
        licenses.push(("Unknown".to_string(), false));
    }

    licenses
}

pub fn normalize_license_name(license: &str) -> String {
    let license = license.trim();
    let license_lower = license.to_lowercase();
    
    // Handle copyright statements and invalid licenses first
    if license.starts_with("Copyright") || license.starts_with("=") || license.len() < 3 {
        return "Unknown".to_string();
    }
    
    // Exact matches first (most common cases)
    match license {
        "MIT" | "MIT License" | "MIT license" | "Expat license" => return "MIT".to_string(),
        "Apache-2.0" | "Apache License" | "Apache Software License" => return "Apache-2.0".to_string(),
        "BSD-3-Clause" | "BSD 3-Clause" | "BSD 3-Clause License" => return "BSD-3-Clause".to_string(),
        "BSD-2-Clause" | "BSD 2-Clause" | "BSD 2-Clause License" => return "BSD-2-Clause".to_string(),
        "MPL-2.0" | "Mozilla Public License 2.0" => return "MPL-2.0".to_string(),
        "ISC" | "ISC License" => return "ISC".to_string(),
        "GPL-2.0" | "GPLv2" => return "GPL-2.0".to_string(),
        "GPL-3.0" | "GPLv3" => return "GPL-3.0".to_string(),
        "LGPL-2.1" | "LGPLv2.1" => return "LGPL-2.1".to_string(),
        "LGPL-3.0" | "LGPLv3" => return "LGPL-3.0".to_string(),
        _ => {}
    }
    
    // Pattern matching for variations
    if license_lower.contains("mit") {
        return "MIT".to_string();
    }
    if license_lower.contains("apache") && (license_lower.contains("2.0") || license_lower.contains("software license")) {
        return "Apache-2.0".to_string();
    }
    if license_lower.contains("bsd") && license_lower.contains("3") {
        return "BSD-3-Clause".to_string();
    }
    if license_lower.contains("bsd") && license_lower.contains("2") {
        return "BSD-2-Clause".to_string();
    }
    if license_lower.contains("bsd license") {
        return "BSD-3-Clause".to_string();
    }
    if license_lower.contains("gpl") && license_lower.contains("3") {
        return "GPL-3.0".to_string();
    }
    if license_lower.contains("gpl") && license_lower.contains("2") {
        return "GPL-2.0".to_string();
    }
    if license_lower.contains("lgpl") && license_lower.contains("3") {
        return "LGPL-3.0".to_string();
    }
    if license_lower.contains("lgpl") && license_lower.contains("2") {
        return "LGPL-2.1".to_string();
    }
    if license_lower.contains("mozilla public license") || license_lower == "mpl-2.0" {
        return "MPL-2.0".to_string();
    }
    if license_lower == "isc license" || license_lower == "isc" {
        return "ISC".to_string();
    }
    if license_lower.contains("unlicense") {
        return "Unlicense".to_string();
    }
    
    // Return original if no normalization found
    license.to_string()
}

fn is_osi_approved_license(license: &str) -> bool {
    // Common OSI-approved licenses
    let osi_licenses = [
        "MIT", "Apache-2.0", "Apache License", "BSD", "BSD-2-Clause", "BSD-3-Clause",
        "GPL-2.0", "GPL-3.0", "LGPL-2.1", "LGPL-3.0", "MPL-2.0", "ISC", "Unlicense",
        "CC0-1.0", "AGPL-3.0", "EPL-2.0", "Apache Software License",
    ];

    osi_licenses.iter().any(|&osi_license| {
        license.contains(osi_license) || 
        license.to_lowercase().contains(&osi_license.to_lowercase())
    })
}

pub fn get_effective_license(package: &PackageLicense) -> Option<String> {
    // Return license field if available
    if let Some(license) = &package.license {
        return Some(license.clone());
    }
    
    // Infer from classifiers if license field is null
    for classifier in &package.license_classifiers {
        if let Some(license_name) = extract_license_from_classifier(classifier) {
            return Some(license_name);
        }
    }
    
    None
}

fn extract_license_from_classifier(classifier: &str) -> Option<String> {
    // Extract license name from classifier like "License :: OSI Approved :: MIT License"
    if classifier.starts_with("License :: ") {
        let parts: Vec<&str> = classifier.split(" :: ").collect();
        if parts.len() >= 3 {
            return Some(parts[2].to_string());
        } else if parts.len() == 2 {
            return Some(parts[1].to_string());
        }
    }
    None
}
