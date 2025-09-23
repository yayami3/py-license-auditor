use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use anyhow::Result;
use std::fs;
use std::path::PathBuf;

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

#[derive(Debug, Serialize, Deserialize)]
pub struct LicenseTypes {
    pub osi_approved: Vec<(String, usize)>,
    pub non_osi: Vec<(String, usize)>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LicenseSummary {
    pub total_packages: usize,
    pub with_license: usize,
    pub without_license: usize,
    pub license_types: LicenseTypes,
}

#[derive(Debug, Serialize, Deserialize)]
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

pub fn create_report(packages: Vec<PackageLicense>) -> LicenseReport {
    let total_packages = packages.len();
    let with_license = packages.iter()
        .filter(|p| p.license.is_some() || !p.license_classifiers.is_empty())
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

    // Convert HashMap to Vec and sort by count (descending)
    let mut osi_approved: Vec<(String, usize)> = osi_counts.into_iter().collect();
    osi_approved.sort_by(|a, b| b.1.cmp(&a.1));

    let mut non_osi: Vec<(String, usize)> = non_osi_counts.into_iter().collect();
    non_osi.sort_by(|a, b| b.1.cmp(&a.1));

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

fn normalize_license_name(license: &str) -> String {
    let license_lower = license.to_lowercase();
    
    // Common license normalizations
    if license_lower.contains("mit") {
        return "MIT".to_string();
    }
    if license_lower.contains("apache") && license_lower.contains("2.0") {
        return "Apache-2.0".to_string();
    }
    if license_lower.contains("apache software license") {
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
