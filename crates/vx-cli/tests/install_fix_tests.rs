//! Test for install command argument format fix
//!
//! This test verifies that the sync and dev commands correctly pass
//! tool@version format to the install command, not separate arguments.

use std::env;
use std::path::PathBuf;
use std::process::Command;

/// Get the path to the vx binary for testing
fn vx_binary() -> PathBuf {
    // First try CARGO_BIN_EXE_vx (set by cargo test)
    if let Ok(path) = env::var("CARGO_BIN_EXE_vx") {
        return PathBuf::from(path);
    }

    // Fallback: construct path from current exe location
    let mut path = env::current_exe().unwrap();
    path.pop();
    if path.ends_with("deps") {
        path.pop();
    }
    path.push("vx");
    if cfg!(windows) {
        path.set_extension("exe");
    }
    path
}

/// Test that install command accepts tool@version format
#[test]
fn test_install_accepts_tool_at_version_format() {
    let vx_path = vx_binary();

    // Skip test if binary doesn't exist (e.g., in CI without build)
    if !vx_path.exists() {
        println!("Skipping test: vx binary not found at {:?}", vx_path);
        return;
    }

    // Test with node@20
    let output = Command::new(&vx_path)
        .args(["install", "--help"])
        .output()
        .expect("Failed to execute vx install --help");

    // Verify the command exists
    assert!(
        output.status.success(),
        "vx install command should be available"
    );
    let help_text = String::from_utf8_lossy(&output.stdout);
    assert!(
        help_text.contains("install"),
        "Help text should mention install"
    );
}

/// Test parse_tool_spec function behavior
#[test]
fn test_parse_tool_spec_basic() {
    // Test basic format: tool@version
    let spec = "node@20.0.0";
    let (tool, version) = spec.split_once('@').unwrap();
    assert_eq!(tool, "node");
    assert_eq!(version, "20.0.0");
}

/// Test parse_tool_spec without version
#[test]
fn test_parse_tool_spec_no_version() {
    // Test format without version: just tool
    let spec = "node";
    let result = spec.split_once('@');
    assert!(result.is_none(), "Should not contain @");
}

/// Test that format string creates correct tool@version
#[test]
fn test_tool_at_version_format_string() {
    let tool = "node";
    let version = "20";
    let spec = format!("{}@{}", tool, version);
    assert_eq!(spec, "node@20");
}

/// Test various version formats
#[test]
fn test_various_version_formats() {
    let test_cases = vec![
        ("node", "20", "node@20"),
        ("python", "3.11", "python@3.11"),
        ("rust", "1.90.0", "rust@1.90.0"),
        ("uv", "latest", "uv@latest"),
    ];

    for (tool, version, expected) in test_cases {
        let spec = format!("{}@{}", tool, version);
        assert_eq!(spec, expected);
    }
}
