use anyhow::{Context, Result};
use clap::{Parser, ValueEnum};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(name = "uv-license-extractor")]
#[command(about = "Extract license information from Python packages")]
struct Cli {
    /// Path to site-packages directory or virtual environment
    #[arg(short, long)]
    path: Option<PathBuf>,

    /// Output format
    #[arg(short, long, default_value = "json")]
    format: OutputFormat,

    /// Output file (default: stdout)
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Include packages without license information
    #[arg(long)]
    include_unknown: bool,
}

#[derive(Clone, ValueEnum)]
enum OutputFormat {
    Json,
    Toml,
    Csv,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PackageLicense {
    name: String,
    version: Option<String>,
    license: Option<String>,
    license_classifiers: Vec<String>,
    metadata_source: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LicenseReport {
    packages: Vec<PackageLicense>,
    summary: LicenseSummary,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LicenseSummary {
    total_packages: usize,
    with_license: usize,
    without_license: usize,
    license_types: HashMap<String, usize>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let site_packages_path = find_site_packages_path(cli.path)?;
    println!("Scanning: {}", site_packages_path.display());

    let packages = extract_all_licenses(&site_packages_path, cli.include_unknown)?;
    let report = create_report(packages);

    let output = match cli.format {
        OutputFormat::Json => serde_json::to_string_pretty(&report)?,
        OutputFormat::Toml => toml::to_string_pretty(&report)?,
        OutputFormat::Csv => format_as_csv(&report.packages)?,
    };

    match cli.output {
        Some(path) => fs::write(path, output)?,
        None => println!("{}", output),
    }

    Ok(())
}

fn find_site_packages_path(path: Option<PathBuf>) -> Result<PathBuf> {
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

fn extract_all_licenses(site_packages_path: &Path, include_unknown: bool) -> Result<Vec<PackageLicense>> {
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

pub fn parse_name_version_from_dist_info(dist_info_path: &Path) -> Result<(String, Option<String>)> {
    let file_name = dist_info_path.file_name()
        .and_then(|n| n.to_str())
        .context("Invalid dist-info directory name")?;
    
    let name_version = file_name.strip_suffix(".dist-info")
        .context("Invalid dist-info directory name")?;
    
    if let Some(dash_pos) = name_version.rfind('-') {
        let name = name_version[..dash_pos].to_string();
        let version = name_version[dash_pos + 1..].to_string();
        Ok((name, Some(version)))
    } else {
        Ok((name_version.to_string(), None))
    }
}

fn parse_name_version_from_egg_info(egg_info_path: &Path) -> Result<(String, Option<String>)> {
    let file_name = egg_info_path.file_name()
        .and_then(|n| n.to_str())
        .context("Invalid egg-info directory name")?;
    
    let name_version = file_name.strip_suffix(".egg-info")
        .context("Invalid egg-info directory name")?;
    
    if let Some(dash_pos) = name_version.rfind('-') {
        let name = name_version[..dash_pos].to_string();
        let version = name_version[dash_pos + 1..].to_string();
        Ok((name, Some(version)))
    } else {
        Ok((name_version.to_string(), None))
    }
}

pub fn parse_metadata_content(content: &str) -> (Option<String>, Vec<String>) {
    let mut license = None;
    let mut classifiers = Vec::new();

    for line in content.lines() {
        if let Some(license_value) = line.strip_prefix("License: ") {
            let trimmed = license_value.trim();
            if !trimmed.is_empty() && trimmed != "UNKNOWN" {
                license = Some(trimmed.to_string());
            }
        } else if let Some(classifier) = line.strip_prefix("Classifier: ") {
            let trimmed = classifier.trim();
            if trimmed.contains("License") {
                classifiers.push(trimmed.to_string());
            }
        }
    }

    (license, classifiers)
}

pub fn create_report(packages: Vec<PackageLicense>) -> LicenseReport {
    let total_packages = packages.len();
    let with_license = packages.iter()
        .filter(|p| p.license.is_some() || !p.license_classifiers.is_empty())
        .count();
    let without_license = total_packages - with_license;

    let mut license_types = HashMap::new();
    for package in &packages {
        if let Some(ref license) = package.license {
            *license_types.entry(license.clone()).or_insert(0) += 1;
        }
        for classifier in &package.license_classifiers {
            *license_types.entry(classifier.clone()).or_insert(0) += 1;
        }
    }

    LicenseReport {
        packages,
        summary: LicenseSummary {
            total_packages,
            with_license,
            without_license,
            license_types,
        },
    }
}

fn format_as_csv(packages: &[PackageLicense]) -> Result<String> {
    let mut csv = String::from("name,version,license,license_classifiers,metadata_source\n");
    
    for package in packages {
        let version = package.version.as_deref().unwrap_or("");
        let license = package.license.as_deref().unwrap_or("");
        let classifiers = package.license_classifiers.join("; ");
        
        csv.push_str(&format!(
            "{},{},{},\"{}\",{}\n",
            package.name, version, license, classifiers, package.metadata_source
        ));
    }
    
    Ok(csv)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_parse_metadata_content() {
        let content = r#"Name: test-package
Version: 1.0.0
License: MIT
Classifier: License :: OSI Approved :: MIT License
Classifier: Programming Language :: Python :: 3
"#;

        let (license, classifiers) = parse_metadata_content(content);
        assert_eq!(license, Some("MIT".to_string()));
        assert_eq!(classifiers, vec!["License :: OSI Approved :: MIT License"]);
    }

    #[test]
    fn test_parse_metadata_content_no_license() {
        let content = r#"Name: test-package
Version: 1.0.0
"#;

        let (license, classifiers) = parse_metadata_content(content);
        assert_eq!(license, None);
        assert!(classifiers.is_empty());
    }

    #[test]
    fn test_parse_metadata_content_unknown_license() {
        let content = r#"Name: test-package
Version: 1.0.0
License: UNKNOWN
"#;

        let (license, _classifiers) = parse_metadata_content(content);
        assert_eq!(license, None);
    }

    #[test]
    fn test_parse_name_version_from_dist_info() {
        let temp_dir = TempDir::new().unwrap();
        let dist_info_path = temp_dir.path().join("requests-2.31.0.dist-info");
        fs::create_dir(&dist_info_path).unwrap();

        let (name, version) = parse_name_version_from_dist_info(&dist_info_path).unwrap();
        assert_eq!(name, "requests");
        assert_eq!(version, Some("2.31.0".to_string()));
    }

    #[test]
    fn test_create_report() {
        let packages = vec![
            PackageLicense {
                name: "requests".to_string(),
                version: Some("2.31.0".to_string()),
                license: Some("Apache-2.0".to_string()),
                license_classifiers: vec!["License :: OSI Approved :: Apache Software License".to_string()],
                metadata_source: "METADATA".to_string(),
            },
            PackageLicense {
                name: "click".to_string(),
                version: Some("8.1.7".to_string()),
                license: None,
                license_classifiers: vec![],
                metadata_source: "METADATA".to_string(),
            },
        ];

        let report = create_report(packages);
        assert_eq!(report.summary.total_packages, 2);
        assert_eq!(report.summary.with_license, 1);
        assert_eq!(report.summary.without_license, 1);
        assert_eq!(report.summary.license_types.get("Apache-2.0"), Some(&1));
    }
}
