use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use anyhow::{Context, Result};
use crate::policy::LicensePolicy;
use clap::ValueEnum;

#[derive(Debug, Clone, ValueEnum)]
pub enum BuiltinPolicy {
    Corporate,
    Permissive,
    Strict,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Built-in policy name or path to policy file
    pub policy: Option<String>,
    
    /// Output format (json, toml, csv)
    pub format: Option<String>,
    
    /// Include packages without license information
    pub include_unknown: Option<bool>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            policy: None,
            format: Some("json".to_string()),
            include_unknown: Some(false),
        }
    }
}

impl Config {
    /// Resolve policy from CLI arguments and config, with CLI taking precedence
    pub fn resolve_policy_from_cli(&self, cli_policy_file: Option<&PathBuf>, cli_policy: Option<&BuiltinPolicy>) -> Result<Option<LicensePolicy>> {
        let cli_policy_str = cli_policy.map(|p| match p {
            BuiltinPolicy::Corporate => "corporate",
            BuiltinPolicy::Permissive => "permissive",
            BuiltinPolicy::Strict => "strict",
        });
        
        self.resolve_policy(cli_policy_file, cli_policy_str)
    }
    
    /// Resolve policy from CLI arguments and config, with CLI taking precedence
    pub fn resolve_policy(&self, cli_policy_file: Option<&PathBuf>, cli_policy: Option<&str>) -> Result<Option<LicensePolicy>> {
        match (cli_policy_file, cli_policy) {
            (Some(policy_path), None) => Ok(Some(load_policy(policy_path)?)),
            (None, Some(builtin_policy)) => Ok(Some(load_builtin_policy(builtin_policy.to_string())?)),
            (None, None) => match &self.policy {
                Some(config_policy) => Ok(Some(load_builtin_policy(config_policy.clone())?)),
                None => Ok(None),
            },
            (Some(_), Some(_)) => unreachable!(), // conflicts_with prevents this
        }
    }
}

/// Load configuration from pyproject.toml
pub fn load_config() -> Result<Config> {
    let pyproject_path = std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("pyproject.toml");
    
    if !pyproject_path.exists() {
        return Ok(Config::default());
    }
    
    let content = fs::read_to_string(&pyproject_path)
        .with_context(|| format!("Failed to read pyproject.toml: {}", pyproject_path.display()))?;
    
    let pyproject: toml::Value = toml::from_str(&content)
        .with_context(|| format!("Failed to parse pyproject.toml: {}", pyproject_path.display()))?;
    
    // Extract [tool.py-license-auditor] section
    if let Some(tool) = pyproject.get("tool") {
        if let Some(py_license_auditor) = tool.get("py-license-auditor") {
            let config: Config = py_license_auditor.clone().try_into()
                .context("Failed to parse [tool.py-license-auditor] section")?;
            return Ok(config);
        }
    }
    
    Ok(Config::default())
}

// Helper functions for policy loading
fn load_policy(policy_path: &PathBuf) -> Result<LicensePolicy> {
    let content = std::fs::read_to_string(policy_path)
        .with_context(|| format!("Failed to read policy file: {}", policy_path.display()))?;
    
    let policy: LicensePolicy = toml::from_str(&content)
        .with_context(|| format!("Failed to parse policy file: {}", policy_path.display()))?;
    
    Ok(policy)
}

fn load_builtin_policy(policy_name: String) -> Result<LicensePolicy> {
    let content = match policy_name.as_str() {
        "corporate" => include_str!("../examples/policy-corporate.toml"),
        "permissive" => include_str!("../examples/policy-permissive.toml"),
        "strict" => include_str!("../examples/policy-strict.toml"),
        _ => return Err(anyhow::anyhow!("Unknown built-in policy: {}", policy_name)),
    };
    
    let policy: LicensePolicy = toml::from_str(content)
        .with_context(|| format!("Failed to parse built-in {} policy", policy_name))?;
    
    Ok(policy)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs;

    #[test]
    fn test_config_load_default() {
        let temp_dir = tempdir().unwrap();
        std::env::set_current_dir(&temp_dir).unwrap();
        
        let config = load_config().unwrap();
        assert_eq!(config.policy, None);
        assert_eq!(config.format, Some("json".to_string()));
        assert_eq!(config.include_unknown, Some(false));
    }

    #[test]
    fn test_config_load_from_pyproject() {
        let temp_dir = tempdir().unwrap();
        std::env::set_current_dir(&temp_dir).unwrap();
        
        let pyproject_content = r#"
[tool.py-license-auditor]
policy = "corporate"
format = "csv"
include_unknown = true
"#;
        fs::write("pyproject.toml", pyproject_content).unwrap();
        
        let config = load_config().unwrap();
        assert_eq!(config.policy, Some("corporate".to_string()));
        assert_eq!(config.format, Some("csv".to_string()));
        assert_eq!(config.include_unknown, Some(true));
    }

    #[test]
    fn test_policy_resolution_priority() {
        let config = Config {
            policy: Some("permissive".to_string()),
            format: Some("json".to_string()),
            include_unknown: Some(false),
        };
        
        // CLI policy takes precedence
        let cli_policy = BuiltinPolicy::Strict;
        let policy = config.resolve_policy_from_cli(None, Some(&cli_policy)).unwrap();
        assert!(policy.is_some());
        
        // Config policy used when no CLI policy
        let policy = config.resolve_policy_from_cli(None, None).unwrap();
        assert!(policy.is_some());
        
        // No policy when both are None
        let empty_config = Config::default();
        let policy = empty_config.resolve_policy_from_cli(None, None).unwrap();
        assert!(policy.is_none());
    }
}
