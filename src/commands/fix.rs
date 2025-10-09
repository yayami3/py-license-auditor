use anyhow::Result;
use std::path::PathBuf;
use crate::cli::OutputFormat;
use py_license_auditor::license::extract_licenses_auto;
use py_license_auditor::config::load_config;

pub fn handle_fix(
    path: Option<PathBuf>,
    dry_run: bool,
    interactive: bool,
    _format: Option<OutputFormat>,
    quiet: bool,
) -> Result<()> {
    // Load configuration
    let config = load_config()?;
    
    if config.policy.is_none() {
        if !quiet {
            eprintln!("No policy configured. Run 'py-license-auditor init <policy>' first.");
        }
        std::process::exit(1);
    }
    
    let policy = config.policy.unwrap();
    
    // Extract packages
    let include_unknown = config.include_unknown.unwrap_or(false);
    let packages = extract_licenses_auto(path, include_unknown)?;
    
    // Check for violations
    let violations = policy.detect_violations(&packages);
    
    if violations.total == 0 {
        if !quiet {
            println!("No violations found, nothing to fix");
        }
        return Ok(());
    }
    
    if interactive {
        // Use interactive license-based exception handling
        use py_license_auditor::exceptions::handle_interactive_exceptions;
        
        if dry_run {
            if !quiet {
                println!("Interactive mode with --dry-run is not supported");
                println!("Use either --interactive or --dry-run, not both");
            }
            return Ok(());
        }
        
        let remaining_violations = handle_interactive_exceptions(violations)?;
        
        if !quiet {
            if remaining_violations.total == 0 {
                println!("✅ All violations resolved with exceptions");
            } else {
                println!("⚠️  {} violations remain unresolved", remaining_violations.total);
            }
        }
        
        return Ok(());
    }
    
    // Default behavior: create exceptions from violations
    let mut exceptions = Vec::new();
    for detail in &violations.details {
        let exception = py_license_auditor::policy::PackageException {
            name: detail.package_name.clone(),
            version: detail.package_version.clone(),
            reason: format!("Auto-generated exception for {} license", 
                          detail.license.as_deref().unwrap_or("unknown")),
        };
        exceptions.push(exception);
    }
    
    if dry_run {
        if !quiet {
            println!("Would add {} exceptions to pyproject.toml:", exceptions.len());
            for exception in &exceptions {
                println!("  - {} {} ({})", exception.name, 
                        exception.version.as_deref().unwrap_or("*"), 
                        exception.reason);
            }
        }
        return Ok(());
    }
    
    // Add exceptions to pyproject.toml
    py_license_auditor::config::add_exceptions_to_config(exceptions.clone())?;
    
    if !quiet {
        println!("Added {} exceptions to pyproject.toml:", exceptions.len());
        for exception in &exceptions {
            println!("  ✅ {} {} - {}", exception.name, 
                    exception.version.as_deref().unwrap_or("*"), 
                    exception.reason);
        }
    }
    
    Ok(())
}
