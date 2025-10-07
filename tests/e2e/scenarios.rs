use super::helpers::TestProject;

#[test]
fn test_basic_license_extraction() {
    let test_env = TestProject::new();
    
    // Setup test project
    test_env.init_uv_project("test-app", &["requests", "click"]).unwrap();
    
    // Run license extraction with check subcommand
    let output = test_env.run_auditor("test-app", &["check", "--format", "json"]);
    
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("requests"));
    assert!(String::from_utf8_lossy(&output.stdout).contains("click"));
}

#[test]
fn test_policy_initialization_and_checking() {
    let test_env = TestProject::new();
    
    // Setup test project
    test_env.init_uv_project("policy-test", &["requests"]).unwrap();
    
    // Initialize red policy (fail_on_violations = false) with init subcommand
    let init_output = test_env.run_auditor("policy-test", &["init", "red"]);
    assert!(init_output.status.success());
    
    // Run policy check with check subcommand
    let check_output = test_env.run_auditor("policy-test", &["check"]);
    assert!(check_output.status.success());
    
    // Should contain violations section in JSON output
    let stdout = String::from_utf8_lossy(&check_output.stdout);
    assert!(stdout.contains("violations"));
}

#[test]
fn test_different_output_formats() {
    let test_env = TestProject::new();
    
    test_env.init_uv_project("format-test", &["click"]).unwrap();
    
    // Test JSON format with check subcommand
    let json_output = test_env.run_auditor("format-test", &["check", "--format", "json"]);
    assert!(json_output.status.success());
    assert!(String::from_utf8_lossy(&json_output.stdout).contains("packages"));
    
    // Test table format with check subcommand
    let table_output = test_env.run_auditor("format-test", &["check", "--format", "table"]);
    assert!(table_output.status.success());
    assert!(String::from_utf8_lossy(&table_output.stdout).contains("License Summary"));
}

#[test]
fn test_policy_violation_detection() {
    let test_env = TestProject::new();
    
    test_env.init_uv_project("violation-test", &["requests", "pandas"]).unwrap();
    
    // Initialize strict CI policy with init subcommand
    test_env.run_auditor("violation-test", &["init", "yellow"]);
    
    // Run check - should find violations
    let output = test_env.run_auditor("violation-test", &["check"]);
    
    // Check stderr for violation message
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("violations found") || stderr.contains("License violations"));
}

#[test]
fn test_fix_subcommand() {
    let test_env = TestProject::new();
    
    test_env.init_uv_project("fix-test", &["requests"]).unwrap();
    
    // Initialize green policy (strict)
    let init_output = test_env.run_auditor("fix-test", &["init", "green"]);
    assert!(init_output.status.success());
    
    // Test dry-run mode
    let dry_run_output = test_env.run_auditor("fix-test", &["fix", "--dry-run"]);
    assert!(dry_run_output.status.success());
    let stdout = String::from_utf8_lossy(&dry_run_output.stdout);
    assert!(stdout.contains("Would add") || stdout.contains("No violations"));
}

#[test]
fn test_global_options() {
    let test_env = TestProject::new();
    
    test_env.init_uv_project("global-test", &["click"]).unwrap();
    
    // Test quiet mode
    let quiet_output = test_env.run_auditor("global-test", &["--quiet", "init", "green"]);
    assert!(quiet_output.status.success());
    
    // Test config validation
    let validate_output = test_env.run_auditor("global-test", &["config", "--validate"]);
    assert!(validate_output.status.success());
}
