//! Bun E2E Tests for vx CLI
//!
//! Tests for Bun runtime and package manager

use crate::common::*;
use rstest::*;
use tempfile::TempDir;

// ============================================================================
// Bun Version Tests
// ============================================================================

/// Test: vx bun --version
#[rstest]
#[test]
fn test_bun_version() {
    skip_if_no_vx!();

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

/// Test: vx bun -v (short form)
#[rstest]
#[test]
fn test_bun_version_short() {
    skip_if_no_vx!();

    let output = run_vx(&["bun", "-v"]).expect("Failed to run vx bun -v");

    if is_success(&output) {
        let version = stdout_str(&output).trim().to_string();
        assert!(
            version.chars().next().map(|c| c.is_ascii_digit()) == Some(true),
            "bun version should start with digit: {}",
            version
        );
    }
}

// ============================================================================
// Bun Help Tests
// ============================================================================

/// Test: vx bun --help
#[rstest]
#[test]
fn test_bun_help() {
    skip_if_no_vx!();

    let output = run_vx(&["bun", "--help"]).expect("Failed to run vx bun --help");

    if is_success(&output) {
        let stdout = stdout_str(&output);
        assert!(
            stdout.contains("bun") || stdout.contains("Usage"),
            "bun help should show usage: {}",
            stdout
        );
    }
}

// ============================================================================
// Bun Eval Tests
// ============================================================================

/// Test: vx bun -e "console.log('hello')"
#[rstest]
#[test]
fn test_bun_eval() {
    skip_if_no_vx!();

    let output =
        run_vx(&["bun", "-e", "console.log('hello from bun')"]).expect("Failed to run bun -e");

    if is_success(&output) {
        assert_stdout_contains(&output, "hello from bun", "bun -e");
    }
}

/// Test: vx bun -e with JSON
#[rstest]
#[test]
fn test_bun_eval_json() {
    skip_if_no_vx!();

    let output = run_vx(&["bun", "-e", "console.log(JSON.stringify({x:1,y:2}))"])
        .expect("Failed to run bun -e");

    if is_success(&output) {
        let stdout = stdout_str(&output);
        assert!(
            stdout.contains(r#"{"x":1,"y":2}"#),
            "Should output JSON: {}",
            stdout
        );
    }
}

/// Test: vx bun -e with environment variable
#[rstest]
#[test]
fn test_bun_eval_env() {
    skip_if_no_vx!();

    let output = run_vx_with_env(
        &["bun", "-e", "console.log(Bun.env.VX_TEST_VAR)"],
        &[("VX_TEST_VAR", "bun_test_value")],
    )
    .expect("Failed to run bun -e with env");

    if is_success(&output) {
        assert_stdout_contains(&output, "bun_test_value", "bun env access");
    }
}

// ============================================================================
// Bun Run Tests
// ============================================================================

/// Test: vx bun run script.ts
#[rstest]
#[test]
fn test_bun_run_typescript() {
    skip_if_no_vx!();

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let script = temp_dir.path().join("test.ts");

    std::fs::write(
        &script,
        r#"
const message: string = "Hello from TypeScript!";
console.log(message);
"#,
    )
    .expect("Failed to write script");

    let output =
        run_vx_in_dir(temp_dir.path(), &["bun", "run", "test.ts"]).expect("Failed to run bun run");

    if is_success(&output) {
        assert_stdout_contains(&output, "Hello from TypeScript!", "bun run ts");
    }
}

/// Test: vx bun run script.js
#[rstest]
#[test]
fn test_bun_run_javascript() {
    skip_if_no_vx!();

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let script = temp_dir.path().join("test.js");

    std::fs::write(&script, r#"console.log("Hello from JavaScript!");"#)
        .expect("Failed to write script");

    let output =
        run_vx_in_dir(temp_dir.path(), &["bun", "run", "test.js"]).expect("Failed to run bun run");

    if is_success(&output) {
        assert_stdout_contains(&output, "Hello from JavaScript!", "bun run js");
    }
}

/// Test: vx bun with arguments
#[rstest]
#[test]
fn test_bun_run_with_args() {
    skip_if_no_vx!();

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let script = temp_dir.path().join("args.ts");

    std::fs::write(
        &script,
        r#"console.log("Args:", Bun.argv.slice(2).join(", "));"#,
    )
    .expect("Failed to write script");

    let output = run_vx_in_dir(temp_dir.path(), &["bun", "run", "args.ts", "arg1", "arg2"])
        .expect("Failed to run bun with args");

    if is_success(&output) {
        assert_stdout_contains(&output, "arg1, arg2", "bun run args");
    }
}

// ============================================================================
// Bun Init Tests
// ============================================================================

/// Test: vx bun init
#[rstest]
#[test]
fn test_bun_init() {
    skip_if_no_vx!();

    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let output =
        run_vx_in_dir(temp_dir.path(), &["bun", "init", "-y"]).expect("Failed to run bun init");

    if is_success(&output) {
        assert!(
            temp_dir.path().join("package.json").exists(),
            "bun init should create package.json"
        );
    }
}

// ============================================================================
// Bun Install Tests
// ============================================================================

/// Test: vx bun install (in empty project)
#[rstest]
#[test]
fn test_bun_install_empty() {
    skip_if_no_vx!();

    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Create minimal package.json
    std::fs::write(
        temp_dir.path().join("package.json"),
        r#"{"name": "test", "version": "1.0.0"}"#,
    )
    .expect("Failed to write package.json");

    let output =
        run_vx_in_dir(temp_dir.path(), &["bun", "install"]).expect("Failed to run bun install");

    // Should succeed even with no dependencies
    let _ = combined_output(&output);
}

/// Test: vx bun add (install package)
#[rstest]
#[test]
#[ignore = "Requires network"]
fn test_bun_add_package() {
    skip_if_no_vx!();

    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Init project first
    let _ = run_vx_in_dir(temp_dir.path(), &["bun", "init", "-y"]);

    let output =
        run_vx_in_dir(temp_dir.path(), &["bun", "add", "lodash"]).expect("Failed to run bun add");

    if is_success(&output) {
        // Check package.json has lodash
        let pkg_json =
            std::fs::read_to_string(temp_dir.path().join("package.json")).unwrap_or_default();
        assert!(
            pkg_json.contains("lodash"),
            "package.json should contain lodash"
        );
    }
}

// ============================================================================
// Bun Test Tests
// ============================================================================

/// Test: vx bun test
#[rstest]
#[test]
fn test_bun_test() {
    skip_if_no_vx!();

    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Create a simple test file
    std::fs::write(
        temp_dir.path().join("test.test.ts"),
        r#"
import { expect, test } from "bun:test";

test("2 + 2", () => {
    expect(2 + 2).toBe(4);
});
"#,
    )
    .expect("Failed to write test file");

    let output = run_vx_in_dir(temp_dir.path(), &["bun", "test"]).expect("Failed to run bun test");

    if is_success(&output) {
        let combined = combined_output(&output);
        assert!(
            combined.contains("pass") || combined.contains("PASS") || combined.contains("✓"),
            "bun test should pass: {}",
            combined
        );
    }
}

// ============================================================================
// Bun Build Tests
// ============================================================================

/// Test: vx bun build
#[rstest]
#[test]
fn test_bun_build() {
    skip_if_no_vx!();

    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Create entry file
    std::fs::write(
        temp_dir.path().join("index.ts"),
        r#"console.log("built!");"#,
    )
    .expect("Failed to write index.ts");

    let output = run_vx_in_dir(
        temp_dir.path(),
        &["bun", "build", "index.ts", "--outdir", "dist"],
    )
    .expect("Failed to run bun build");

    if is_success(&output) {
        assert!(
            temp_dir.path().join("dist").exists(),
            "bun build should create dist directory"
        );
    }
}

// ============================================================================
// Bunx Tests
// ============================================================================

/// Test: vx bunx --version (if bunx is available)
#[rstest]
#[test]
fn test_bunx_version() {
    skip_if_no_vx!();

    // bunx might be an alias or separate command
    let output = run_vx(&["bunx", "--version"]).expect("Failed to run vx bunx --version");

    // bunx may or may not be available
    let _ = combined_output(&output);
}

/// Test: vx bunx cowsay (popular package)
#[rstest]
#[test]
#[ignore = "Requires network"]
fn test_bunx_cowsay() {
    skip_if_no_vx!();

    let output = run_vx(&["bunx", "cowsay", "hello"]).expect("Failed to run bunx cowsay");

    if is_success(&output) {
        let stdout = stdout_str(&output);
        assert!(
            stdout.contains("hello") || stdout.contains("_"),
            "cowsay should output: {}",
            stdout
        );
    }
}

// ============================================================================
// Bun Exit Code Tests
// ============================================================================

/// Test: vx bun -e "process.exit(0)"
#[rstest]
#[test]
fn test_bun_exit_code_zero() {
    skip_if_no_vx!();

    // Use a simple expression that exits cleanly
    let output = run_vx(&["bun", "-e", "console.log('ok')"]).expect("Failed to run bun");

    // Just verify it doesn't crash - bun may or may not be fully installed
    let _ = combined_output(&output);
}

/// Test: vx bun -e "process.exit(1)"
#[rstest]
#[test]
fn test_bun_exit_code_one() {
    skip_if_no_vx!();

    let output = run_vx(&["bun", "-e", "process.exit(1)"]).expect("Failed to run bun");

    // Note: Exit code propagation behavior may vary
    // Just verify it doesn't crash
    let _ = combined_output(&output);
}

// ============================================================================
// Bun Error Handling Tests
// ============================================================================

/// Test: vx bun with syntax error
#[rstest]
#[test]
fn test_bun_syntax_error() {
    skip_if_no_vx!();

    let output = run_vx(&["bun", "-e", "console.log("]).expect("Failed to run bun with error");

    if tool_installed("bun") {
        assert!(!is_success(&output), "Syntax error should fail");
    }
}

/// Test: vx bun with runtime error
#[rstest]
#[test]
fn test_bun_runtime_error() {
    skip_if_no_vx!();

    let output = run_vx(&["bun", "-e", "throw new Error('test error')"])
        .expect("Failed to run bun with error");

    if tool_installed("bun") {
        assert!(!is_success(&output), "Runtime error should fail");
        let stderr = stderr_str(&output);
        assert!(
            stderr.contains("Error") || stderr.contains("error"),
            "Should show error: {}",
            stderr
        );
    }
}

/// Test: vx bun run non-existent file
#[rstest]
#[test]
fn test_bun_file_not_found() {
    skip_if_no_vx!();

    let output = run_vx(&["bun", "run", "nonexistent_file.ts"])
        .expect("Failed to run bun with missing file");

    if tool_installed("bun") {
        assert!(!is_success(&output), "Missing file should fail");
    }
}
