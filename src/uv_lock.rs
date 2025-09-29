use serde::Deserialize;
use std::path::Path;
use anyhow::{Result, Context};

#[derive(Debug, Clone, Deserialize)]
pub struct UvLockFile {
    pub version: u32,
    pub revision: Option<u32>,
    #[serde(rename = "requires-python")]
    pub requires_python: Option<String>,
    #[serde(rename = "resolution-markers")]
    pub resolution_markers: Option<Vec<String>>,
    #[serde(rename = "package")]
    pub packages: Vec<UvPackage>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UvPackage {
    pub name: String,
    pub version: String,
    pub source: Option<UvSource>,
    pub dependencies: Option<Vec<UvDependency>>,
    pub sdist: Option<UvDistribution>,
    pub wheels: Option<Vec<UvDistribution>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UvSource {
    pub registry: Option<String>,
    pub git: Option<String>,
    pub path: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum UvDependency {
    Simple(String),
    Complex { name: String, marker: Option<String> },
}

#[derive(Debug, Clone, Deserialize)]
pub struct UvDistribution {
    pub url: String,
    pub hash: String,
    pub size: Option<u64>,
    #[serde(rename = "upload-time")]
    pub upload_time: Option<String>,
}

pub struct UvLockParser;

impl UvLockParser {
    /// Parse uv.lock file and return structured data
    pub fn parse_uv_lock<P: AsRef<Path>>(path: P) -> Result<UvLockFile> {
        let path_ref = path.as_ref();
        
        // Check if file exists
        if !path_ref.exists() {
            return Err(anyhow::anyhow!("uv.lock file not found: {}", path_ref.display()));
        }
        
        // Check if file is readable
        let content = std::fs::read_to_string(path_ref)
            .with_context(|| format!("Failed to read uv.lock file: {}", path_ref.display()))?;
        
        // Check if file is empty
        if content.trim().is_empty() {
            return Err(anyhow::anyhow!("uv.lock file is empty: {}", path_ref.display()));
        }
        
        // Parse TOML with detailed error information
        let lock_file: UvLockFile = toml::from_str(&content)
            .with_context(|| {
                format!("Failed to parse uv.lock file as TOML: {}\nThis might indicate a corrupted or incompatible uv.lock file.", path_ref.display())
            })?;
        
        // Validate the parsed structure
        if lock_file.packages.is_empty() {
            eprintln!("Warning: uv.lock file contains no packages: {}", path_ref.display());
        }
        
        Ok(lock_file)
    }

    /// Extract package names and versions from uv.lock
    pub fn extract_packages(lock_file: &UvLockFile) -> Vec<(String, String)> {
        lock_file.packages
            .iter()
            .filter_map(|pkg| {
                // Validate package name and version
                if pkg.name.trim().is_empty() {
                    eprintln!("Warning: Skipping package with empty name");
                    return None;
                }
                if pkg.version.trim().is_empty() {
                    eprintln!("Warning: Skipping package '{}' with empty version", pkg.name);
                    return None;
                }
                Some((pkg.name.clone(), pkg.version.clone()))
            })
            .collect()
    }

    /// Find uv.lock file in current directory or parent directories
    pub fn find_uv_lock() -> Option<std::path::PathBuf> {
        let mut current = std::env::current_dir().ok()?;
        
        loop {
            let uv_lock_path = current.join("uv.lock");
            if uv_lock_path.exists() {
                // Validate that the file is readable and not empty
                if let Ok(metadata) = std::fs::metadata(&uv_lock_path) {
                    if metadata.len() > 0 {
                        return Some(uv_lock_path);
                    } else {
                        eprintln!("Warning: Found empty uv.lock file at {}, continuing search...", uv_lock_path.display());
                    }
                } else {
                    eprintln!("Warning: Found uv.lock file at {} but cannot read metadata, continuing search...", uv_lock_path.display());
                }
            }
            
            if !current.pop() {
                break;
            }
        }
        
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_parse_simple_uv_lock() {
        let uv_lock_content = r#"
version = 1
revision = 3
requires-python = ">=3.10"

[[package]]
name = "requests"
version = "2.31.0"
source = { registry = "https://pypi.org/simple" }

[[package]]
name = "click"
version = "8.1.7"
source = { registry = "https://pypi.org/simple" }
dependencies = [
    { name = "colorama", marker = "platform_system == 'Windows'" },
]
"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(uv_lock_content.as_bytes()).unwrap();
        
        let lock_file = UvLockParser::parse_uv_lock(temp_file.path()).unwrap();
        
        assert_eq!(lock_file.version, 1);
        assert_eq!(lock_file.packages.len(), 2);
        assert_eq!(lock_file.packages[0].name, "requests");
        assert_eq!(lock_file.packages[0].version, "2.31.0");
    }

    #[test]
    fn test_extract_packages() {
        let lock_file = UvLockFile {
            version: 1,
            revision: None,
            requires_python: None,
            resolution_markers: None,
            packages: vec![
                UvPackage {
                    name: "requests".to_string(),
                    version: "2.31.0".to_string(),
                    source: None,
                    dependencies: None,
                    sdist: None,
                    wheels: None,
                },
                UvPackage {
                    name: "click".to_string(),
                    version: "8.1.7".to_string(),
                    source: None,
                    dependencies: None,
                    sdist: None,
                    wheels: None,
                },
            ],
        };

        let packages = UvLockParser::extract_packages(&lock_file);
        assert_eq!(packages.len(), 2);
        assert_eq!(packages[0], ("requests".to_string(), "2.31.0".to_string()));
        assert_eq!(packages[1], ("click".to_string(), "8.1.7".to_string()));
    }

    #[test]
    fn test_integration_with_license_extraction() {
        // This test verifies that the uv.lock integration works end-to-end
        use crate::license::extract_licenses_from_uv_lock;
        use tempfile::TempDir;
        use std::fs;

        let temp_dir = TempDir::new().unwrap();
        let uv_lock_path = temp_dir.path().join("uv.lock");
        
        let uv_lock_content = r#"
version = 1
revision = 3
requires-python = ">=3.10"

[[package]]
name = "test-package"
version = "1.0.0"
source = { registry = "https://pypi.org/simple" }
"#;

        fs::write(&uv_lock_path, uv_lock_content).unwrap();
        
        // Create a fake site-packages directory
        let site_packages = temp_dir.path().join("site-packages");
        fs::create_dir(&site_packages).unwrap();
        
        // Test that the function handles missing packages gracefully
        let result = extract_licenses_from_uv_lock(Some(uv_lock_path), Some(site_packages), false);
        assert!(result.is_ok());
        
        let packages = result.unwrap();
        // With include_unknown=false, packages without license info are filtered out
        assert_eq!(packages.len(), 0);
    }
}
