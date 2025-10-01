use crate::license::{LicenseReport, PackageLicense};
use crate::policy::ViolationLevel;

fn format_with_padding(text: &str, width: usize) -> String {
    // Calculate actual display width (excluding ANSI codes)
    let display_width = text.chars()
        .filter(|&c| c != '\x1b')
        .collect::<String>()
        .replace("[32m", "")
        .replace("[33m", "")
        .replace("[31m", "")
        .replace("[0m", "")
        .len();
    
    let padding = if display_width < width {
        " ".repeat(width - display_width)
    } else {
        String::new()
    };
    
    format!("{}{}", text, padding)
}

#[derive(Debug, Clone, PartialEq)]
enum PackageStatus {
    Ok,
    Unknown,      // No license info
    Violation,    // Policy violation
}

fn get_package_status(package: &PackageLicense, report: &LicenseReport) -> PackageStatus {
    // Check if no license info
    if package.license.is_none() {
        return PackageStatus::Unknown;
    }
    
    // Check for policy violations
    if let Some(violations) = &report.violations {
        if violations.details.iter().any(|v| 
            v.package_name == package.name && 
            v.package_version.as_deref() == package.version.as_deref() &&
            v.license.is_some()
        ) {
            return PackageStatus::Violation;
        }
    }
    
    PackageStatus::Ok
}

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
        output.push_str(&format_package_table(&report.packages, true, Some(report)));
    } else {
        // Show only issues
        let issues = get_issue_packages(report);
        if !issues.is_empty() {
            output.push_str("🔍 Issues Found:\n");
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

fn format_package_table(packages: &[PackageLicense], show_status: bool, report: Option<&LicenseReport>) -> String {
    if packages.is_empty() {
        return "No packages found.\n".to_string();
    }
    
    let mut output = String::new();
    
    // Table header
    if show_status {
        output.push_str("┌─────────────────┬─────────┬─────────────┬─────────────────┐\n");
        output.push_str("│ Package         │ Version │ License     │ Status          │\n");
        output.push_str("├─────────────────┼─────────┼─────────────┼─────────────────┤\n");
    } else {
        output.push_str("┌─────────────────┬─────────┬─────────────┬─────────────────┐\n");
        output.push_str("│ Package         │ Version │ License     │ Problem         │\n");
        output.push_str("├─────────────────┼─────────┼─────────────┼─────────────────┤\n");
    }
    
    // Table rows
    for package in packages {
        let name = truncate(&package.name, 15);
        let version = truncate(package.version.as_deref().unwrap_or("unknown"), 7);
        let license = package.license.as_deref().unwrap_or("(unknown)");
        let license = truncate(license, 11);
        
        if show_status {
            let status = match get_package_status(package, report.unwrap_or(&LicenseReport::default())) {
                PackageStatus::Ok => "\x1b[32mOK\x1b[0m",        // Green
                PackageStatus::Unknown => "\x1b[33mUnknown\x1b[0m", // Yellow
                PackageStatus::Violation => "\x1b[31mProblem\x1b[0m", // Red
            };
            let formatted_status = format_with_padding(status, 15);
            output.push_str(&format!("│ {:<15} │ {:<7} │ {:<11} │ {} │\n", 
                                   name, version, license, formatted_status));
        } else {
            let issue = if package.license.is_none() { "No license info" } else { "Requires review" };
            let issue = truncate(issue, 15);
            output.push_str(&format!("│ {:<15} │ {:<7} │ {:<11} │ {:<15} │\n", 
                                   name, version, license, issue));
        }
    }
    
    // Table footer
    if show_status {
        output.push_str("└─────────────────┴─────────┴─────────────┴─────────────────┘\n");
    } else {
        output.push_str("└─────────────────┴─────────┴─────────────┴─────────────────┘\n");
    }
    
    output
}

fn format_issue_table(issues: &[(PackageLicense, String)]) -> String {
    if issues.is_empty() {
        return "No issues found.\n".to_string();
    }
    
    let mut output = String::new();
    
    // Table header
    output.push_str("┌─────────────────┬─────────┬─────────────┬─────────────────┐\n");
    output.push_str("│ Package         │ Version │ License     │ Problem         │\n");
    output.push_str("├─────────────────┼─────────┼─────────────┼─────────────────┤\n");
    
    // Table rows
    for (package, issue) in issues {
        let name = truncate(&package.name, 15);
        let version = truncate(package.version.as_deref().unwrap_or("unknown"), 7);
        let license = package.license.as_deref().unwrap_or("(unknown)");
        let license = truncate(license, 11);
        let issue = truncate(issue, 15);
        
        output.push_str(&format!("│ {:<15} │ {:<7} │ {:<11} │ {:<15} │\n", 
                               name, version, license, issue));
    }
    
    // Table footer
    output.push_str("└─────────────────┴─────────┴─────────────┴─────────────────┘\n");
    
    output
}

fn get_issue_packages(report: &LicenseReport) -> Vec<(PackageLicense, String)> {
    let mut issues = Vec::new();
    
    for package in &report.packages {
        let status = get_package_status(package, report);
        match status {
            PackageStatus::Unknown => {
                issues.push((package.clone(), "No license info".to_string()));
            }
            PackageStatus::Violation => {
                // Find the specific violation message
                if let Some(violations) = &report.violations {
                    if let Some(violation) = violations.details.iter().find(|v| 
                        v.package_name == package.name && 
                        v.package_version.as_deref() == package.version.as_deref()
                    ) {
                        let issue = match violation.violation_level {
                            ViolationLevel::Forbidden => "Forbidden".to_string(),
                            ViolationLevel::ReviewRequired => "Review required".to_string(),
                            ViolationLevel::Unknown => "Not allowed".to_string(),
                            ViolationLevel::Allowed => continue,
                        };
                        issues.push((package.clone(), issue));
                    }
                }
            }
            PackageStatus::Ok => {} // Skip OK packages
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
