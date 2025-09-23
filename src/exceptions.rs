use serde::{Deserialize, Serialize};
use std::io::{self, Write};
use std::fs;
use std::path::Path;
use anyhow::{Context, Result};
use chrono::{DateTime, Utc, NaiveDate};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Exception {
    pub name: String,
    pub version: Option<String>,
    pub reason: String,
    pub added_by: Option<String>,
    pub added_date: DateTime<Utc>,
    pub expires: Option<NaiveDate>,
    pub permanent: bool,
    pub added_interactively: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExceptionsFile {
    pub exceptions: Vec<Exception>,
}

impl ExceptionsFile {
    pub fn new() -> Self {
        Self {
            exceptions: Vec::new(),
        }
    }

    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read exceptions file: {}", path.as_ref().display()))?;
        
        let exceptions: ExceptionsFile = toml::from_str(&content)
            .with_context(|| format!("Failed to parse exceptions file: {}", path.as_ref().display()))?;
        
        Ok(exceptions)
    }

    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content = toml::to_string_pretty(self)
            .context("Failed to serialize exceptions")?;
        
        fs::write(&path, content)
            .with_context(|| format!("Failed to write exceptions file: {}", path.as_ref().display()))?;
        
        Ok(())
    }

    pub fn add_exception(&mut self, exception: Exception) {
        self.exceptions.push(exception);
    }

    pub fn is_excepted(&self, package_name: &str, package_version: Option<&str>) -> bool {
        self.exceptions.iter().any(|exc| {
            // Check if exception has expired
            if let Some(expires) = exc.expires {
                if Utc::now().date_naive() > expires {
                    return false;
                }
            }

            // Check package name match
            if exc.name != package_name {
                return false;
            }

            // Check version match (if specified)
            match (&exc.version, package_version) {
                (Some(exc_version), Some(pkg_version)) => {
                    exc_version == "*" || exc_version == pkg_version
                }
                (Some(_), None) => false,
                (None, _) => true,
            }
        })
    }

    pub fn cleanup_expired(&mut self) -> usize {
        let original_count = self.exceptions.len();
        let today = Utc::now().date_naive();
        
        self.exceptions.retain(|exc| {
            match exc.expires {
                Some(expires) => today <= expires,
                None => true, // Keep permanent exceptions
            }
        });
        
        original_count - self.exceptions.len()
    }
}

pub fn get_exceptions_file_path() -> std::path::PathBuf {
    std::env::current_dir()
        .unwrap_or_else(|_| std::path::PathBuf::from("."))
        .join(".exceptions.toml")
}

pub fn load_exceptions() -> Result<ExceptionsFile> {
    let path = get_exceptions_file_path();
    if path.exists() {
        ExceptionsFile::load_from_file(path)
    } else {
        Ok(ExceptionsFile::new())
    }
}

pub fn save_exceptions(exceptions: &ExceptionsFile) -> Result<()> {
    let path = get_exceptions_file_path();
    exceptions.save_to_file(path)
}

pub fn prompt_for_exception(package_name: &str, package_version: Option<&str>, license: &str, violation_type: &str) -> Result<Option<Exception>> {
    println!("\n❌ Package: {} ({})", package_name, package_version.unwrap_or("unknown"));
    println!("   License: {}", license);
    println!("   Violation: {}", violation_type);
    
    print!("   Add exception for this package? [y/N]: ");
    io::stdout().flush()?;
    
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    
    if !input.trim().to_lowercase().starts_with('y') {
        return Ok(None);
    }
    
    // Get reason
    print!("   Reason [research/migration/legacy]: ");
    io::stdout().flush()?;
    let mut reason = String::new();
    io::stdin().read_line(&mut reason)?;
    let reason = reason.trim();
    
    let reason = if reason.is_empty() {
        "temporary exception".to_string()
    } else {
        match reason {
            "research" => "research prototype".to_string(),
            "migration" => "migration in progress".to_string(),
            "legacy" => "legacy compatibility".to_string(),
            _ => reason.to_string(),
        }
    };
    
    // Get expiration date
    let today = Utc::now().date_naive();
    let default_expires = today + chrono::Duration::days(90); // 3 months default
    
    print!("   Expires (YYYY-MM-DD) [{}]: ", default_expires.format("%Y-%m-%d"));
    io::stdout().flush()?;
    let mut expires_input = String::new();
    io::stdin().read_line(&mut expires_input)?;
    let expires_input = expires_input.trim();
    
    let expires = if expires_input.is_empty() {
        Some(default_expires)
    } else if expires_input.to_lowercase() == "permanent" {
        None
    } else {
        match NaiveDate::parse_from_str(expires_input, "%Y-%m-%d") {
            Ok(date) => Some(date),
            Err(_) => {
                println!("   Invalid date format, using default: {}", default_expires.format("%Y-%m-%d"));
                Some(default_expires)
            }
        }
    };
    
    let exception = Exception {
        name: package_name.to_string(),
        version: package_version.map(|v| v.to_string()),
        reason,
        added_by: std::env::var("USER").ok().or_else(|| std::env::var("USERNAME").ok()),
        added_date: Utc::now(),
        expires,
        permanent: expires.is_none(),
        added_interactively: true,
    };
    
    println!("   ✅ Exception added for {}", package_name);
    
    Ok(Some(exception))
}
