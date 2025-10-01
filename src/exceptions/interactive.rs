use std::io::{self, Write};
use std::collections::HashMap;
use anyhow::Result;
use chrono::{Utc, NaiveDate};
use super::models::Exception;
use super::storage::{load_exceptions, save_exceptions};
use crate::policy::{ViolationSummary, Violation};
use crate::license::normalize_license_name;

#[derive(Debug)]
struct LicenseGroup {
    license: String,
    normalized_license: String,
    packages: Vec<Violation>,
    violation_type: String,
}

fn group_violations_by_license(violations: Vec<Violation>) -> Vec<LicenseGroup> {
    let mut groups: HashMap<String, LicenseGroup> = HashMap::new();
    
    for violation in violations {
        let license = violation.license.as_ref().unwrap_or(&"Unknown".to_string()).clone();
        let normalized = normalize_license_name(&license);
        
        let violation_type = match violation.violation_level {
            crate::policy::ViolationLevel::Forbidden => "Forbidden license",
            crate::policy::ViolationLevel::ReviewRequired => "Review required", 
            crate::policy::ViolationLevel::Unknown => "Unknown license",
            _ => "Violation",
        };
        
        groups.entry(normalized.clone())
            .or_insert_with(|| LicenseGroup {
                license: license.clone(),
                normalized_license: normalized,
                packages: Vec::new(),
                violation_type: violation_type.to_string(),
            })
            .packages.push(violation);
    }
    
    let mut result: Vec<_> = groups.into_values().collect();
    result.sort_by(|a, b| a.normalized_license.cmp(&b.normalized_license));
    result
}

fn display_license_group(group: &LicenseGroup, group_num: usize, total_groups: usize) {
    println!("\n🔍 License Group [{}/{}]", group_num, total_groups);
    println!("   License: {} (normalized: {})", group.license, group.normalized_license);
    println!("   Violation: {}", group.violation_type);
    println!("   Affected packages ({}):", group.packages.len());
    
    for (i, violation) in group.packages.iter().enumerate() {
        let version = violation.package_version.as_deref().unwrap_or("unknown");
        if i < 5 {
            println!("     • {} ({})", violation.package_name, version);
        } else if i == 5 {
            println!("     • ... and {} more packages", group.packages.len() - 5);
            break;
        }
    }
}

fn prompt_for_group_exception(group: &LicenseGroup, group_num: usize, total_groups: usize) -> Result<Option<(String, Option<NaiveDate>)>> {
    display_license_group(group, group_num, total_groups);
    
    print!("   Add exception for ALL packages with this license? [y/N/s(kip)/q(uit)]: ");
    io::stdout().flush()?;
    
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim().to_lowercase();
    
    match input.as_str() {
        "q" | "quit" => return Err(anyhow::anyhow!("User quit")),
        "s" | "skip" => return Ok(None),
        input if input.starts_with('y') => {},
        _ => return Ok(None),
    }
    
    // Get reason
    print!("   Reason [research/migration/legacy]: ");
    io::stdout().flush()?;
    let mut reason = String::new();
    io::stdin().read_line(&mut reason)?;
    let reason = reason.trim();
    
    let reason = if reason.is_empty() {
        format!("temporary exception for {} license", group.normalized_license)
    } else {
        match reason {
            "research" => format!("research prototype - {} license", group.normalized_license),
            "migration" => format!("migration in progress - {} license", group.normalized_license),
            "legacy" => format!("legacy compatibility - {} license", group.normalized_license),
            _ => reason.to_string(),
        }
    };
    
    // Get expiration date
    let today = Utc::now().date_naive();
    let default_expiry = today + chrono::Duration::days(90);
    
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
    
    Ok(Some((reason, expires)))
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

/// Handle interactive exception processing for violations using license grouping
pub fn handle_interactive_exceptions(violations: ViolationSummary) -> Result<ViolationSummary> {
    if violations.details.is_empty() {
        return Ok(violations);
    }

    let mut exceptions = load_exceptions()?;
    let mut exceptions_added = 0;
    let mut remaining_violations = Vec::new();
    
    // Filter out already excepted violations
    let unexcepted_violations: Vec<_> = violations.details.into_iter()
        .filter(|v| !exceptions.is_excepted(&v.package_name, v.package_version.as_deref()))
        .collect();
    
    if unexcepted_violations.is_empty() {
        return Ok(ViolationSummary {
            total: 0,
            errors: 0,
            warnings: 0,
            details: Vec::new(),
        });
    }
    
    // Group violations by license
    let groups = group_violations_by_license(unexcepted_violations);
    
    println!("\n📋 Found {} license groups with violations", groups.len());
    
    for (i, group) in groups.iter().enumerate() {
        match prompt_for_group_exception(group, i + 1, groups.len()) {
            Ok(Some((reason, expires))) => {
                // Add exceptions for all packages in this group
                for violation in &group.packages {
                    let exception = Exception {
                        name: violation.package_name.clone(),
                        version: violation.package_version.clone(),
                        reason: reason.clone(),
                        added_by: std::env::var("USER").ok().or_else(|| std::env::var("USERNAME").ok()),
                        added_date: Utc::now(),
                        expires,
                        permanent: expires.is_none(),
                        added_interactively: true,
                    };
                    exceptions.add_exception(exception);
                    exceptions_added += 1;
                }
            }
            Ok(None) => {
                // Skip this group - add violations to remaining
                remaining_violations.extend(group.packages.clone());
            }
            Err(_) => {
                // User quit - add all remaining violations
                for group in &groups[i..] {
                    remaining_violations.extend(group.packages.clone());
                }
                break;
            }
        }
    }
    
    // Save exceptions if any were added
    if exceptions_added > 0 {
        save_exceptions(&exceptions)?;
        eprintln!("✅ Added {} exceptions to .exceptions.toml", exceptions_added);
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
