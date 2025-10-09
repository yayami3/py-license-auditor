use anyhow::Result;
use crate::cli::InitPreset;
use py_license_auditor::init;

pub fn handle_init(policy: InitPreset, quiet: bool) -> Result<()> {
    let init_preset = match policy {
        InitPreset::Green => init::InitPreset::Green,
        InitPreset::Yellow => init::InitPreset::Yellow,
        InitPreset::Red => init::InitPreset::Red,
    };
    
    let result = init::generate_config(init_preset);
    
    if result.is_ok() && !quiet {
        println!("✅ Configuration initialized successfully");
    }
    
    result
}
