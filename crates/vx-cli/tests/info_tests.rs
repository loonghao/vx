//! Tests for the info command and system tools discovery

use rstest::rstest;

mod common;

/// Test that info command returns valid JSON
#[rstest]
#[tokio::test]
async fn test_info_json_output() {
    common::init_test_env();
    
    // Parse JSON output
    let output = run_vx_command(&["info", "--json"]).await;
    assert!(output.status.success(), "info --json should succeed");

    let json: serde_json::Value = serde_json::from_str(&output.stdout)
        .expect("info --json should return valid JSON");

    // Check required fields
    assert!(json.get("version").is_some(), "should have version field");
    assert!(json.get("platform").is_some(), "should have platform field");
    assert!(json.get("runtimes").is_some(), "should have runtimes field");
    assert!(
        json.get("system_tools").is_some(),
        "should have system_tools field"
    );
    assert!(json.get("features").is_some(), "should have features field");
}

/// Test that info includes platform information
#[rstest]
#[tokio::test]
async fn test_info_platform_info() {
    common::init_test_env();
    
    let output = run_vx_command(&["info", "--json"]).await;
    assert!(output.status.success());

    let json: serde_json::Value = serde_json::from_str(&output.stdout).unwrap();

    let platform = json.get("platform").expect("should have platform");
    assert!(platform.get("os").is_some(), "should have os field");
    assert!(platform.get("arch").is_some(), "should have arch field");

    // Verify platform matches current system
    let os = platform.get("os").unwrap().as_str().unwrap();
    assert!(
        ["windows", "linux", "macos"].contains(&os),
        "os should be a valid platform"
    );
}

/// Test that info includes feature flags
#[rstest]
#[tokio::test]
async fn test_info_features() {
    common::init_test_env();
    
    let output = run_vx_command(&["info", "--json"]).await;
    assert!(output.status.success());

    let json: serde_json::Value = serde_json::from_str(&output.stdout).unwrap();

    let features = json.get("features").expect("should have features");
    assert!(
        features.get("auto_install").is_some(),
        "should have auto_install feature"
    );
    assert!(
        features.get("shell_mode").is_some(),
        "should have shell_mode feature"
    );
    assert!(
        features.get("project_config").is_some(),
        "should have project_config feature"
    );
}

/// Test that info text output works
#[rstest]
#[tokio::test]
async fn test_info_text_output() {
    common::init_test_env();
    
    let output = run_vx_command(&["info"]).await;
    assert!(output.status.success(), "info should succeed");

    // Check for expected content
    assert!(
        output.stdout.contains("capabilities") || output.stdout.contains("Platform"),
        "should show info"
    );
}

/// Test list --system command
#[rstest]
#[tokio::test]
async fn test_list_system_tools() {
    common::init_test_env();
    
    let output = run_vx_command(&["list", "--system"]).await;
    assert!(output.status.success(), "list --system should succeed");

    // Check for expected content
    assert!(
        output.stdout.contains("System Tools"),
        "should show system tools header"
    );
    assert!(
        output.stdout.contains("Summary"),
        "should show summary"
    );
}

/// Test list --system --all shows unavailable tools
#[rstest]
#[tokio::test]
async fn test_list_system_tools_all() {
    common::init_test_env();
    
    let output = run_vx_command(&["list", "--system", "--all"]).await;
    assert!(output.status.success(), "list --system --all should succeed");

    // Check for expected content
    assert!(
        output.stdout.contains("System Tools"),
        "should show system tools header"
    );
}

/// Helper to run vx command
async fn run_vx_command(args: &[&str]) -> CommandOutput {
    let output = common::run_vx(args).expect("Failed to execute vx");

    CommandOutput {
        status: output.status,
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
    }
}

struct CommandOutput {
    status: std::process::ExitStatus,
    stdout: String,
    #[allow(dead_code)]
    stderr: String,
}
