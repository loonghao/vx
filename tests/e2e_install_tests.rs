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

/// Check if an error message represents an acceptable network/API error in CI.
///
/// These errors are transient and not caused by bugs in vx itself:
/// - Network connectivity issues (timeout, connection refused, DNS failures)
/// - GitHub API rate limiting or server errors (502, 503, 504)
/// - Missing authentication tokens for rate-limited APIs
fn is_acceptable_network_error(output: &str) -> bool {
    let lower = output.to_lowercase();
    // Network-level errors
    lower.contains("network")
        || lower.contains("timeout")
        || lower.contains("time-out")
        || lower.contains("timed out")
        || lower.contains("connection")
        || lower.contains("dns")
        || lower.contains("error sending request")
        // HTTP server errors
        || lower.contains("502")
        || lower.contains("503")
        || lower.contains("504")
        || lower.contains("bad gateway")
        || lower.contains("service unavailable")
        || lower.contains("gateway")
        // Rate limiting
        || lower.contains("rate limit")
        || lower.contains("403")
        || lower.contains("too many requests")
        || lower.contains("429")
        // Auth token hints (GitHub API without token)
        || lower.contains("github_token")
        || lower.contains("gh_token")
        // Generic fetch failures
        || lower.contains("failed to fetch")
        || lower.contains("fetch failed")
        || lower.contains("api fetch failed")
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
        let combined = format!("{}{}", stdout, stderr);
        assert!(
            is_acceptable_network_error(&combined),
            "Unexpected error listing zig versions: {}",
            combined
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
        let combined = format!("{}{}", stdout, stderr);
        assert!(
            is_acceptable_network_error(&combined),
            "Unexpected error listing node versions: {}",
            combined
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
        let combined = format!("{}{}", stdout, stderr);
        assert!(
            is_acceptable_network_error(&combined),
            "Unexpected error listing go versions: {}",
            combined
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
// Python version listing (regression: python-build-standalone API)
// ============================================================================

#[test]
fn test_versions_list_python() {
    let env = E2ETestEnv::new();
    let output = env.run(&["versions", "python"]);

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if output.status.success() {
        // Should contain Python version numbers like 3.12, 3.13, etc.
        assert!(
            stdout.contains("3.") || stdout.contains("Version"),
            "Expected Python version numbers in output: {}",
            stdout
        );
    } else {
        // Network errors are acceptable in CI
        // Python versions come from GitHub API (python-build-standalone)
        // which is particularly prone to rate limiting and gateway timeouts
        let combined = format!("{}{}", stdout, stderr);
        assert!(
            is_acceptable_network_error(&combined),
            "Unexpected error listing Python versions: {}",
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
