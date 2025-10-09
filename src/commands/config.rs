use anyhow::Result;

pub fn handle_config(show: bool, validate: bool, quiet: bool) -> Result<()> {
    if !show && !validate {
        if !quiet {
            eprintln!("Use --show or --validate");
        }
        std::process::exit(1);
    }
    
    if show {
        match py_license_auditor::config::load_config() {
            Ok(config) => {
                if !quiet {
                    println!("{}", serde_json::to_string_pretty(&config)?);
                }
            }
            Err(e) => {
                if !quiet {
                    eprintln!("Error loading configuration: {}", e);
                }
                std::process::exit(1);
            }
        }
    }
    
    if validate {
        match py_license_auditor::config::load_config() {
            Ok(_) => {
                if !quiet {
                    println!("✅ Configuration is valid");
                }
            }
            Err(e) => {
                if !quiet {
                    eprintln!("❌ Configuration validation failed: {}", e);
                }
                std::process::exit(1);
            }
        }
    }
    
    Ok(())
}
