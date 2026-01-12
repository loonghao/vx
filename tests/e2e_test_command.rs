//! E2E tests for `vx test` command
//!
//! Tests the universal provider testing framework across different scenarios:
//! - Testing individual runtimes
//! - Testing all providers
//! - Testing local providers (development)
//! - Testing remote extensions

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
    workdir: TempDir,
}

impl E2ETestEnv {
    fn new() -> Self {
        Self {
            home: TempDir::new().expect("Failed to create temp VX_HOME"),
            workdir: TempDir::new().expect("Failed to create temp workdir"),
        }
    }

    fn run(&self, args: &[&str]) -> std::process::Output {
        Command::new(vx_binary())
            .args(args)
            .env("VX_HOME", self.home.path())
            .current_dir(self.workdir.path())
            .output()
            .expect("Failed to execute vx command")
    }

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

    /// Create a mock provider.toml for testing
    fn create_mock_provider(&self, name: &str, runtimes: &[(&str, bool)]) -> PathBuf {
        let provider_dir = self.workdir.path().join(format!("provider-{}", name));
        std::fs::create_dir_all(&provider_dir).unwrap();

        let mut toml_content = format!(
            r#"
name = "{}"
description = "Test provider for {}"
version = "0.1.0"

[[runtimes]]
"#,
            name, name
        );

        for (runtime, supported) in runtimes {
            let platforms = if *supported {
                r#"
[[runtimes.platforms]]
os = "windows"
arch = "x86_64"

[[runtimes.platforms]]
os = "linux"
arch = "x86_64"

[[runtimes.platforms]]
os = "macos"
arch = "x86_64"
"#
            } else {
                "" // No platforms = not supported
            };

            toml_content.push_str(&format!(
                r#"
name = "{}"
{}
"#,
                runtime, platforms
            ));
        }

        let toml_path = provider_dir.join("provider.toml");
        std::fs::write(&toml_path, toml_content).unwrap();

        provider_dir
    }
}

// ============================================================================
// Test: vx test <runtime>
// ============================================================================

#[test]
fn test_single_runtime_platform_check() {
    let env = E2ETestEnv::new();

    // Test a runtime that should be supported on all platforms
    let output = env.run(&["test", "go", "--platform-only", "--quiet"]);
    
    // go should be supported on all common platforms
    assert!(output.status.success(), "go should be platform supported");
}

#[test]
fn test_single_runtime_not_installed() {
    let env = E2ETestEnv::new();

    // Test a runtime that's likely not installed
    let output = env.run(&["test", "zig", "--quiet"]);
    
    // Should exit with 1 (not available) but not crash
    assert!(!output.status.success() || output.status.code() == Some(1));
}

#[test]
fn test_single_runtime_json_output() {
    let env = E2ETestEnv::new();

    let output = env.run(&["test", "node", "--json"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // Should output valid JSON
    assert!(
        stdout.contains("\"runtime\"") && stdout.contains("\"platform_supported\""),
        "Expected JSON output with runtime info"
    );
}

#[test]
fn test_single_runtime_detailed() {
    let env = E2ETestEnv::new();

    let output = env.run(&["test", "go", "--detailed"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // Should show detailed information
    assert!(
        stdout.contains("✓") || stdout.contains("✗") || stdout.contains("⚠"),
        "Expected status symbols in detailed output"
    );
}

#[test]
fn test_unknown_runtime() {
    let env = E2ETestEnv::new();

    let output = env.run(&["test", "unknown-runtime-xyz-123"]);
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    // Should fail with unknown runtime error
    assert!(!output.status.success());
    assert!(
        stderr.contains("Unknown") || stderr.contains("not found"),
        "Expected 'Unknown runtime' error message"
    );
}

// ============================================================================
// Test: vx test --all
// ============================================================================

#[test]
fn test_all_providers() {
    let env = E2ETestEnv::new();

    let output = env.run(&["test", "--all", "--platform-only"]);
    
    // Should test multiple runtimes
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Test Summary") || stdout.contains("Total"),
        "Expected test summary in output"
    );
}

#[test]
fn test_all_providers_json() {
    let env = E2ETestEnv::new();

    let output = env.run(&["test", "--all", "--platform-only", "--json"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // Should output valid JSON summary
    assert!(
        stdout.contains("\"total\"") && stdout.contains("\"passed\"") && stdout.contains("\"failed\""),
        "Expected JSON summary with test counts"
    );
    
    // Verify it's valid JSON
    let _: serde_json::Value = serde_json::from_str(&stdout)
        .expect("Output should be valid JSON");
}

#[test]
fn test_all_providers_quiet() {
    let env = E2ETestEnv::new();

    let output = env.run(&["test", "--all", "--platform-only", "--quiet"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // Quiet mode should produce minimal output
    assert!(
        stdout.is_empty() || stdout.len() < 100,
        "Expected minimal output in quiet mode"
    );
}

// ============================================================================
// Test: vx test --local <path>
// ============================================================================

#[test]
fn test_local_provider_valid() {
    let env = E2ETestEnv::new();
    
    // Create a mock provider
    let provider_dir = env.create_mock_provider("test-tool", &[
        ("mytool", true),
        ("myutil", true),
    ]);

    let output = env.run(&[
        "test",
        "--local",
        provider_dir.to_str().unwrap(),
        "--platform-only",
    ]);
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // Should successfully validate provider
    assert!(
        stdout.contains("Validating") || stdout.contains("Provider"),
        "Expected provider validation message"
    );
}

#[test]
fn test_local_provider_invalid_path() {
    let env = E2ETestEnv::new();

    let output = env.run(&[
        "test",
        "--local",
        "/nonexistent/path/to/provider",
    ]);
    
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    // Should fail with appropriate error
    assert!(!output.status.success());
    assert!(
        stderr.contains("not found") || stderr.contains("provider.toml"),
        "Expected provider.toml not found error"
    );
}

#[test]
fn test_local_provider_missing_toml() {
    let env = E2ETestEnv::new();
    
    // Create directory without provider.toml
    let empty_dir = env.workdir.path().join("empty-provider");
    std::fs::create_dir_all(&empty_dir).unwrap();

    let output = env.run(&[
        "test",
        "--local",
        empty_dir.to_str().unwrap(),
    ]);
    
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    // Should fail
    assert!(!output.status.success());
    assert!(
        stderr.contains("provider.toml") || stderr.contains("Not a valid provider"),
        "Expected provider.toml missing error"
    );
}

#[test]
fn test_local_provider_json_output() {
    let env = E2ETestEnv::new();
    
    let provider_dir = env.create_mock_provider("json-test", &[("tool1", true)]);

    let output = env.run(&[
        "test",
        "--local",
        provider_dir.to_str().unwrap(),
        "--json",
    ]);
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // Should output valid JSON
    let _: serde_json::Value = serde_json::from_str(&stdout)
        .expect("Output should be valid JSON");
}

// ============================================================================
// Test: vx test --extension <url>
// ============================================================================

#[test]
#[ignore] // Requires network access
fn test_extension_github_url() {
    let env = E2ETestEnv::new();

    // Test with a hypothetical extension URL
    let output = env.run(&[
        "test",
        "--extension",
        "https://github.com/example/vx-provider-example",
    ]);
    
    // Should attempt to download
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    
    assert!(
        combined.contains("Download") || combined.contains("not yet implemented"),
        "Expected download attempt or not-implemented message"
    );
}

#[test]
fn test_extension_invalid_url() {
    let env = E2ETestEnv::new();

    let output = env.run(&[
        "test",
        "--extension",
        "not-a-valid-url",
    ]);
    
    // Should fail gracefully
    assert!(!output.status.success() || output.status.code() == Some(1));
}

// ============================================================================
// Test: Error handling
// ============================================================================

#[test]
fn test_no_arguments() {
    let env = E2ETestEnv::new();

    let output = env.run(&["test"]);
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    // Should show helpful error
    assert!(!output.status.success());
    assert!(
        stderr.contains("Please specify") || stderr.contains("--help"),
        "Expected helpful error message"
    );
}

#[test]
fn test_conflicting_arguments() {
    let env = E2ETestEnv::new();

    // --all and runtime name should conflict
    let output = env.run(&["test", "node", "--all"]);
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    // clap should catch the conflict
    assert!(!output.status.success());
    assert!(
        stderr.contains("conflict") || stderr.contains("cannot be used"),
        "Expected argument conflict error"
    );
}

#[test]
fn test_help_message() {
    let env = E2ETestEnv::new();

    let output = env.run(&["test", "--help"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // Should show comprehensive help
    assert!(output.status.success());
    assert!(stdout.contains("Test runtime availability"));
    assert!(stdout.contains("--all"));
    assert!(stdout.contains("--local"));
    assert!(stdout.contains("--extension"));
}

// ============================================================================
// Test: CI/CD Integration scenarios
// ============================================================================

#[test]
fn test_ci_scenario_check_tool_available() {
    let env = E2ETestEnv::new();

    // Simulate CI checking if a tool is available
    let output = env.run(&["test", "go", "--quiet"]);
    
    // Exit code should indicate availability
    // 0 = available, 1 = not available
    assert!(output.status.code() == Some(0) || output.status.code() == Some(1));
}

#[test]
fn test_ci_scenario_json_parsing() {
    let env = E2ETestEnv::new();

    let output = env.run(&["test", "node", "--json"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // CI should be able to parse JSON output
    let result: serde_json::Value = serde_json::from_str(&stdout)
        .expect("CI should be able to parse JSON");
    
    // Verify expected fields
    assert!(result.get("runtime").is_some());
    assert!(result.get("platform_supported").is_some());
    assert!(result.get("available").is_some());
}

#[test]
fn test_ci_scenario_test_all_providers() {
    let env = E2ETestEnv::new();

    // CI running comprehensive tests
    let output = env.run(&["test", "--all", "--json", "--platform-only"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    let summary: serde_json::Value = serde_json::from_str(&stdout)
        .expect("CI should parse summary JSON");
    
    // Verify summary structure
    assert!(summary.get("total").is_some());
    assert!(summary.get("passed").is_some());
    assert!(summary.get("failed").is_some());
    assert!(summary.get("results").and_then(|v| v.as_array()).is_some());
}
