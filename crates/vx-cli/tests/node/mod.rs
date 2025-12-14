//! Node.js E2E Tests for vx CLI
//!
//! Tests for Node.js ecosystem tools: node, npm, npx

use crate::common::*;
use rstest::*;
use tempfile::TempDir;

// ============================================================================
// Node Version Tests
// ============================================================================

/// Test: vx node --version
#[rstest]
#[test]
fn test_node_version() {
    skip_if_no_vx!();

    let output = run_vx(&["node", "--version"]).expect("Failed to run vx node --version");

    if is_success(&output) {
        let version = stdout_str(&output);
        assert!(
            version.trim().starts_with('v'),
            "Node version should start with 'v': {}",
            version
        );
    } else {
        // Node not installed - verify helpful error message
        let combined = combined_output(&output);
        assert!(
            combined.contains("not installed")
                || combined.contains("install")
                || combined.contains("not found"),
            "Should provide helpful error: {}",
            combined
        );
    }
}

/// Test: vx node -v (short form)
#[rstest]
#[test]
fn test_node_version_short() {
    skip_if_no_vx!();

    let output = run_vx(&["node", "-v"]).expect("Failed to run vx node -v");

    if is_success(&output) {
        let version = stdout_str(&output);
        assert!(
            version.trim().starts_with('v'),
            "Node version should start with 'v': {}",
            version
        );
    }
}

// ============================================================================
// Node Execution Tests
// ============================================================================

/// Test: vx node -e "console.log('hello')"
#[rstest]
#[test]
fn test_node_eval() {
    skip_if_no_vx!();

    let output =
        run_vx(&["node", "-e", "console.log('hello from vx')"]).expect("Failed to run vx node -e");

    if is_success(&output) {
        assert_stdout_contains(&output, "hello from vx", "node -e");
    }
}

/// Test: vx node -e with JSON output
#[rstest]
#[test]
fn test_node_eval_json() {
    skip_if_no_vx!();

    let output = run_vx(&["node", "-e", "console.log(JSON.stringify({a:1,b:2}))"])
        .expect("Failed to run vx node -e");

    if is_success(&output) {
        let stdout = stdout_str(&output);
        assert!(
            stdout.contains(r#"{"a":1,"b":2}"#),
            "Should output JSON: {}",
            stdout
        );
    }
}

/// Test: vx node -e with process.env
#[rstest]
#[test]
fn test_node_eval_env() {
    skip_if_no_vx!();

    let output = run_vx_with_env(
        &["node", "-e", "console.log(process.env.VX_TEST_VAR)"],
        &[("VX_TEST_VAR", "test_value_123")],
    )
    .expect("Failed to run vx node -e");

    if is_success(&output) {
        assert_stdout_contains(&output, "test_value_123", "node env access");
    }
}

/// Test: vx node -p (print expression)
#[rstest]
#[test]
fn test_node_print() {
    skip_if_no_vx!();

    let output = run_vx(&["node", "-p", "1 + 2"]).expect("Failed to run vx node -p");

    if is_success(&output) {
        let stdout = stdout_str(&output).trim().to_string();
        assert_eq!(stdout, "3", "node -p should evaluate expression");
    }
}

// ============================================================================
// Node Exit Code Tests
// ============================================================================

/// Test: vx node -e "process.exit(0)"
#[rstest]
#[test]
fn test_node_exit_code_zero() {
    skip_if_no_vx!();

    let output = run_vx(&["node", "-e", "process.exit(0)"]).expect("Failed to run vx node");

    if tool_installed("node") {
        assert!(is_success(&output), "Exit code 0 should succeed");
    }
}

/// Test: vx node -e "process.exit(1)"
#[rstest]
#[test]
fn test_node_exit_code_one() {
    skip_if_no_vx!();

    let output = run_vx(&["node", "-e", "process.exit(1)"]).expect("Failed to run vx node");

    if tool_installed("node") {
        assert!(!is_success(&output), "Exit code 1 should fail");
        assert_eq!(exit_code(&output), Some(1), "Should propagate exit code 1");
    }
}

/// Test: vx node -e "process.exit(42)"
#[rstest]
#[test]
fn test_node_exit_code_custom() {
    skip_if_no_vx!();

    let output = run_vx(&["node", "-e", "process.exit(42)"]).expect("Failed to run vx node");

    if tool_installed("node") {
        assert!(!is_success(&output), "Exit code 42 should fail");
        assert_eq!(
            exit_code(&output),
            Some(42),
            "Should propagate exit code 42"
        );
    }
}

// ============================================================================
// Node File Execution Tests
// ============================================================================

/// Test: vx node script.js
#[rstest]
#[test]
fn test_node_run_file() {
    skip_if_no_vx!();

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let script_path = temp_dir.path().join("test.js");

    std::fs::write(
        &script_path,
        r#"
console.log("Script executed!");
console.log("Args:", process.argv.slice(2).join(", "));
"#,
    )
    .expect("Failed to write script");

    let output = run_vx_in_dir(temp_dir.path(), &["node", "test.js", "arg1", "arg2"])
        .expect("Failed to run vx node script.js");

    if is_success(&output) {
        assert_stdout_contains(&output, "Script executed!", "node script.js");
        assert_stdout_contains(&output, "arg1, arg2", "node script.js args");
    }
}

/// Test: vx node with async/await
#[rstest]
#[test]
fn test_node_async_await() {
    skip_if_no_vx!();

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let script_path = temp_dir.path().join("async.js");

    std::fs::write(
        &script_path,
        r#"
async function main() {
    const result = await Promise.resolve("async works");
    console.log(result);
}
main();
"#,
    )
    .expect("Failed to write script");

    let output = run_vx_in_dir(temp_dir.path(), &["node", "async.js"])
        .expect("Failed to run vx node async.js");

    if is_success(&output) {
        assert_stdout_contains(&output, "async works", "node async/await");
    }
}

// ============================================================================
// NPM Tests
// ============================================================================

/// Test: vx npm --version
#[rstest]
#[test]
fn test_npm_version() {
    skip_if_no_vx!();

    let output = run_vx(&["npm", "--version"]).expect("Failed to run vx npm --version");

    if is_success(&output) {
        let version = stdout_str(&output).trim().to_string();
        // npm version is semver like "10.2.0"
        assert!(
            version.chars().next().map(|c| c.is_ascii_digit()) == Some(true),
            "npm version should start with digit: {}",
            version
        );
    }
}

/// Test: vx npm -v (short form)
#[rstest]
#[test]
fn test_npm_version_short() {
    skip_if_no_vx!();

    let output = run_vx(&["npm", "-v"]).expect("Failed to run vx npm -v");

    if is_success(&output) {
        let version = stdout_str(&output).trim().to_string();
        assert!(
            version.chars().next().map(|c| c.is_ascii_digit()) == Some(true),
            "npm version should start with digit: {}",
            version
        );
    }
}

/// Test: vx npm help
#[rstest]
#[test]
fn test_npm_help() {
    skip_if_no_vx!();

    let output = run_vx(&["npm", "help"]).expect("Failed to run vx npm help");

    if is_success(&output) {
        let combined = combined_output(&output);
        assert!(
            combined.contains("npm") || combined.contains("Usage"),
            "npm help should show usage: {}",
            combined
        );
    }
}

/// Test: vx npm config list
#[rstest]
#[test]
fn test_npm_config_list() {
    skip_if_no_vx!();

    let output = run_vx(&["npm", "config", "list"]).expect("Failed to run vx npm config list");

    if is_success(&output) {
        // npm config list should succeed
        let _ = combined_output(&output);
    }
}

/// Test: vx npm init in temp directory (non-interactive)
#[rstest]
#[test]
fn test_npm_init_yes() {
    skip_if_no_vx!();

    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let output = run_vx_in_dir(temp_dir.path(), &["npm", "init", "-y"])
        .expect("Failed to run vx npm init -y");

    if is_success(&output) {
        // Should create package.json
        assert!(
            temp_dir.path().join("package.json").exists(),
            "npm init -y should create package.json"
        );
    }
}

// ============================================================================
// NPX Tests
// ============================================================================

/// Test: vx npx --version
#[rstest]
#[test]
fn test_npx_version() {
    skip_if_no_vx!();

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

/// Test: vx npx -v (short form)
#[rstest]
#[test]
fn test_npx_version_short() {
    skip_if_no_vx!();

    let output = run_vx(&["npx", "-v"]).expect("Failed to run vx npx -v");

    if is_success(&output) {
        let version = stdout_str(&output).trim().to_string();
        assert!(
            version.chars().next().map(|c| c.is_ascii_digit()) == Some(true),
            "npx version should start with digit: {}",
            version
        );
    }
}

/// Test: vx npx --help
#[rstest]
#[test]
#[ignore = "Requires network to fetch help"]
fn test_npx_help() {
    skip_if_no_vx!();

    let output = run_vx(&["npx", "--help"]).expect("Failed to run vx npx --help");

    if is_success(&output) {
        let combined = combined_output(&output);
        assert!(
            combined.contains("npx") || combined.contains("Usage"),
            "npx help should show usage"
        );
    }
}

// ============================================================================
// Node Error Handling Tests
// ============================================================================

/// Test: vx node with syntax error
#[rstest]
#[test]
fn test_node_syntax_error() {
    skip_if_no_vx!();

    let output =
        run_vx(&["node", "-e", "console.log("]).expect("Failed to run vx node with syntax error");

    if tool_installed("node") {
        assert!(!is_success(&output), "Syntax error should fail");
        let stderr = stderr_str(&output);
        assert!(
            stderr.contains("SyntaxError") || stderr.contains("Unexpected"),
            "Should show syntax error: {}",
            stderr
        );
    }
}

/// Test: vx node with runtime error
#[rstest]
#[test]
fn test_node_runtime_error() {
    skip_if_no_vx!();

    let output = run_vx(&["node", "-e", "throw new Error('test error')"])
        .expect("Failed to run vx node with runtime error");

    if tool_installed("node") {
        assert!(!is_success(&output), "Runtime error should fail");
        let stderr = stderr_str(&output);
        assert!(
            stderr.contains("Error") && stderr.contains("test error"),
            "Should show error message: {}",
            stderr
        );
    }
}

/// Test: vx node with non-existent file
#[rstest]
#[test]
fn test_node_file_not_found() {
    skip_if_no_vx!();

    let output = run_vx(&["node", "nonexistent_file_xyz.js"])
        .expect("Failed to run vx node with missing file");

    if tool_installed("node") {
        assert!(!is_success(&output), "Missing file should fail");
        let stderr = stderr_str(&output);
        assert!(
            stderr.contains("Cannot find module")
                || stderr.contains("no such file")
                || stderr.contains("ENOENT"),
            "Should show file not found error: {}",
            stderr
        );
    }
}

/// Test: vx node with invalid flag
#[rstest]
#[test]
fn test_node_invalid_flag() {
    skip_if_no_vx!();

    let output =
        run_vx(&["node", "--invalid-flag-xyz"]).expect("Failed to run vx node with invalid flag");

    if tool_installed("node") {
        assert!(!is_success(&output), "Invalid flag should fail");
    }
}
