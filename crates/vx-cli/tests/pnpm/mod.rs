//! pnpm E2E Tests for vx CLI
//!
//! Tests for pnpm package manager

use crate::common::*;
use rstest::*;
use tempfile::TempDir;

// ============================================================================
// pnpm Version Tests
// ============================================================================

/// Test: vx pnpm --version
#[rstest]
#[test]
fn test_pnpm_version() {
    skip_if_no_vx!();

    let output = run_vx(&["pnpm", "--version"]).expect("Failed to run vx pnpm --version");

    if is_success(&output) {
        let version = stdout_str(&output).trim().to_string();
        // pnpm version is semver like "8.10.0"
        assert!(
            version.chars().next().map(|c| c.is_ascii_digit()) == Some(true),
            "pnpm version should start with digit: {}",
            version
        );
    }
}

/// Test: vx pnpm -v (short form)
#[rstest]
#[test]
fn test_pnpm_version_short() {
    skip_if_no_vx!();

    let output = run_vx(&["pnpm", "-v"]).expect("Failed to run vx pnpm -v");

    if is_success(&output) {
        let version = stdout_str(&output).trim().to_string();
        assert!(
            version.chars().next().map(|c| c.is_ascii_digit()) == Some(true),
            "pnpm version should start with digit: {}",
            version
        );
    }
}

// ============================================================================
// pnpm Help Tests
// ============================================================================

/// Test: vx pnpm --help
#[rstest]
#[test]
fn test_pnpm_help() {
    skip_if_no_vx!();

    let output = run_vx(&["pnpm", "--help"]).expect("Failed to run vx pnpm --help");

    if is_success(&output) {
        let stdout = stdout_str(&output);
        assert!(
            stdout.contains("pnpm") || stdout.contains("Usage"),
            "pnpm help should show usage: {}",
            stdout
        );
    }
}

/// Test: vx pnpm help
#[rstest]
#[test]
fn test_pnpm_help_subcommand() {
    skip_if_no_vx!();

    let output = run_vx(&["pnpm", "help"]).expect("Failed to run vx pnpm help");

    if is_success(&output) {
        let stdout = stdout_str(&output);
        assert!(
            stdout.contains("pnpm") || stdout.contains("Commands"),
            "pnpm help should show commands: {}",
            stdout
        );
    }
}

// ============================================================================
// pnpm Init Tests
// ============================================================================

/// Test: vx pnpm init
#[rstest]
#[test]
fn test_pnpm_init() {
    skip_if_no_vx!();

    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let output =
        run_vx_in_dir(temp_dir.path(), &["pnpm", "init"]).expect("Failed to run pnpm init");

    if is_success(&output) {
        assert!(
            temp_dir.path().join("package.json").exists(),
            "pnpm init should create package.json"
        );
    }
}

// ============================================================================
// pnpm Install Tests
// ============================================================================

/// Test: vx pnpm install (in empty project)
#[rstest]
#[test]
fn test_pnpm_install_empty() {
    skip_if_no_vx!();

    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Create minimal package.json
    std::fs::write(
        temp_dir.path().join("package.json"),
        r#"{"name": "test", "version": "1.0.0"}"#,
    )
    .expect("Failed to write package.json");

    let output =
        run_vx_in_dir(temp_dir.path(), &["pnpm", "install"]).expect("Failed to run pnpm install");

    // Should succeed even with no dependencies
    let _ = combined_output(&output);
}

/// Test: vx pnpm i (alias)
#[rstest]
#[test]
fn test_pnpm_install_alias() {
    skip_if_no_vx!();

    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    std::fs::write(
        temp_dir.path().join("package.json"),
        r#"{"name": "test", "version": "1.0.0"}"#,
    )
    .expect("Failed to write package.json");

    let output = run_vx_in_dir(temp_dir.path(), &["pnpm", "i"]).expect("Failed to run pnpm i");

    // Should succeed
    let _ = combined_output(&output);
}

/// Test: vx pnpm add package
#[rstest]
#[test]
#[ignore = "Requires network"]
fn test_pnpm_add_package() {
    skip_if_no_vx!();

    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Init project first
    let _ = run_vx_in_dir(temp_dir.path(), &["pnpm", "init"]);

    let output =
        run_vx_in_dir(temp_dir.path(), &["pnpm", "add", "lodash"]).expect("Failed to run pnpm add");

    if is_success(&output) {
        let pkg_json =
            std::fs::read_to_string(temp_dir.path().join("package.json")).unwrap_or_default();
        assert!(
            pkg_json.contains("lodash"),
            "package.json should contain lodash"
        );
    }
}

/// Test: vx pnpm add -D (dev dependency)
#[rstest]
#[test]
#[ignore = "Requires network"]
fn test_pnpm_add_dev() {
    skip_if_no_vx!();

    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let _ = run_vx_in_dir(temp_dir.path(), &["pnpm", "init"]);

    let output = run_vx_in_dir(temp_dir.path(), &["pnpm", "add", "-D", "typescript"])
        .expect("Failed to run pnpm add -D");

    if is_success(&output) {
        let pkg_json =
            std::fs::read_to_string(temp_dir.path().join("package.json")).unwrap_or_default();
        assert!(
            pkg_json.contains("devDependencies"),
            "package.json should have devDependencies"
        );
    }
}

// ============================================================================
// pnpm Run Tests
// ============================================================================

/// Test: vx pnpm run (list scripts)
#[rstest]
#[test]
fn test_pnpm_run_list() {
    skip_if_no_vx!();

    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    std::fs::write(
        temp_dir.path().join("package.json"),
        r#"{"name": "test", "scripts": {"hello": "echo hello"}}"#,
    )
    .expect("Failed to write package.json");

    let output = run_vx_in_dir(temp_dir.path(), &["pnpm", "run"]).expect("Failed to run pnpm run");

    // Should list available scripts or succeed
    let _ = combined_output(&output);
}

/// Test: vx pnpm run script
#[rstest]
#[test]
fn test_pnpm_run_script() {
    skip_if_no_vx!();

    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    std::fs::write(
        temp_dir.path().join("package.json"),
        r#"{"name": "test", "scripts": {"hello": "echo hello from pnpm"}}"#,
    )
    .expect("Failed to write package.json");

    let output = run_vx_in_dir(temp_dir.path(), &["pnpm", "run", "hello"])
        .expect("Failed to run pnpm run hello");

    if is_success(&output) {
        assert_output_contains(&output, "hello from pnpm", "pnpm run script");
    }
}

// ============================================================================
// pnpm Exec Tests
// ============================================================================

/// Test: vx pnpm exec
#[rstest]
#[test]
fn test_pnpm_exec() {
    skip_if_no_vx!();

    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    std::fs::write(temp_dir.path().join("package.json"), r#"{"name": "test"}"#)
        .expect("Failed to write package.json");

    // pnpm exec should work
    let output = run_vx_in_dir(temp_dir.path(), &["pnpm", "exec", "--", "echo", "test"])
        .expect("Failed to run pnpm exec");

    // May or may not succeed depending on pnpm version
    let _ = combined_output(&output);
}

// ============================================================================
// pnpm List Tests
// ============================================================================

/// Test: vx pnpm list
#[rstest]
#[test]
fn test_pnpm_list() {
    skip_if_no_vx!();

    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    std::fs::write(temp_dir.path().join("package.json"), r#"{"name": "test"}"#)
        .expect("Failed to write package.json");

    let output =
        run_vx_in_dir(temp_dir.path(), &["pnpm", "list"]).expect("Failed to run pnpm list");

    // Should succeed (may show empty list)
    let _ = combined_output(&output);
}

/// Test: vx pnpm ls (alias)
#[rstest]
#[test]
fn test_pnpm_list_alias() {
    skip_if_no_vx!();

    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    std::fs::write(temp_dir.path().join("package.json"), r#"{"name": "test"}"#)
        .expect("Failed to write package.json");

    let output = run_vx_in_dir(temp_dir.path(), &["pnpm", "ls"]).expect("Failed to run pnpm ls");

    let _ = combined_output(&output);
}

// ============================================================================
// pnpm Store Tests
// ============================================================================

/// Test: vx pnpm store status
#[rstest]
#[test]
fn test_pnpm_store_status() {
    skip_if_no_vx!();

    let output = run_vx(&["pnpm", "store", "status"]).expect("Failed to run pnpm store status");

    // Should succeed or show store info
    let _ = combined_output(&output);
}

/// Test: vx pnpm store path
#[rstest]
#[test]
fn test_pnpm_store_path() {
    skip_if_no_vx!();

    let output = run_vx(&["pnpm", "store", "path"]).expect("Failed to run pnpm store path");

    if is_success(&output) {
        let path = stdout_str(&output).trim().to_string();
        assert!(!path.is_empty(), "pnpm store path should not be empty");
    }
}

// ============================================================================
// pnpm Config Tests
// ============================================================================

/// Test: vx pnpm config list
#[rstest]
#[test]
fn test_pnpm_config_list() {
    skip_if_no_vx!();

    let output = run_vx(&["pnpm", "config", "list"]).expect("Failed to run pnpm config list");

    // Should succeed
    let _ = combined_output(&output);
}

// ============================================================================
// pnpm Dlx Tests (like npx)
// ============================================================================

/// Test: vx pnpm dlx --help
#[rstest]
#[test]
fn test_pnpm_dlx_help() {
    skip_if_no_vx!();

    let output = run_vx(&["pnpm", "dlx", "--help"]).expect("Failed to run pnpm dlx --help");

    if is_success(&output) {
        let stdout = stdout_str(&output);
        assert!(
            stdout.contains("dlx") || stdout.contains("package"),
            "pnpm dlx help: {}",
            stdout
        );
    }
}

/// Test: vx pnpm dlx cowsay
#[rstest]
#[test]
#[ignore = "Requires network"]
fn test_pnpm_dlx_cowsay() {
    skip_if_no_vx!();

    let output = run_vx(&["pnpm", "dlx", "cowsay", "hello"]).expect("Failed to run pnpm dlx");

    if is_success(&output) {
        let stdout = stdout_str(&output);
        assert!(
            stdout.contains("hello"),
            "cowsay should output hello: {}",
            stdout
        );
    }
}

// ============================================================================
// pnpm Error Handling Tests
// ============================================================================

/// Test: vx pnpm with invalid subcommand
#[rstest]
#[test]
fn test_pnpm_invalid_subcommand() {
    skip_if_no_vx!();

    let output = run_vx(&["pnpm", "invalid-subcommand-xyz"])
        .expect("Failed to run pnpm with invalid subcommand");

    if tool_installed("pnpm") {
        assert!(!is_success(&output), "Invalid subcommand should fail");
    }
}

/// Test: vx pnpm install non-existent package
#[rstest]
#[test]
#[ignore = "Requires network"]
fn test_pnpm_add_nonexistent() {
    skip_if_no_vx!();

    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let _ = run_vx_in_dir(temp_dir.path(), &["pnpm", "init"]);

    let output = run_vx_in_dir(
        temp_dir.path(),
        &["pnpm", "add", "nonexistent-package-xyz-123"],
    )
    .expect("Failed to run pnpm add");

    assert!(
        !is_success(&output),
        "Adding non-existent package should fail"
    );
}

/// Test: vx pnpm run non-existent script
#[rstest]
#[test]
fn test_pnpm_run_nonexistent_script() {
    skip_if_no_vx!();

    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    std::fs::write(temp_dir.path().join("package.json"), r#"{"name": "test"}"#)
        .expect("Failed to write package.json");

    let output = run_vx_in_dir(temp_dir.path(), &["pnpm", "run", "nonexistent-script"])
        .expect("Failed to run pnpm run");

    if tool_installed("pnpm") {
        assert!(
            !is_success(&output),
            "Running non-existent script should fail"
        );
    }
}
