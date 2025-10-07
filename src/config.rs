use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use anyhow::{Context, Result};
use crate::policy::{LicensePolicy, PackageException};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Output format (json, table, csv)
    pub format: Option<String>,
    
    /// Include packages without license information
    pub include_unknown: Option<bool>,
    
    /// Check for policy violations
    pub check_violations: Option<bool>,
    
    /// Fail on policy violations
    pub fail_on_violations: Option<bool>,
    
    /// Embedded policy configuration
    pub policy: Option<LicensePolicy>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            format: Some("table".to_string()),  // Default to table
            include_unknown: Some(false),
            check_violations: Some(false),
            fail_on_violations: Some(false),
            policy: None,
        }
    }
}

impl Config {
    // Config struct now directly contains the policy, no resolution needed
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

/// Add exceptions to pyproject.toml
pub fn add_exceptions_to_config(exceptions: Vec<PackageException>) -> Result<()> {
    let pyproject_path = std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("pyproject.toml");
    
    if !pyproject_path.exists() {
        return Err(anyhow::anyhow!("pyproject.toml not found. Run 'py-license-auditor init <policy>' first."));
    }
    
    let content = fs::read_to_string(&pyproject_path)
        .with_context(|| format!("Failed to read pyproject.toml: {}", pyproject_path.display()))?;
    
    let mut pyproject: toml::Value = toml::from_str(&content)
        .with_context(|| format!("Failed to parse pyproject.toml: {}", pyproject_path.display()))?;
    
    // Navigate to [tool.py-license-auditor.policy.exceptions]
    let tool = pyproject.get_mut("tool")
        .ok_or_else(|| anyhow::anyhow!("No [tool] section found in pyproject.toml"))?;
    
    let py_license_auditor = tool.get_mut("py-license-auditor")
        .ok_or_else(|| anyhow::anyhow!("No [tool.py-license-auditor] section found"))?;
    
    let policy = py_license_auditor.get_mut("policy")
        .ok_or_else(|| anyhow::anyhow!("No policy configuration found"))?;
    
    // Get or create exceptions array
    let exceptions_array = policy.get_mut("exceptions")
        .and_then(|v| v.as_array_mut())
        .ok_or_else(|| anyhow::anyhow!("Invalid exceptions format"))?;
    
    // Add new exceptions
    for exception in exceptions {
        let exception_value = toml::Value::try_from(&exception)
            .context("Failed to serialize exception")?;
        exceptions_array.push(exception_value);
    }
    
    // Write back to file
    let updated_content = toml::to_string_pretty(&pyproject)
        .context("Failed to serialize updated pyproject.toml")?;
    
    fs::write(&pyproject_path, updated_content)
        .with_context(|| format!("Failed to write pyproject.toml: {}", pyproject_path.display()))?;
    
    Ok(())
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
        assert_eq!(config.format, Some("table".to_string()));  // Updated to table
        assert_eq!(config.include_unknown, Some(false));
        assert_eq!(config.check_violations, Some(false));
        assert_eq!(config.fail_on_violations, Some(false));
    }

    #[test]
    fn test_config_load_from_pyproject() {
        let temp_dir = tempdir().unwrap();
        std::env::set_current_dir(&temp_dir).unwrap();
        
        let pyproject_content = r#"
[tool.py-license-auditor]
format = "csv"
include_unknown = true
check_violations = true
fail_on_violations = true

[tool.py-license-auditor.policy]
name = "Test Policy"
description = "Test policy for unit tests"

[tool.py-license-auditor.policy.allowed_licenses]
exact = ["MIT", "Apache-2.0"]
patterns = []

[tool.py-license-auditor.policy.forbidden_licenses]
exact = ["GPL-3.0"]
patterns = []

[tool.py-license-auditor.policy.review_required]
exact = []
patterns = []

[[tool.py-license-auditor.policy.exceptions]]
name = "test-package"
version = "1.0.0"
reason = "Test exception"
"#;
        fs::write("pyproject.toml", pyproject_content).unwrap();
        
        let config = load_config().unwrap();
        assert_eq!(config.format, Some("csv".to_string()));
        assert_eq!(config.include_unknown, Some(true));
        assert_eq!(config.check_violations, Some(true));
        assert_eq!(config.fail_on_violations, Some(true));
        assert!(config.policy.is_some());
        
        let policy = config.policy.unwrap();
        assert_eq!(policy.name, "Test Policy");
        assert_eq!(policy.allowed_licenses.exact, vec!["MIT", "Apache-2.0"]);
    }

    #[test]
    fn test_embedded_policy_architecture() {
        let temp_dir = tempdir().unwrap();
        std::env::set_current_dir(&temp_dir).unwrap();
        
        // Test that the new architecture works end-to-end
        let pyproject_content = r#"
[tool.py-license-auditor]
format = "json"
include_unknown = false
check_violations = true
fail_on_violations = true

[tool.py-license-auditor.policy]
name = "Embedded Policy Test"
description = "Test embedded policy functionality"

[tool.py-license-auditor.policy.allowed_licenses]
exact = ["MIT", "Apache-2.0", "BSD-3-Clause"]
patterns = ["BSD-*"]

[tool.py-license-auditor.policy.forbidden_licenses]
exact = ["GPL-3.0"]
patterns = ["GPL-*"]

[tool.py-license-auditor.policy.review_required]
exact = ["MPL-2.0"]
patterns = []

[[tool.py-license-auditor.policy.exceptions]]
name = "legacy-package"
version = "1.0.0"
reason = "Legacy dependency, approved by legal team"
"#;
        fs::write("pyproject.toml", pyproject_content).unwrap();
        
        let config = load_config().unwrap();
        
        // Verify all config fields
        assert_eq!(config.format, Some("json".to_string()));
        assert_eq!(config.include_unknown, Some(false));
        assert_eq!(config.check_violations, Some(true));
        assert_eq!(config.fail_on_violations, Some(true));
        
        // Verify embedded policy
        assert!(config.policy.is_some());
        let policy = config.policy.unwrap();
        assert_eq!(policy.name, "Embedded Policy Test");
        assert_eq!(policy.description, Some("Test embedded policy functionality".to_string()));
        
        // Verify policy rules
        assert_eq!(policy.allowed_licenses.exact, vec!["MIT", "Apache-2.0", "BSD-3-Clause"]);
        assert_eq!(policy.allowed_licenses.patterns, vec!["BSD-*"]);
        assert_eq!(policy.forbidden_licenses.exact, vec!["GPL-3.0"]);
        assert_eq!(policy.forbidden_licenses.patterns, vec!["GPL-*"]);
        assert_eq!(policy.review_required.exact, vec!["MPL-2.0"]);
        assert!(policy.review_required.patterns.is_empty());
    }
}
