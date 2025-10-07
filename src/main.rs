use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};
use std::fs;
use std::path::PathBuf;

// Import from our library
use py_license_auditor::license::{extract_licenses_auto, create_report};
use py_license_auditor::output::format_table_output;

use py_license_auditor::config::load_config;
use py_license_auditor::init;

#[derive(Parser)]
#[command(name = "py-license-auditor")]
#[command(about = "Extract license information from Python packages")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run license audit on packages
    Check {
        /// Path to site-packages directory or virtual environment
        path: Option<PathBuf>,

        /// Output format
        #[arg(short, long)]
        format: Option<OutputFormat>,

        /// Output file (default: stdout)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Include packages without license information
        #[arg(long)]
        include_unknown: bool,

        /// Show errors only
        #[arg(short, long)]
        quiet: bool,

        /// Show detailed information
        #[arg(short, long)]
        verbose: bool,

        /// Exit with code 0 even on violations
        #[arg(long)]
        exit_zero: bool,
    },
    /// Initialize configuration with preset policy
    Init {
        /// Policy preset
        policy: InitPreset,
    },
    /// Automatically fix violations by adding exceptions
    Fix {
        /// Path to site-packages directory or virtual environment
        path: Option<PathBuf>,

        /// Show changes without applying them
        #[arg(long)]
        dry_run: bool,

        /// Output format for changes
        #[arg(short, long)]
        format: Option<OutputFormat>,
    },
    /// Show or validate configuration
    Config {
        /// Show current configuration
        #[arg(long)]
        show: bool,

        /// Validate configuration file
        #[arg(long)]
        validate: bool,
    },
}

#[derive(Clone, ValueEnum)]
enum OutputFormat {
    Table,
    Json,
    Csv,
}

#[derive(Clone, ValueEnum)]
enum InitPreset {
    Green,
    Yellow,
    Red,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Check { 
            path, 
            format, 
            output, 
            include_unknown, 
            quiet, 
            verbose, 
            exit_zero 
        } => {
            handle_check(path, format, output, include_unknown, quiet, verbose, exit_zero)
        }
        Commands::Init { policy } => {
            handle_init(policy)
        }
        Commands::Fix { path, dry_run, format } => {
            handle_fix(path, dry_run, format)
        }
        Commands::Config { show, validate } => {
            handle_config(show, validate)
        }
    }
}

fn handle_check(
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
            _ => OutputFormat::Table,
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

fn handle_init(policy: InitPreset) -> Result<()> {
    let init_preset = match policy {
        InitPreset::Green => init::InitPreset::Green,
        InitPreset::Yellow => init::InitPreset::Yellow,
        InitPreset::Red => init::InitPreset::Red,
    };
    init::generate_config(init_preset)
}

fn handle_fix(
    path: Option<PathBuf>,
    dry_run: bool,
    _format: Option<OutputFormat>,
) -> Result<()> {
    // Load configuration
    let config = py_license_auditor::config::load_config()?;
    
    if config.policy.is_none() {
        eprintln!("No policy configured. Run 'py-license-auditor init <policy>' first.");
        std::process::exit(1);
    }
    
    let policy = config.policy.unwrap();
    
    // Extract packages
    let include_unknown = config.include_unknown.unwrap_or(false);
    let packages = extract_licenses_auto(path, include_unknown)?;
    
    // Check for violations
    let violations = policy.detect_violations(&packages);
    
    if violations.total == 0 {
        println!("No violations found, nothing to fix");
        return Ok(());
    }
    
    // Create exceptions from violations
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
        println!("Would add {} exceptions to pyproject.toml:", exceptions.len());
        for exception in &exceptions {
            println!("  - {} {} ({})", exception.name, 
                    exception.version.as_deref().unwrap_or("*"), 
                    exception.reason);
        }
        return Ok(());
    }
    
    // Add exceptions to pyproject.toml
    py_license_auditor::config::add_exceptions_to_config(exceptions.clone())?;
    
    println!("Added {} exceptions to pyproject.toml:", exceptions.len());
    for exception in &exceptions {
        println!("  ✅ {} {} - {}", exception.name, 
                exception.version.as_deref().unwrap_or("*"), 
                exception.reason);
    }
    
    Ok(())
}

fn handle_config(show: bool, validate: bool) -> Result<()> {
    if show {
        match load_config() {
            Ok(config) => {
                println!("{}", serde_json::to_string_pretty(&config)?);
            }
            Err(e) => {
                eprintln!("Error loading configuration: {}", e);
                std::process::exit(1);
            }
        }
    }
    
    if validate {
        match load_config() {
            Ok(_) => println!("Configuration is valid"),
            Err(e) => {
                eprintln!("Configuration validation failed: {}", e);
                std::process::exit(1);
            }
        }
    }
    
    if !show && !validate {
        eprintln!("Use --show or --validate");
        std::process::exit(1);
    }
    
    Ok(())
}
