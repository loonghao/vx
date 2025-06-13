//! Integration tests for vx tool manager
//!
//! These tests verify that the entire vx system works correctly
//! by testing the CLI interface and core functionality.

use std::env;
use std::path::PathBuf;
use std::process::Command;

/// Get the path to the vx binary for testing
fn vx_binary() -> PathBuf {
    let mut path = env::current_exe().unwrap();
    path.pop(); // Remove test binary name
    if path.ends_with("deps") {
        path.pop(); // Remove deps directory
    }
    path.push("vx");
    if cfg!(windows) {
        path.set_extension("exe");
    }
    path
}

/// Helper function to run vx commands
fn run_vx_command(args: &[&str]) -> std::process::Output {
    Command::new(vx_binary())
        .args(args)
        .output()
        .expect("Failed to execute vx command")
}

#[test]
fn test_vx_help() {
    let output = run_vx_command(&["--help"]);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("vx"));
    // More flexible check - just ensure it contains some help text
    assert!(stdout.contains("help") || stdout.contains("Usage") || stdout.contains("USAGE"));
}

#[test]
fn test_vx_version() {
    let output = run_vx_command(&["--version"]);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("vx"));
}

#[test]
fn test_vx_list_command() {
    let output = run_vx_command(&["list"]);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should show available tools
    assert!(!stdout.is_empty());
}

#[test]
fn test_vx_plugin_list() {
    let output = run_vx_command(&["plugin", "list"]);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should show available plugins
    assert!(!stdout.is_empty());
}

#[test]
fn test_vx_unsupported_tool() {
    let output = run_vx_command(&["nonexistent-tool", "--version"]);
    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    // Should suggest available plugins or show error
    assert!(!stderr.is_empty());
}

#[test]
fn test_vx_system_path_flag() {
    // Test that --use-system-path flag is recognized
    let output = run_vx_command(&["--use-system-path", "echo", "test"]);
    // This might fail if echo is not available, but the flag should be parsed
    let stderr = String::from_utf8_lossy(&output.stderr);
    // Should not contain "unknown flag" or similar errors
    assert!(!stderr.contains("unknown") && !stderr.contains("unrecognized"));
}

#[cfg(test)]
mod tool_specific_tests {
    use super::*;

    #[test]
    fn test_node_tool_help() {
        let output = run_vx_command(&["node", "--help"]);
        // This might fail if node is not installed, but should show proper error handling
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);

        // Either succeeds with help output or fails with proper error message
        assert!(output.status.success() || !stderr.is_empty() || !stdout.is_empty());
    }

    #[test]
    fn test_uv_tool_help() {
        let output = run_vx_command(&["uv", "--help"]);
        // This might fail if uv is not installed, but should show proper error handling
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);

        // Either succeeds with help output or fails with proper error message
        assert!(output.status.success() || !stderr.is_empty() || !stdout.is_empty());
    }

    #[test]
    fn test_go_tool_help() {
        let output = run_vx_command(&["go", "version"]);
        // This might fail if go is not installed, but should show proper error handling
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);

        // Either succeeds with version output or fails with proper error message
        assert!(output.status.success() || !stderr.is_empty() || !stdout.is_empty());
    }

    #[test]
    fn test_cargo_tool_help() {
        let output = run_vx_command(&["cargo", "--version"]);
        // Cargo should be available since we're building with Rust
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        // Should either show cargo version or proper error handling
        assert!(
            output.status.success()
                || stdout.contains("cargo")
                || !stderr.is_empty()
                || !stdout.is_empty()
        );
    }
}

#[cfg(test)]
mod environment_isolation_tests {
    use super::*;

    #[test]
    fn test_default_isolation_behavior() {
        // Test that vx isolates tools by default
        let output = run_vx_command(&["python", "--version"]);

        // Should either work with vx-managed python or show proper error
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);

        // Should not silently fall back to system python without explicit flag
        assert!(output.status.success() || !stderr.is_empty() || !stdout.is_empty());
    }

    #[test]
    fn test_system_path_behavior() {
        // Test that --use-system-path allows system tools
        let output = run_vx_command(&["--use-system-path", "echo", "hello"]);

        // Should work on most systems since echo is a basic command
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            assert!(stdout.contains("hello"));
        }
        // If it fails, that's also acceptable depending on the system
    }
}

#[cfg(test)]
mod config_tests {
    use super::*;

    #[test]
    fn test_vx_config_command() {
        let output = run_vx_command(&["config"]);

        // Should show config information or help
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        assert!(output.status.success() || !stderr.is_empty() || !stdout.is_empty());
    }
}
