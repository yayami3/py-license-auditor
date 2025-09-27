use std::fs;
use std::path::{Path, PathBuf};
use anyhow::{Context, Result};
use super::models::ExceptionsFile;

impl ExceptionsFile {
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read exceptions file: {}", path.as_ref().display()))?;
        
        let exceptions: ExceptionsFile = toml::from_str(&content)
            .with_context(|| format!("Failed to parse exceptions file: {}", path.as_ref().display()))?;
        
        Ok(exceptions)
    }

    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content = toml::to_string_pretty(self)
            .context("Failed to serialize exceptions")?;
        
        fs::write(&path, content)
            .with_context(|| format!("Failed to write exceptions file: {}", path.as_ref().display()))?;
        
        Ok(())
    }
}

pub fn get_exceptions_file_path() -> PathBuf {
    std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join(".exceptions.toml")
}

pub fn load_exceptions() -> Result<ExceptionsFile> {
    let path = get_exceptions_file_path();
    if path.exists() {
        ExceptionsFile::load_from_file(path)
    } else {
        Ok(ExceptionsFile::new())
    }
}

pub fn save_exceptions(exceptions: &ExceptionsFile) -> Result<()> {
    let path = get_exceptions_file_path();
    exceptions.save_to_file(path)
}
