use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use super::PackageLicense;

/// Extract license information from all packages in site-packages directory
pub fn extract_all_licenses(site_packages_path: &Path, include_unknown: bool) -> Result<Vec<PackageLicense>> {
    let mut packages = Vec::new();

    for entry in fs::read_dir(site_packages_path)? {
        let entry = entry?;
        let file_name = entry.file_name();
        let name_str = file_name.to_string_lossy();

        if name_str.ends_with(".dist-info") {
            if let Some(package) = extract_from_dist_info(&entry.path())? {
                if include_unknown || package.license.is_some() || !package.license_classifiers.is_empty() {
                    packages.push(package);
                }
            }
        } else if name_str.ends_with(".egg-info") {
            if let Some(package) = extract_from_egg_info(&entry.path())? {
                if include_unknown || package.license.is_some() || !package.license_classifiers.is_empty() {
                    packages.push(package);
                }
            }
        }
    }

    packages.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(packages)
}

/// Extract license information for a specific package by name
pub fn extract_license_for_package(site_packages_path: &Path, package_name: &str) -> Result<PackageLicense> {
    // Try .dist-info first (modern format)
    let dist_info_pattern = format!("{}-*.dist-info", package_name.replace("-", "_"));
    for entry in fs::read_dir(site_packages_path)? {
        let entry = entry?;
        let file_name = entry.file_name();
        let name_str = file_name.to_string_lossy();
        
        if name_str.starts_with(&package_name.replace("-", "_")) && name_str.ends_with(".dist-info") {
            if let Some(package) = extract_from_dist_info(&entry.path())? {
                return Ok(package);
            }
        }
    }

    // Try .egg-info (legacy format)
    let egg_info_pattern = format!("{}-*.egg-info", package_name.replace("-", "_"));
    for entry in fs::read_dir(site_packages_path)? {
        let entry = entry?;
        let file_name = entry.file_name();
        let name_str = file_name.to_string_lossy();
        
        if name_str.starts_with(&package_name.replace("-", "_")) && name_str.ends_with(".egg-info") {
            if let Some(package) = extract_from_egg_info(&entry.path())? {
                return Ok(package);
            }
        }
    }

    anyhow::bail!("Package '{}' not found in site-packages", package_name)
}

fn extract_from_dist_info(dist_info_path: &Path) -> Result<Option<PackageLicense>> {
    let metadata_path = dist_info_path.join("METADATA");
    if !metadata_path.exists() {
        return Ok(None);
    }

    let content = fs::read_to_string(&metadata_path)
        .with_context(|| format!("Failed to read {}", metadata_path.display()))?;

    let (name, version) = parse_name_version_from_dist_info(dist_info_path)?;
    let (license, classifiers) = parse_metadata_content(&content);

    Ok(Some(PackageLicense {
        name,
        version,
        license,
        license_classifiers: classifiers,
        metadata_source: "METADATA".to_string(),
    }))
}

fn extract_from_egg_info(egg_info_path: &Path) -> Result<Option<PackageLicense>> {
    let pkg_info_path = egg_info_path.join("PKG-INFO");
    if !pkg_info_path.exists() {
        return Ok(None);
    }

    let content = fs::read_to_string(&pkg_info_path)
        .with_context(|| format!("Failed to read {}", pkg_info_path.display()))?;

    let (name, version) = parse_name_version_from_egg_info(egg_info_path)?;
    let (license, classifiers) = parse_metadata_content(&content);

    Ok(Some(PackageLicense {
        name,
        version,
        license,
        license_classifiers: classifiers,
        metadata_source: "PKG-INFO".to_string(),
    }))
}

fn parse_name_version_from_dist_info(dist_info_path: &Path) -> Result<(String, Option<String>)> {
    let file_name = dist_info_path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| anyhow::anyhow!("Invalid dist-info directory name"))?;

    let name_version = file_name.strip_suffix(".dist-info")
        .ok_or_else(|| anyhow::anyhow!("Invalid dist-info directory name"))?;

    // Split by the last occurrence of '-' to separate name and version
    if let Some(last_dash) = name_version.rfind('-') {
        let name = name_version[..last_dash].to_string();
        let version = name_version[last_dash + 1..].to_string();
        Ok((name, Some(version)))
    } else {
        Ok((name_version.to_string(), None))
    }
}

fn parse_name_version_from_egg_info(egg_info_path: &Path) -> Result<(String, Option<String>)> {
    let file_name = egg_info_path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| anyhow::anyhow!("Invalid egg-info directory name"))?;

    let name_version = file_name.strip_suffix(".egg-info")
        .ok_or_else(|| anyhow::anyhow!("Invalid egg-info directory name"))?;

    // Split by the last occurrence of '-' to separated name and version
    if let Some(last_dash) = name_version.rfind('-') {
        let name = name_version[..last_dash].to_string();
        let version = name_version[last_dash + 1..].to_string();
        Ok((name, Some(version)))
    } else {
        Ok((name_version.to_string(), None))
    }
}

fn parse_metadata_content(content: &str) -> (Option<String>, Vec<String>) {
    let mut license = None;
    let mut classifiers = Vec::new();

    for line in content.lines() {
        if let Some(value) = line.strip_prefix("License: ") {
            if !value.trim().is_empty() && value.trim() != "UNKNOWN" {
                license = Some(value.trim().to_string());
            }
        } else if let Some(value) = line.strip_prefix("Classifier: ") {
            if value.contains("License") {
                classifiers.push(value.trim().to_string());
            }
        }
    }

    (license, classifiers)
}
