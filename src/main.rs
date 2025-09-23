use anyhow::{Context, Result};
use clap::{Parser, ValueEnum};
use std::fs;
use std::path::{Path, PathBuf};

// Import from our library
use py_license_auditor::license::{extract_all_licenses, find_site_packages_path, create_report};
use py_license_auditor::policy::LicensePolicy;
use py_license_auditor::exceptions::{load_exceptions, save_exceptions, prompt_for_exception};

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

#[derive(Debug, Clone, ValueEnum)]
enum BuiltinPolicy {
    Corporate,
    Permissive,
    Strict,
}

/// ポリシーファイルを読み込む
fn load_policy(policy_path: &Path) -> Result<LicensePolicy> {
    let content = fs::read_to_string(policy_path)
        .with_context(|| format!("Failed to read policy file: {}", policy_path.display()))?;
    
    let policy: LicensePolicy = toml::from_str(&content)
        .with_context(|| format!("Failed to parse policy file: {}", policy_path.display()))?;
    
    Ok(policy)
}

/// 組み込みポリシーを読み込む
fn load_builtin_policy(policy_type: BuiltinPolicy) -> Result<LicensePolicy> {
    let content = match policy_type {
        BuiltinPolicy::Corporate => include_str!("../examples/policy-corporate.toml"),
        BuiltinPolicy::Permissive => include_str!("../examples/policy-permissive.toml"),
        BuiltinPolicy::Strict => include_str!("../examples/policy-strict.toml"),
    };
    
    let policy: LicensePolicy = toml::from_str(content)
        .with_context(|| format!("Failed to parse built-in {:?} policy", policy_type))?;
    
    Ok(policy)
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let site_packages_path = find_site_packages_path(cli.path)?;
    eprintln!("Scanning: {}", site_packages_path.display());

    let packages = extract_all_licenses(&site_packages_path, cli.include_unknown)?;
    
    // ポリシーチェックの実行
    let policy = if cli.check_violations || cli.policy_file.is_some() || cli.policy.is_some() {
        match (&cli.policy_file, &cli.policy) {
            (Some(policy_path), None) => Some(load_policy(policy_path)?),
            (None, Some(builtin_policy)) => Some(load_builtin_policy(builtin_policy.clone())?),
            (None, None) => {
                eprintln!("Warning: --check-violations specified but no --policy-file or --policy provided");
                None
            }
            (Some(_), Some(_)) => unreachable!(), // conflicts_with prevents this
        }
    } else {
        None
    };

    let mut report = create_report(packages);
    
    // 違反検出の実行
    if let Some(policy) = &policy {
        let mut exceptions = load_exceptions()?;
        let mut violations = policy.detect_violations(&report.packages);
        
        // インタラクティブモードで例外処理
        if cli.interactive && !violations.details.is_empty() {
            let mut exceptions_added = 0;
            let mut remaining_violations = Vec::new();
            
            for violation in violations.details {
                // 例外チェック（既に例外に含まれているかもしれない）
                if exceptions.is_excepted(&violation.package_name, violation.package_version.as_deref()) {
                    continue;
                }
                
                let violation_type = match violation.violation_level {
                    py_license_auditor::policy::ViolationLevel::Forbidden => "Forbidden license",
                    py_license_auditor::policy::ViolationLevel::ReviewRequired => "Review required",
                    py_license_auditor::policy::ViolationLevel::Unknown => "Unknown license",
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
            
            // 例外を保存
            if exceptions_added > 0 {
                save_exceptions(&exceptions)?;
                eprintln!("Added {} exceptions to .exceptions.toml", exceptions_added);
            }
            
            // 残りの違反で違反サマリーを更新
            let errors = remaining_violations.iter().filter(|v| v.violation_level == py_license_auditor::policy::ViolationLevel::Forbidden).count();
            let warnings = remaining_violations.iter().filter(|v| 
                v.violation_level == py_license_auditor::policy::ViolationLevel::ReviewRequired || 
                v.violation_level == py_license_auditor::policy::ViolationLevel::Unknown
            ).count();
            
            violations = py_license_auditor::policy::ViolationSummary {
                total: remaining_violations.len(),
                errors,
                warnings,
                details: remaining_violations,
            };
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
