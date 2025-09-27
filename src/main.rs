use anyhow::Result;
use clap::{Parser, ValueEnum};
use std::fs;
use std::path::PathBuf;

// Import from our library
use py_license_auditor::license::{extract_licenses_auto, find_site_packages_path, create_report};

use py_license_auditor::exceptions::handle_interactive_exceptions;
use py_license_auditor::config::{BuiltinPolicy, load_config};


#[derive(Parser)]
#[command(name = "py-license-auditor")]
#[command(about = "Extract license information from Python packages")]
#[command(version)]
struct Cli {
    /// Path to site-packages directory or virtual environment
    #[arg(short, long)]
    path: Option<PathBuf>,

    /// Output format
    #[arg(short, long, default_value = "json")]
    format: OutputFormat,

    /// Output file (default: stdout)
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Include packages without license information
    #[arg(long)]
    include_unknown: bool,

    /// Path to license policy file (TOML format)
    #[arg(long)]
    policy_file: Option<PathBuf>,

    /// Use built-in policy (corporate, permissive, strict)
    #[arg(long, conflicts_with = "policy_file")]
    policy: Option<BuiltinPolicy>,

    /// Check for license violations according to policy
    #[arg(long)]
    check_violations: bool,

    /// Exit with error code if violations are found
    #[arg(long)]
    fail_on_violations: bool,

    /// Interactive mode for handling violations
    #[arg(long)]
    interactive: bool,
}

#[derive(Clone, ValueEnum)]
enum OutputFormat {
    Json,
    Toml,
    Csv,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Load configuration from pyproject.toml
    let config = load_config()?;
    
    // CLI arguments override config values
    let include_unknown = cli.include_unknown || config.include_unknown.unwrap_or(false);

    // Auto-detect uv.lock or fallback to site-packages
    let packages = extract_licenses_auto(cli.path)?;
    
    // ポリシーチェックの実行
    let policy = if cli.check_violations || cli.policy_file.is_some() || cli.policy.is_some() || config.policy.is_some() {
        config.resolve_policy_from_cli(cli.policy_file.as_ref(), cli.policy.as_ref())?
    } else {
        None
    };
    
    if policy.is_none() && cli.check_violations {
        eprintln!("Warning: --check-violations specified but no policy provided");
    }

    let mut report = create_report(packages);
    
    // 違反検出の実行
    if let Some(policy) = &policy {
        let mut violations = policy.detect_violations(&report.packages);
        
        // インタラクティブモードで例外処理
        if cli.interactive {
            violations = handle_interactive_exceptions(violations)?;
        }
        
        // 違反があった場合の処理
        if violations.total > 0 {
            eprintln!("License violations found: {} total ({} errors, {} warnings)", 
                     violations.total, violations.errors, violations.warnings);
            
            if cli.fail_on_violations && violations.errors > 0 {
                eprintln!("Exiting with error due to forbidden licenses");
                std::process::exit(1);
            }
        }
        
        report.violations = Some(violations);
    }

    let output = match cli.format {
        OutputFormat::Json => serde_json::to_string_pretty(&report)?,
        OutputFormat::Toml => toml::to_string_pretty(&report)?,
        OutputFormat::Csv => "CSV not implemented yet".to_string(),
    };

    match cli.output {
        Some(path) => fs::write(path, output)?,
        None => println!("{}", output),
    }

    Ok(())
}
