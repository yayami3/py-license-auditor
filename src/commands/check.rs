use anyhow::Result;
use std::fs;
use std::path::PathBuf;
use crate::cli::OutputFormat;
use py_license_auditor::license::{extract_licenses_auto, create_report};
use py_license_auditor::output::format_table_output;
use py_license_auditor::config::load_config;

pub fn handle_check(
    path: Option<PathBuf>,
    format: Option<OutputFormat>,
    output: Option<PathBuf>,
    include_unknown: bool,
    quiet: bool,
    verbose: bool,
    exit_zero: bool,
) -> Result<()> {
    // Load configuration from pyproject.toml
    let config = load_config()?;
    
    // CLI arguments override config values
    let include_unknown = include_unknown || config.include_unknown.unwrap_or(false);

    // Auto-detect uv.lock or fallback to site-packages
    let packages = extract_licenses_auto(path, include_unknown)?;
    
    let mut report = create_report(packages);
    
    // Policy checking (if configured)
    if let Some(policy) = &config.policy {
        if config.check_violations.unwrap_or(false) {
            let violations = policy.detect_violations(&report.packages);
            
            // Handle violations
            if violations.total > 0 {
                if !quiet {
                    eprintln!("License violations found: {} total ({} errors, {} warnings)", 
                             violations.total, violations.errors, violations.warnings);
                }
                
                if !exit_zero && config.fail_on_violations.unwrap_or(false) && violations.errors > 0 {
                    eprintln!("Exiting with error due to forbidden licenses");
                    std::process::exit(1);
                }
            }
            
            report.violations = Some(violations);
        }
    }

    // Determine output format
    let format = format.unwrap_or_else(|| {
        match config.format.as_deref() {
            Some("json") => OutputFormat::Json,
            Some("csv") => OutputFormat::Csv,
            Some("table") => OutputFormat::Table,
            _ => OutputFormat::Table,  // Default to table instead of JSON
        }
    });
    
    // Generate output
    let output_content = match format {
        OutputFormat::Json => serde_json::to_string_pretty(&report)?,
        OutputFormat::Table => format_table_output(&report, verbose),
        OutputFormat::Csv => "CSV not implemented yet".to_string(),
    };

    match output {
        Some(path) => fs::write(path, output_content)?,
        None => {
            if !quiet {
                println!("{}", output_content);
            }
        }
    }

    Ok(())
}
