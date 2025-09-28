use crate::license::{LicenseReport, PackageLicense};
use crate::policy::ViolationLevel;

pub fn format_table_output(report: &LicenseReport, verbose: bool) -> String {
    let mut output = String::new();
    
    // Summary header
    let total = report.summary.total_packages;
    let with_license = report.summary.with_license;
    let without_license = report.summary.without_license;
    let violations = report.violations.as_ref().map(|v| v.total).unwrap_or(0);
    
    output.push_str(&format!("📦 License Summary ({} packages)\n", total));
    output.push_str(&format!("✅ {} with licenses  ⚠️ {} unknown  🚫 {} violations\n\n", 
                             with_license, without_license, violations));
    
    if verbose {
        // Show all packages
        output.push_str("📦 All Packages:\n");
        output.push_str(&format_package_table(&report.packages, true));
    } else {
        // Show only issues
        let issues = get_issue_packages(report);
        if !issues.is_empty() {
            output.push_str("⚠️  Issues Found:\n");
            output.push_str(&format_issue_table(&issues));
        } else {
            output.push_str("✅ No issues found!\n");
        }
        
        if !verbose && report.packages.len() > issues.len() {
            output.push_str(&format!("\n💡 Run with --verbose to see all {} packages\n", 
                                   report.packages.len()));
        }
    }
    
    output
}

fn format_package_table(packages: &[PackageLicense], show_status: bool) -> String {
    if packages.is_empty() {
        return "No packages found.\n".to_string();
    }
    
    let mut output = String::new();
    
    // Table header
    if show_status {
        output.push_str("┌─────────────────┬─────────┬─────────────┬────────┐\n");
        output.push_str("│ Package         │ Version │ License     │ Status │\n");
        output.push_str("├─────────────────┼─────────┼─────────────┼────────┤\n");
    } else {
        output.push_str("┌─────────────────┬─────────┬─────────────┬─────────────────────┐\n");
        output.push_str("│ Package         │ Version │ License     │ Issue               │\n");
        output.push_str("├─────────────────┼─────────┼─────────────┼─────────────────────┤\n");
    }
    
    // Table rows
    for package in packages {
        let name = truncate(&package.name, 15);
        let version = truncate(package.version.as_deref().unwrap_or("unknown"), 7);
        let license = package.license.as_deref().unwrap_or("(unknown)");
        let license = truncate(license, 11);
        
        if show_status {
            let status = if package.license.is_some() { "✅ OK" } else { "⚠️ Issue" };
            output.push_str(&format!("│ {:<15} │ {:<7} │ {:<11} │ {:<6} │\n", 
                                   name, version, license, status));
        } else {
            let issue = if package.license.is_none() { "No license info" } else { "Requires review" };
            let issue = truncate(issue, 19);
            output.push_str(&format!("│ {:<15} │ {:<7} │ {:<11} │ {:<19} │\n", 
                                   name, version, license, issue));
        }
    }
    
    // Table footer
    if show_status {
        output.push_str("└─────────────────┴─────────┴─────────────┴────────┘\n");
    } else {
        output.push_str("└─────────────────┴─────────┴─────────────┴─────────────────────┘\n");
    }
    
    output
}

fn format_issue_table(issues: &[(PackageLicense, String)]) -> String {
    if issues.is_empty() {
        return "No issues found.\n".to_string();
    }
    
    let mut output = String::new();
    
    // Table header
    output.push_str("┌─────────────────┬─────────┬─────────────┬─────────────────────┐\n");
    output.push_str("│ Package         │ Version │ License     │ Issue               │\n");
    output.push_str("├─────────────────┼─────────┼─────────────┼─────────────────────┤\n");
    
    // Table rows
    for (package, issue) in issues {
        let name = truncate(&package.name, 15);
        let version = truncate(package.version.as_deref().unwrap_or("unknown"), 7);
        let license = package.license.as_deref().unwrap_or("(unknown)");
        let license = truncate(license, 11);
        let issue = truncate(issue, 19);
        
        output.push_str(&format!("│ {:<15} │ {:<7} │ {:<11} │ {:<19} │\n", 
                               name, version, license, issue));
    }
    
    // Table footer
    output.push_str("└─────────────────┴─────────┴─────────────┴─────────────────────┘\n");
    
    output
}

fn get_issue_packages(report: &LicenseReport) -> Vec<(PackageLicense, String)> {
    let mut issues = Vec::new();
    
    // Add packages without license
    for package in &report.packages {
        if package.license.is_none() {
            issues.push((package.clone(), "No license info".to_string()));
        }
    }
    
    // Add violation packages
    if let Some(violations) = &report.violations {
        for violation in &violations.details {
            if let Some(package) = report.packages.iter()
                .find(|p| p.name == violation.package_name && 
                         p.version.as_deref() == violation.package_version.as_deref()) {
                let issue = match violation.violation_level {
                    ViolationLevel::Forbidden => "Forbidden license".to_string(),
                    ViolationLevel::ReviewRequired => "Requires review".to_string(),
                    ViolationLevel::Unknown => "License issue".to_string(),
                    ViolationLevel::Allowed => continue, // Skip allowed licenses
                };
                issues.push((package.clone(), issue));
            }
        }
    }
    
    // Remove duplicates
    issues.sort_by(|a, b| a.0.name.cmp(&b.0.name));
    issues.dedup_by(|a, b| a.0.name == b.0.name && a.0.version == b.0.version);
    
    issues
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}…", &s[..max_len-1])
    }
}
