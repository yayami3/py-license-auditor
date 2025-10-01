use anyhow::Result;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub enum InitPreset {
    Green,
    Yellow,
    Red,
}

pub fn generate_config(preset: InitPreset) -> Result<()> {
    generate_config_at_path("pyproject.toml", preset)
}

pub fn generate_config_at_path<P: AsRef<Path>>(path: P, preset: InitPreset) -> Result<()> {
    let pyproject_path = path.as_ref();
    
    if !pyproject_path.exists() {
        return Err(anyhow::anyhow!(
            "pyproject.toml not found. Please run 'uv init' first to create a project."
        ));
    }
    
    add_license_config_to_existing(pyproject_path, preset)?;
    println!("✅ Added [tool.py-license-auditor] section to pyproject.toml");
    
    Ok(())
}

fn add_license_config_to_existing<P: AsRef<Path>>(path: P, preset: InitPreset) -> Result<()> {
    let config_content = get_preset_config(preset);
    let existing_content = fs::read_to_string(&path)?;
    
    // Parse existing TOML
    let mut doc = existing_content.parse::<toml_edit::DocumentMut>()?;
    
    // Parse embedded config to extract tool section
    let embedded_doc: toml::Value = toml::from_str(config_content)?;
    let tool_section = embedded_doc
        .get("tool")
        .and_then(|t| t.get("py-license-auditor"))
        .ok_or_else(|| anyhow::anyhow!("Invalid preset config format"))?;
    
    // Ensure tool table exists
    if !doc.contains_key("tool") {
        doc["tool"] = toml_edit::Item::Table(toml_edit::Table::new());
    }
    
    // Convert and add license config
    let tool_item = toml_value_to_edit_item(tool_section)?;
    if let Some(tool_table) = doc["tool"].as_table_mut() {
        tool_table["py-license-auditor"] = tool_item;
    }
    
    fs::write(&path, doc.to_string())?;
    Ok(())
}

fn get_preset_config(preset: InitPreset) -> &'static str {
    match preset {
        InitPreset::Red => include_str!("../examples/red.toml"),
        InitPreset::Green => include_str!("../examples/green.toml"),
        InitPreset::Yellow => include_str!("../examples/yellow.toml"),
    }
}

fn toml_value_to_edit_item(value: &toml::Value) -> Result<toml_edit::Item> {
    match value {
        toml::Value::String(s) => Ok(toml_edit::value(s.as_str())),
        toml::Value::Integer(i) => Ok(toml_edit::value(*i)),
        toml::Value::Float(f) => Ok(toml_edit::value(*f)),
        toml::Value::Boolean(b) => Ok(toml_edit::value(*b)),
        toml::Value::Array(arr) => {
            let mut edit_arr = toml_edit::Array::new();
            for item in arr {
                match item {
                    toml::Value::String(s) => edit_arr.push(s.as_str()),
                    toml::Value::Table(table) => {
                        // Handle array of tables (like exceptions)
                        let mut inline_table = toml_edit::InlineTable::new();
                        for (key, val) in table {
                            match val {
                                toml::Value::String(s) => {
                                    inline_table.insert(key, s.as_str().into());
                                }
                                _ => return Err(anyhow::anyhow!("Unsupported table value type in array")),
                            }
                        }
                        edit_arr.push(toml_edit::Value::InlineTable(inline_table));
                    }
                    _ => return Err(anyhow::anyhow!("Unsupported array item type: {:?}", item)),
                }
            }
            Ok(toml_edit::Item::Value(edit_arr.into()))
        }
        toml::Value::Table(table) => {
            let mut edit_table = toml_edit::Table::new();
            for (key, val) in table {
                edit_table[key] = toml_value_to_edit_item(val)?;
            }
            Ok(toml_edit::Item::Table(edit_table))
        }
        _ => Err(anyhow::anyhow!("Unsupported TOML value type")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_add_config_to_existing_file() -> Result<()> {
        let temp_dir = TempDir::new()?;
        
        // Create existing pyproject.toml (uv-style)
        let pyproject_path = temp_dir.path().join("pyproject.toml");
        let existing_content = r#"
[project]
name = "test-project"
version = "0.1.0"
dependencies = []

[build-system]
requires = ["hatchling"]
build-backend = "hatchling.build"
"#;
        fs::write(&pyproject_path, existing_content)?;
        
        generate_config_at_path(&pyproject_path, InitPreset::Green)?;
        
        let content = fs::read_to_string(&pyproject_path)?;
        assert!(content.contains("name = \"test-project\""));  // Existing content preserved
        assert!(content.contains("tool.py-license-auditor"));  // New section added
        assert!(content.contains("Green License Policy"));
        
        Ok(())
    }

    #[test]
    fn test_error_when_no_pyproject_toml() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let pyproject_path = temp_dir.path().join("pyproject.toml");
        
        let result = generate_config_at_path(&pyproject_path, InitPreset::Red);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("uv init"));
        
        Ok(())
    }

    #[test]
    fn test_all_presets() -> Result<()> {
        // Test each preset in separate temp directories
        let presets = [
            (InitPreset::Red, "Red License Policy"),
            (InitPreset::Green, "Green License Policy"), 
            (InitPreset::Yellow, "Yellow License Policy"),
        ];
        
        for (preset, expected_policy) in presets {
            let temp_dir = TempDir::new()?;
            let pyproject_path = temp_dir.path().join("pyproject.toml");
            
            // Create pyproject.toml
            fs::write(&pyproject_path, "[project]\nname = \"test\"")?;
            
            let result = generate_config_at_path(&pyproject_path, preset);
            assert!(result.is_ok());
            
            let content = fs::read_to_string(&pyproject_path)?;
            assert!(content.contains("tool.py-license-auditor"));
            assert!(content.contains(expected_policy));
        }
        
        Ok(())
    }

    #[test]
    fn test_config_consistency() {
        // Test that embedded configs are valid TOML
        let red_config = include_str!("../examples/red.toml");
        let green_config = include_str!("../examples/green.toml");
        let yellow_config = include_str!("../examples/yellow.toml");
        
        assert!(toml::from_str::<toml::Value>(red_config).is_ok());
        assert!(toml::from_str::<toml::Value>(green_config).is_ok());
        assert!(toml::from_str::<toml::Value>(yellow_config).is_ok());
    }
}
