use anyhow::Result;
use clap::{Parser, ValueEnum};
use std::fs;
use std::path::PathBuf;

// Import from our library
use py_license_auditor::license::{extract_licenses_auto, create_report};
use py_license_auditor::output::format_table_output;
use py_license_auditor::exceptions::handle_interactive_exceptions;
use py_license_auditor::config::load_config;
use py_license_auditor::init;


#[derive(Parser)]
#[command(name = "py-license-auditor")]
#[command(about = "Extract license information from Python packages")]
#[command(version)]
struct Cli {
    /// Path to site-packages directory or virtual environment
    #[arg(short, long)]
    path: Option<PathBuf>,

    /// Output format
    #[arg(short, long)]
    format: Option<OutputFormat>,

    /// Output file (default: stdout)
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Show all packages (default: issues only)
    #[arg(long)]
    verbose: bool,

    /// Include packages without license information
    #[arg(long)]
    include_unknown: bool,

    /// Interactive mode for handling violations
    #[arg(long)]
    interactive: bool,

    /// Initialize configuration file with preset
    #[arg(long)]
    init: Option<InitPreset>,
}

#[derive(Clone, ValueEnum)]
enum OutputFormat {
    Table,
    Json,
    Csv,
}

#[derive(Clone, ValueEnum)]
enum InitPreset {
    Personal,
    Corporate,
    Ci,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Handle init command first
    if let Some(preset) = cli.init {
        let init_preset = match preset {
            InitPreset::Personal => init::InitPreset::Personal,
            InitPreset::Corporate => init::InitPreset::Corporate,
            InitPreset::Ci => init::InitPreset::Ci,
        };
        return init::generate_config(init_preset);
    }
    
    // Load configuration from pyproject.toml
    let config = load_config()?;
    
    // CLI arguments override config values
    let include_unknown = cli.include_unknown || config.include_unknown.unwrap_or(false);

    // Auto-detect uv.lock or fallback to site-packages
    let packages = extract_licenses_auto(cli.path, include_unknown)?;
    
    let mut report = create_report(packages);
    
    // Policy checking (if configured)
    if let Some(policy) = &config.policy {
        if config.check_violations.unwrap_or(false) {
            let mut violations = policy.detect_violations(&report.packages);
            
            // Interactive mode for exception handling
            if cli.interactive {
                violations = handle_interactive_exceptions(violations)?;
            }
            
            // Handle violations
            if violations.total > 0 {
                eprintln!("License violations found: {} total ({} errors, {} warnings)", 
                         violations.total, violations.errors, violations.warnings);
                
                if config.fail_on_violations.unwrap_or(false) && violations.errors > 0 {
                    eprintln!("Exiting with error due to forbidden licenses");
                    std::process::exit(1);
                }
            }
            
            report.violations = Some(violations);
        }
    }

    // Determine output format
    let format = cli.format.unwrap_or_else(|| {
        match config.format.as_deref() {
            Some("json") => OutputFormat::Json,
            Some("csv") => OutputFormat::Csv,
            _ => OutputFormat::Table,
        }
    });
    
    // Generate output
    let output = match format {
        OutputFormat::Json => serde_json::to_string_pretty(&report)?,
        OutputFormat::Table => format_table_output(&report, cli.verbose),
        OutputFormat::Csv => "CSV not implemented yet".to_string(),
    };

    match cli.output {
        Some(path) => fs::write(path, output)?,
        None => println!("{}", output),
    }

    Ok(())
}
