use std::io::{self, Write};
use anyhow::Result;
use chrono::{Utc, NaiveDate};
use super::models::Exception;
use super::storage::{load_exceptions, save_exceptions};
use crate::policy::ViolationSummary;

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
    let default_expiry = today + chrono::Duration::days(90); // 3 months default
    
    print!("   Expiry date (YYYY-MM-DD) [{}]: ", default_expiry.format("%Y-%m-%d"));
    io::stdout().flush()?;
    let mut expiry_input = String::new();
    io::stdin().read_line(&mut expiry_input)?;
    let expiry_input = expiry_input.trim();
    
    let expires = if expiry_input.is_empty() {
        Some(default_expiry)
    } else if expiry_input.to_lowercase() == "never" || expiry_input.to_lowercase() == "permanent" {
        None
    } else {
        match NaiveDate::parse_from_str(expiry_input, "%Y-%m-%d") {
            Ok(date) => Some(date),
            Err(_) => {
                println!("   Invalid date format, using default: {}", default_expiry.format("%Y-%m-%d"));
                Some(default_expiry)
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
    
    Ok(Some(exception))
}

/// Handle interactive exception processing for violations
pub fn handle_interactive_exceptions(violations: ViolationSummary) -> Result<ViolationSummary> {
    if violations.details.is_empty() {
        return Ok(violations);
    }

    let mut exceptions = load_exceptions()?;
    let mut exceptions_added = 0;
    let mut remaining_violations = Vec::new();
    
    for violation in violations.details {
        // Check if already excepted
        if exceptions.is_excepted(&violation.package_name, violation.package_version.as_deref()) {
            continue;
        }
        
        let violation_type = match violation.violation_level {
            crate::policy::ViolationLevel::Forbidden => "Forbidden license",
            crate::policy::ViolationLevel::ReviewRequired => "Review required",
            crate::policy::ViolationLevel::Unknown => "Unknown license",
            _ => "Violation",
        };
        
        if let Some(exception) = prompt_for_exception(
            &violation.package_name,
            violation.package_version.as_deref(),
            &violation.license.as_ref().unwrap_or(&"Unknown".to_string()),
            violation_type,
        )? {
            exceptions.add_exception(exception);
            exceptions_added += 1;
        } else {
            remaining_violations.push(violation);
        }
    }
    
    // Save exceptions if any were added
    if exceptions_added > 0 {
        save_exceptions(&exceptions)?;
        eprintln!("Added {} exceptions to .exceptions.toml", exceptions_added);
    }
    
    // Update violation summary with remaining violations
    let errors = remaining_violations.iter().filter(|v| v.violation_level == crate::policy::ViolationLevel::Forbidden).count();
    let warnings = remaining_violations.iter().filter(|v| 
        v.violation_level == crate::policy::ViolationLevel::ReviewRequired || 
        v.violation_level == crate::policy::ViolationLevel::Unknown
    ).count();
    
    Ok(ViolationSummary {
        total: remaining_violations.len(),
        errors,
        warnings,
        details: remaining_violations,
    })
}
