use std::process::Command;
use std::fs;

#[test]
#[ignore = "Requires actual uv environment with installed packages in site-packages"]
fn test_with_preconfigured_project() {
    let binary_path = env!("CARGO_BIN_EXE_py-license-auditor");
    
    // Create temp directory with minimal test setup
    let temp_dir = tempfile::tempdir().unwrap();
    let project_path = temp_dir.path().join("test-project");
    fs::create_dir(&project_path).unwrap();
    
    // Create minimal pyproject.toml
    fs::write(
        project_path.join("pyproject.toml"),
        r#"[project]
name = "test-project"
version = "0.1.0"

[tool.py-license-auditor]
format = "json"
include_unknown = true
check_violations = false
fail_on_violations = false

[tool.py-license-auditor.policy]
name = "Test Policy"
description = "Simple test policy"

[tool.py-license-auditor.policy.allowed_licenses]
exact = ["MIT", "Apache-2.0"]
patterns = []

[tool.py-license-auditor.policy.forbidden_licenses]
exact = []
patterns = []

[tool.py-license-auditor.policy.review_required]
exact = []
patterns = []
"#
    ).unwrap();
    
    // Create empty uv.lock to avoid site-packages scanning
    fs::write(
        project_path.join("uv.lock"),
        r#"version = 1
requires-python = ">=3.8"

[[package]]
name = "test-project"
version = "0.1.0"
source = { virtual = "." }
"#
    ).unwrap();
    
    // Run the auditor
    let output = Command::new(binary_path)
        .current_dir(&project_path)
        .output()
        .expect("Failed to run py-license-auditor");
    
    // Should succeed even with minimal setup
    if !output.status.success() {
        eprintln!("STDOUT: {}", String::from_utf8_lossy(&output.stdout));
        eprintln!("STDERR: {}", String::from_utf8_lossy(&output.stderr));
    }
    
    assert!(output.status.success());
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("packages") || stdout.contains("License Summary"));
}
