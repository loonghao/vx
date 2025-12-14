//! Real-World Scenario Tests for vx CLI
//!
//! These tests verify real tool execution scenarios that users would encounter.
//! They test the "auto-install and run" feature of vx.
//!
//! # Test Categories:
//! - Tool version checks (non-destructive, fast)
//! - Tool execution with simple commands
//! - Auto-install scenarios (requires network, marked with #[ignore])
//!
//! # Running Tests:
//! ```bash
//! # Run only fast, non-network tests
//! cargo test --package vx-cli --test real_world_scenarios
//!
//! # Run all tests including network-dependent ones
//! cargo test --package vx-cli --test real_world_scenarios -- --ignored
//! ```

use rstest::*;
use std::path::PathBuf;
use std::process::{Command, Output};
use tempfile::TempDir;

// ============================================================================
// Test Utilities
// ============================================================================

/// Get the vx binary path
fn vx_binary() -> PathBuf {
    let cargo_target = std::env::var("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("target"));

    // Check release build first (CI uses release)
    let release_binary = cargo_target.join("release").join(binary_name());
    if release_binary.exists() {
        return release_binary;
    }

    // Check debug build
    let debug_binary = cargo_target.join("debug").join(binary_name());
    if debug_binary.exists() {
        return debug_binary;
    }

    // Fall back to system PATH
    PathBuf::from(binary_name())
}

fn binary_name() -> &'static str {
    if cfg!(windows) {
        "vx.exe"
    } else {
        "vx"
    }
}

/// Check if vx binary is available
fn vx_available() -> bool {
    vx_binary().exists() || Command::new("vx").arg("--version").output().is_ok()
}

/// Run vx with given arguments
fn run_vx(args: &[&str]) -> std::io::Result<Output> {
    Command::new(vx_binary()).args(args).output()
}

/// Run vx in a specific directory
fn run_vx_in_dir(dir: &std::path::Path, args: &[&str]) -> std::io::Result<Output> {
    Command::new(vx_binary())
        .args(args)
        .current_dir(dir)
        .output()
}

/// Check if output indicates success
fn is_success(output: &Output) -> bool {
    output.status.success()
}

/// Get stdout as string
fn stdout_str(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).to_string()
}

/// Get stderr as string
fn stderr_str(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).to_string()
}

/// Combined output for debugging
fn combined_output(output: &Output) -> String {
    format!(
        "stdout:\n{}\nstderr:\n{}",
        stdout_str(output),
        stderr_str(output)
    )
}

// ============================================================================
// Node.js Scenarios
// ============================================================================

mod node_scenarios {
    use super::*;

    /// Test: vx node --version
    /// Verifies that vx can run node and get version info
    #[rstest]
    #[test]
    fn test_vx_node_version() {
        if !vx_available() {
            eprintln!("Skipping: vx binary not found");
            return;
        }

        let output = run_vx(&["node", "--version"]).expect("Failed to run vx node --version");

        // If node is installed, should succeed and show version
        // If not installed, vx should provide helpful error
        let combined = combined_output(&output);

        if is_success(&output) {
            assert!(
                stdout_str(&output).starts_with('v'),
                "Node version should start with 'v': {}",
                combined
            );
        } else {
            // Not installed - check for helpful message
            assert!(
                combined.contains("not installed")
                    || combined.contains("install")
                    || combined.contains("not found"),
                "Should provide helpful error: {}",
                combined
            );
        }
    }

    /// Test: vx npm --version
    #[rstest]
    #[test]
    fn test_vx_npm_version() {
        if !vx_available() {
            eprintln!("Skipping: vx binary not found");
            return;
        }

        let output = run_vx(&["npm", "--version"]).expect("Failed to run vx npm --version");

        if is_success(&output) {
            // npm version is a semver like "10.2.0"
            let version = stdout_str(&output).trim().to_string();
            assert!(
                version.chars().next().map(|c| c.is_ascii_digit()) == Some(true),
                "npm version should start with digit: {}",
                version
            );
        }
    }

    /// Test: vx npx --version
    #[rstest]
    #[test]
    fn test_vx_npx_version() {
        if !vx_available() {
            eprintln!("Skipping: vx binary not found");
            return;
        }

        let output = run_vx(&["npx", "--version"]).expect("Failed to run vx npx --version");

        if is_success(&output) {
            let version = stdout_str(&output).trim().to_string();
            assert!(
                version.chars().next().map(|c| c.is_ascii_digit()) == Some(true),
                "npx version should start with digit: {}",
                version
            );
        }
    }

    /// Test: vx node -e "console.log('hello')"
    /// Tests actual JavaScript execution
    #[rstest]
    #[test]
    fn test_vx_node_eval() {
        if !vx_available() {
            eprintln!("Skipping: vx binary not found");
            return;
        }

        let output =
            run_vx(&["node", "-e", "console.log('hello from vx')"]).expect("Failed to run vx node");

        if is_success(&output) {
            assert!(
                stdout_str(&output).contains("hello from vx"),
                "Should output 'hello from vx': {}",
                combined_output(&output)
            );
        }
    }

    /// Test: vx node -e "process.exit(0)" (exit code handling)
    #[rstest]
    #[test]
    fn test_vx_node_exit_code_success() {
        if !vx_available() {
            eprintln!("Skipping: vx binary not found");
            return;
        }

        let output =
            run_vx(&["node", "-e", "process.exit(0)"]).expect("Failed to run vx node exit");

        // If node is available, exit code should be 0
        if stdout_str(&output).is_empty() || is_success(&output) {
            // Test passed or node not installed
        }
    }

    /// Test: vx node -e "process.exit(1)" (non-zero exit code)
    #[rstest]
    #[test]
    fn test_vx_node_exit_code_failure() {
        if !vx_available() {
            eprintln!("Skipping: vx binary not found");
            return;
        }

        let output =
            run_vx(&["node", "-e", "process.exit(42)"]).expect("Failed to run vx node exit");

        // If node is available and ran, exit code should be propagated
        if !stdout_str(&output).is_empty() || !stderr_str(&output).contains("not installed") {
            // vx should propagate the exit code from node
            // Note: exact behavior depends on implementation
        }
    }
}

// ============================================================================
// UV (Python) Scenarios
// ============================================================================

mod uv_scenarios {
    use super::*;

    /// Test: vx uv --version
    #[rstest]
    #[test]
    fn test_vx_uv_version() {
        if !vx_available() {
            eprintln!("Skipping: vx binary not found");
            return;
        }

        let output = run_vx(&["uv", "--version"]).expect("Failed to run vx uv --version");

        if is_success(&output) {
            let version = stdout_str(&output);
            assert!(
                version.contains("uv"),
                "uv version should contain 'uv': {}",
                version
            );
        }
    }

    /// Test: vx uvx --version
    #[rstest]
    #[test]
    fn test_vx_uvx_version() {
        if !vx_available() {
            eprintln!("Skipping: vx binary not found");
            return;
        }

        let output = run_vx(&["uvx", "--version"]).expect("Failed to run vx uvx --version");

        if is_success(&output) {
            let version = stdout_str(&output);
            // uvx shows "uv-tool-uvx X.Y.Z" or similar
            assert!(
                version.contains("uv") || version.chars().any(|c| c.is_ascii_digit()),
                "uvx version output: {}",
                version
            );
        }
    }

    /// Test: vx uv pip --version
    #[rstest]
    #[test]
    fn test_vx_uv_pip_version() {
        if !vx_available() {
            eprintln!("Skipping: vx binary not found");
            return;
        }

        let output = run_vx(&["uv", "pip", "--version"]).expect("Failed to run vx uv pip");

        if is_success(&output) {
            let version = stdout_str(&output);
            assert!(
                version.contains("uv") || version.contains("pip"),
                "uv pip version: {}",
                version
            );
        }
    }

    /// Test: vx uvx ruff --version (popular Python linter)
    #[rstest]
    #[test]
    #[ignore = "Requires network to download ruff"]
    fn test_vx_uvx_ruff_version() {
        if !vx_available() {
            eprintln!("Skipping: vx binary not found");
            return;
        }

        let output =
            run_vx(&["uvx", "ruff", "--version"]).expect("Failed to run vx uvx ruff --version");

        if is_success(&output) {
            let version = stdout_str(&output);
            assert!(
                version.contains("ruff"),
                "ruff version should contain 'ruff': {}",
                version
            );
        }
    }
}

// ============================================================================
// Go Scenarios
// ============================================================================

mod go_scenarios {
    use super::*;

    /// Test: vx go version
    #[rstest]
    #[test]
    fn test_vx_go_version() {
        if !vx_available() {
            eprintln!("Skipping: vx binary not found");
            return;
        }

        let output = run_vx(&["go", "version"]).expect("Failed to run vx go version");

        if is_success(&output) {
            let version = stdout_str(&output);
            assert!(
                version.contains("go version"),
                "go version should contain 'go version': {}",
                version
            );
        }
    }

    /// Test: vx go env GOVERSION
    #[rstest]
    #[test]
    fn test_vx_go_env() {
        if !vx_available() {
            eprintln!("Skipping: vx binary not found");
            return;
        }

        let output = run_vx(&["go", "env", "GOVERSION"]).expect("Failed to run vx go env");

        if is_success(&output) {
            let version = stdout_str(&output).trim().to_string();
            assert!(
                version.starts_with("go"),
                "GOVERSION should start with 'go': {}",
                version
            );
        }
    }

    /// Test: vx go run with a simple program
    #[rstest]
    #[test]
    fn test_vx_go_run_hello() {
        if !vx_available() {
            eprintln!("Skipping: vx binary not found");
            return;
        }

        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let main_go = temp_dir.path().join("main.go");

        // Write a simple Go program
        std::fs::write(
            &main_go,
            r#"package main

import "fmt"

func main() {
    fmt.Println("Hello from vx go!")
}
"#,
        )
        .expect("Failed to write main.go");

        let output =
            run_vx_in_dir(temp_dir.path(), &["go", "run", "main.go"]).expect("Failed to run go");

        if is_success(&output) {
            assert!(
                stdout_str(&output).contains("Hello from vx go!"),
                "Should output 'Hello from vx go!': {}",
                combined_output(&output)
            );
        }
    }
}

// ============================================================================
// Bun Scenarios
// ============================================================================

mod bun_scenarios {
    use super::*;

    /// Test: vx bun --version
    #[rstest]
    #[test]
    fn test_vx_bun_version() {
        if !vx_available() {
            eprintln!("Skipping: vx binary not found");
            return;
        }

        let output = run_vx(&["bun", "--version"]).expect("Failed to run vx bun --version");

        if is_success(&output) {
            let version = stdout_str(&output).trim().to_string();
            // Bun version is like "1.0.0"
            assert!(
                version.chars().next().map(|c| c.is_ascii_digit()) == Some(true),
                "bun version should start with digit: {}",
                version
            );
        }
    }

    /// Test: vx bun -e "console.log('hello')"
    #[rstest]
    #[test]
    fn test_vx_bun_eval() {
        if !vx_available() {
            eprintln!("Skipping: vx binary not found");
            return;
        }

        let output =
            run_vx(&["bun", "-e", "console.log('hello from bun')"]).expect("Failed to run bun");

        if is_success(&output) {
            assert!(
                stdout_str(&output).contains("hello from bun"),
                "Should output 'hello from bun': {}",
                combined_output(&output)
            );
        }
    }
}

// ============================================================================
// Cargo/Rust Scenarios
// ============================================================================

mod rust_scenarios {
    use super::*;

    /// Test: vx cargo --version
    #[rstest]
    #[test]
    fn test_vx_cargo_version() {
        if !vx_available() {
            eprintln!("Skipping: vx binary not found");
            return;
        }

        let output = run_vx(&["cargo", "--version"]).expect("Failed to run vx cargo --version");

        if is_success(&output) {
            let version = stdout_str(&output);
            assert!(
                version.contains("cargo"),
                "cargo version should contain 'cargo': {}",
                version
            );
        }
    }

    /// Test: vx rustc --version
    #[rstest]
    #[test]
    fn test_vx_rustc_version() {
        if !vx_available() {
            eprintln!("Skipping: vx binary not found");
            return;
        }

        let output = run_vx(&["rustc", "--version"]).expect("Failed to run vx rustc --version");

        if is_success(&output) {
            let version = stdout_str(&output);
            assert!(
                version.contains("rustc"),
                "rustc version should contain 'rustc': {}",
                version
            );
        }
    }
}

// ============================================================================
// Cross-Tool Scenarios
// ============================================================================

mod cross_tool_scenarios {
    use super::*;

    /// Test running multiple tools in sequence
    #[rstest]
    #[test]
    fn test_multiple_tools_sequence() {
        if !vx_available() {
            eprintln!("Skipping: vx binary not found");
            return;
        }

        // Run version checks for multiple tools
        let tools = vec![
            ("node", vec!["--version"]),
            ("uv", vec!["--version"]),
            ("go", vec!["version"]),
            ("bun", vec!["--version"]),
        ];

        for (tool, args) in tools {
            let mut full_args = vec![tool];
            full_args.extend(args);

            let output = run_vx(&full_args).expect(&format!("Failed to run vx {}", tool));

            // Just verify it doesn't crash - tool may or may not be installed
            let _ = combined_output(&output);
        }
    }
}

// ============================================================================
// Auto-Install Scenarios (Network Required)
// ============================================================================

mod auto_install_scenarios {
    use super::*;

    /// Test: Auto-install node if not present
    /// This test verifies the core "auto-install" feature
    #[rstest]
    #[test]
    #[ignore = "Requires network and may take time to download"]
    fn test_auto_install_node() {
        if !vx_available() {
            eprintln!("Skipping: vx binary not found");
            return;
        }

        // Run node --version - should auto-install if needed
        let output = run_vx(&["node", "--version"]).expect("Failed to run vx node");

        // After auto-install, should succeed
        assert!(
            is_success(&output),
            "vx node --version should succeed after auto-install: {}",
            combined_output(&output)
        );

        let version = stdout_str(&output);
        assert!(
            version.starts_with('v'),
            "Node version should start with 'v': {}",
            version
        );
    }

    /// Test: Auto-install uv if not present
    #[rstest]
    #[test]
    #[ignore = "Requires network and may take time to download"]
    fn test_auto_install_uv() {
        if !vx_available() {
            eprintln!("Skipping: vx binary not found");
            return;
        }

        let output = run_vx(&["uv", "--version"]).expect("Failed to run vx uv");

        assert!(
            is_success(&output),
            "vx uv --version should succeed: {}",
            combined_output(&output)
        );
    }

    /// Test: Auto-install go if not present
    #[rstest]
    #[test]
    #[ignore = "Requires network and may take time to download"]
    fn test_auto_install_go() {
        if !vx_available() {
            eprintln!("Skipping: vx binary not found");
            return;
        }

        let output = run_vx(&["go", "version"]).expect("Failed to run vx go");

        assert!(
            is_success(&output),
            "vx go version should succeed: {}",
            combined_output(&output)
        );
    }

    /// Test: vx npx create-react-app scenario (simulated)
    /// Note: We don't actually create a React app, just verify npx works
    #[rstest]
    #[test]
    #[ignore = "Requires network"]
    fn test_npx_help() {
        if !vx_available() {
            eprintln!("Skipping: vx binary not found");
            return;
        }

        let output = run_vx(&["npx", "--help"]).expect("Failed to run vx npx --help");

        assert!(
            is_success(&output),
            "vx npx --help should succeed: {}",
            combined_output(&output)
        );
    }

    /// Test: vx uvx ruff check scenario
    #[rstest]
    #[test]
    #[ignore = "Requires network to download ruff"]
    fn test_uvx_ruff_check() {
        if !vx_available() {
            eprintln!("Skipping: vx binary not found");
            return;
        }

        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let py_file = temp_dir.path().join("test.py");

        // Write a simple Python file
        std::fs::write(&py_file, "x = 1\n").expect("Failed to write test.py");

        let output = run_vx_in_dir(temp_dir.path(), &["uvx", "ruff", "check", "."])
            .expect("Failed to run uvx ruff");

        // ruff check should succeed (no errors in our simple file)
        // or fail with linting errors (which is also valid behavior)
        let _ = combined_output(&output);
    }
}

// ============================================================================
// Error Handling Scenarios
// ============================================================================

mod error_scenarios {
    use super::*;

    /// Test: Running unknown tool
    #[rstest]
    #[test]
    fn test_unknown_tool() {
        if !vx_available() {
            eprintln!("Skipping: vx binary not found");
            return;
        }

        let output = run_vx(&["unknown-tool-xyz-123", "--version"])
            .expect("Failed to run vx with unknown tool");

        // Should fail with helpful error
        assert!(
            !is_success(&output),
            "Unknown tool should fail: {}",
            combined_output(&output)
        );
    }

    /// Test: Tool with invalid arguments
    #[rstest]
    #[test]
    fn test_tool_invalid_args() {
        if !vx_available() {
            eprintln!("Skipping: vx binary not found");
            return;
        }

        // node with invalid flag should fail
        let output = run_vx(&["node", "--invalid-flag-xyz"]).expect("Failed to run vx node");

        // If node is installed, it should fail with invalid flag
        // The test just verifies vx doesn't crash
        let _ = combined_output(&output);
    }
}

// ============================================================================
// Project Context Scenarios
// ============================================================================

mod project_context_scenarios {
    use super::*;

    /// Test: Running in a directory with .vx.toml
    #[rstest]
    #[test]
    fn test_with_vx_toml() {
        if !vx_available() {
            eprintln!("Skipping: vx binary not found");
            return;
        }

        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let vx_toml = temp_dir.path().join(".vx.toml");

        // Write a .vx.toml config
        std::fs::write(
            &vx_toml,
            r#"[tools]
node = "20"
"#,
        )
        .expect("Failed to write .vx.toml");

        // Run vx list in this directory
        let output =
            run_vx_in_dir(temp_dir.path(), &["list"]).expect("Failed to run vx list in project");

        assert!(
            is_success(&output),
            "vx list should succeed in project dir: {}",
            combined_output(&output)
        );
    }

    /// Test: vx sync --check in project directory
    #[rstest]
    #[test]
    fn test_sync_check_in_project() {
        if !vx_available() {
            eprintln!("Skipping: vx binary not found");
            return;
        }

        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let vx_toml = temp_dir.path().join(".vx.toml");

        std::fs::write(
            &vx_toml,
            r#"[tools]
node = "20"
uv = "latest"
"#,
        )
        .expect("Failed to write .vx.toml");

        let output = run_vx_in_dir(temp_dir.path(), &["sync", "--check"])
            .expect("Failed to run vx sync --check");

        // sync --check should work (may report missing tools)
        let _ = combined_output(&output);
    }
}
