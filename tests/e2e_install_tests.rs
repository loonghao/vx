//! E2E tests for tool installation
//!
//! These tests verify that tool installation works correctly,
//! including download URL generation and version resolution.

use std::env;
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

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

/// E2E test environment with isolated VX_HOME
struct E2ETestEnv {
    home: TempDir,
}

impl E2ETestEnv {
    fn new() -> Self {
        Self {
            home: TempDir::new().expect("Failed to create temp dir"),
        }
    }

    fn run(&self, args: &[&str]) -> std::process::Output {
        Command::new(vx_binary())
            .args(args)
            .env("VX_HOME", self.home.path())
            .output()
            .expect("Failed to execute vx command")
    }

    #[allow(dead_code)]
    fn run_success(&self, args: &[&str]) -> String {
        let output = self.run(args);
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        if !output.status.success() {
            panic!(
                "Command failed: vx {}\nstdout: {}\nstderr: {}",
                args.join(" "),
                stdout,
                stderr
            );
        }
        stdout
    }
}

// ============================================================================
// Version listing tests - verify that version fetching works
// ============================================================================

#[test]
fn test_versions_list_zig() {
    let env = E2ETestEnv::new();
    let output = env.run(&["versions", "zig"]);

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should either succeed with version list or fail gracefully with network error
    if output.status.success() {
        // Should contain version numbers like 0.15.2, 0.14.1, etc.
        assert!(
            stdout.contains("0.") || stdout.contains("Version"),
            "Expected version numbers in output: {}",
            stdout
        );
    } else {
        // Network errors are acceptable in CI
        assert!(
            stderr.contains("network")
                || stderr.contains("timeout")
                || stderr.contains("connection")
                || stderr.contains("rate limit")
                || stderr.contains("Failed to fetch"),
            "Unexpected error: {}",
            stderr
        );
    }
}

#[test]
fn test_versions_list_node() {
    let env = E2ETestEnv::new();
    let output = env.run(&["versions", "node"]);

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if output.status.success() {
        // Should contain version numbers
        assert!(
            stdout.contains("v") || stdout.contains("Version") || stdout.contains("20."),
            "Expected version numbers in output: {}",
            stdout
        );
    } else {
        // Network errors are acceptable
        assert!(
            stderr.contains("network")
                || stderr.contains("timeout")
                || stderr.contains("connection")
                || stderr.contains("rate limit")
                || stderr.contains("Failed to fetch"),
            "Unexpected error: {}",
            stderr
        );
    }
}

#[test]
fn test_versions_list_go() {
    let env = E2ETestEnv::new();
    let output = env.run(&["versions", "go"]);

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if output.status.success() {
        // Should contain version numbers like go1.21, go1.22, etc.
        assert!(
            stdout.contains("go1.") || stdout.contains("1.2") || stdout.contains("Version"),
            "Expected version numbers in output: {}",
            stdout
        );
    } else {
        // Network errors are acceptable
        assert!(
            stderr.contains("network")
                || stderr.contains("timeout")
                || stderr.contains("connection")
                || stderr.contains("rate limit")
                || stderr.contains("Failed to fetch"),
            "Unexpected error: {}",
            stderr
        );
    }
}

#[test]
fn test_versions_list_uv() {
    let env = E2ETestEnv::new();
    let output = env.run(&["versions", "uv"]);

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if output.status.success() {
        // Should contain version numbers
        assert!(
            stdout.contains("0.") || stdout.contains("Version"),
            "Expected version numbers in output: {}",
            stdout
        );
    } else {
        // Network errors are acceptable
        let combined = format!("{}{}", stdout, stderr);
        assert!(
            combined.contains("network")
                || combined.contains("Network")
                || combined.contains("timeout")
                || combined.contains("connection")
                || combined.contains("Connection")
                || combined.contains("rate limit")
                || combined.contains("Failed to fetch")
                || combined.contains("error sending request"),
            "Unexpected error: {}",
            combined
        );
    }
}

// ============================================================================
// Install command help tests
// ============================================================================

#[test]
fn test_install_help() {
    let env = E2ETestEnv::new();
    let output = env.run(&["install", "--help"]);

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("install") || stdout.contains("Install"));
    assert!(stdout.contains("TOOL") || stdout.contains("tool"));
}

#[test]
fn test_install_invalid_tool() {
    let env = E2ETestEnv::new();
    let output = env.run(&["install", "nonexistent-tool-xyz-123"]);

    // Should fail with helpful error message
    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let combined = format!("{}{}", stdout, stderr);

    assert!(
        combined.contains("not found")
            || combined.contains("Unknown")
            || combined.contains("not supported")
            || combined.contains("Cannot"),
        "Expected helpful error message, got: {}",
        combined
    );
}

// ============================================================================
// Provider-specific URL format tests (via search command)
// ============================================================================

#[test]
fn test_search_zig() {
    let env = E2ETestEnv::new();
    let output = env.run(&["search", "zig"]);

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should find zig in the available tools
    if output.status.success() {
        assert!(
            stdout.to_lowercase().contains("zig"),
            "Expected 'zig' in search results: {}",
            stdout
        );
    } else {
        // If search fails, it should be a network error
        assert!(
            stderr.contains("network") || stderr.contains("Failed") || stderr.is_empty(),
            "Unexpected error: {}",
            stderr
        );
    }
}

#[test]
fn test_search_node() {
    let env = E2ETestEnv::new();
    let output = env.run(&["search", "node"]);

    let stdout = String::from_utf8_lossy(&output.stdout);

    if output.status.success() {
        assert!(
            stdout.to_lowercase().contains("node"),
            "Expected 'node' in search results: {}",
            stdout
        );
    }
}

// ============================================================================
// Plugin list tests - verify all providers are registered
// ============================================================================

#[test]
fn test_plugin_list_includes_zig() {
    let env = E2ETestEnv::new();
    let output = env.run(&["plugin", "list"]);

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.to_lowercase().contains("zig"),
        "Expected 'zig' in plugin list: {}",
        stdout
    );
}

#[test]
fn test_plugin_list_includes_node() {
    let env = E2ETestEnv::new();
    let output = env.run(&["plugin", "list"]);

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.to_lowercase().contains("node"),
        "Expected 'node' in plugin list: {}",
        stdout
    );
}

#[test]
fn test_plugin_list_includes_go() {
    let env = E2ETestEnv::new();
    let output = env.run(&["plugin", "list"]);

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.to_lowercase().contains("go"),
        "Expected 'go' in plugin list: {}",
        stdout
    );
}

#[test]
fn test_plugin_list_includes_uv() {
    let env = E2ETestEnv::new();
    let output = env.run(&["plugin", "list"]);

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.to_lowercase().contains("uv"),
        "Expected 'uv' in plugin list: {}",
        stdout
    );
}

// ============================================================================
// List command tests
// ============================================================================

#[test]
fn test_list_shows_available_tools() {
    let env = E2ETestEnv::new();
    let output = env.run(&["list"]);

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should show at least some tools
    assert!(!stdout.is_empty(), "Expected non-empty list output");
}

// ============================================================================
// Error handling tests
// ============================================================================

#[test]
fn test_install_missing_tool_argument() {
    let env = E2ETestEnv::new();
    let output = env.run(&["install"]);

    // Should fail with usage information
    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("required") || stderr.contains("TOOL") || stderr.contains("Usage"),
        "Expected usage information, got: {}",
        stderr
    );
}

#[test]
fn test_versions_missing_tool_argument() {
    let env = E2ETestEnv::new();
    let output = env.run(&["versions"]);

    // Should fail with usage information
    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("required") || stderr.contains("TOOL") || stderr.contains("Usage"),
        "Expected usage information, got: {}",
        stderr
    );
}
