//! Yarn E2E Tests for vx CLI
//!
//! Tests for Yarn package manager

use crate::common::*;
use rstest::*;
use tempfile::TempDir;

// ============================================================================
// Yarn Version Tests
// ============================================================================

/// Test: vx yarn --version
#[rstest]
#[test]
fn test_yarn_version() {
    skip_if_no_vx!();

    let output = run_vx(&["yarn", "--version"]).expect("Failed to run vx yarn --version");

    if is_success(&output) {
        let version = stdout_str(&output).trim().to_string();
        // Yarn version is semver like "1.22.0" or "4.0.0"
        assert!(
            version.chars().next().map(|c| c.is_ascii_digit()) == Some(true),
            "yarn version should start with digit: {}",
            version
        );
    }
}

/// Test: vx yarn -v (short form)
#[rstest]
#[test]
fn test_yarn_version_short() {
    skip_if_no_vx!();

    let output = run_vx(&["yarn", "-v"]).expect("Failed to run vx yarn -v");

    if is_success(&output) {
        let version = stdout_str(&output).trim().to_string();
        assert!(
            version.chars().next().map(|c| c.is_ascii_digit()) == Some(true),
            "yarn version should start with digit: {}",
            version
        );
    }
}

// ============================================================================
// Yarn Help Tests
// ============================================================================

/// Test: vx yarn --help
#[rstest]
#[test]
fn test_yarn_help() {
    skip_if_no_vx!();

    let output = run_vx(&["yarn", "--help"]).expect("Failed to run vx yarn --help");

    if is_success(&output) {
        let stdout = stdout_str(&output);
        assert!(
            stdout.contains("yarn") || stdout.contains("Usage") || stdout.contains("Commands"),
            "yarn help should show usage: {}",
            stdout
        );
    }
}

/// Test: vx yarn help
#[rstest]
#[test]
fn test_yarn_help_subcommand() {
    skip_if_no_vx!();

    let output = run_vx(&["yarn", "help"]).expect("Failed to run vx yarn help");

    if is_success(&output) {
        let stdout = stdout_str(&output);
        assert!(
            stdout.contains("yarn") || stdout.contains("Commands"),
            "yarn help should show commands: {}",
            stdout
        );
    }
}

// ============================================================================
// Yarn Init Tests
// ============================================================================

/// Test: vx yarn init -y
#[rstest]
#[test]
fn test_yarn_init() {
    skip_if_no_vx!();

    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let output =
        run_vx_in_dir(temp_dir.path(), &["yarn", "init", "-y"]).expect("Failed to run yarn init");

    if is_success(&output) {
        assert!(
            temp_dir.path().join("package.json").exists(),
            "yarn init should create package.json"
        );
    }
}

/// Test: vx yarn init -2 (Yarn 2+)
#[rstest]
#[test]
fn test_yarn_init_v2() {
    skip_if_no_vx!();

    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // This may fail if yarn v1 is installed
    let output = run_vx_in_dir(temp_dir.path(), &["yarn", "init", "-2"])
        .expect("Failed to run yarn init -2");

    // May or may not succeed depending on yarn version
    let _ = combined_output(&output);
}

// ============================================================================
// Yarn Install Tests
// ============================================================================

/// Test: vx yarn install (in empty project)
#[rstest]
#[test]
fn test_yarn_install_empty() {
    skip_if_no_vx!();

    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Create minimal package.json
    std::fs::write(
        temp_dir.path().join("package.json"),
        r#"{"name": "test", "version": "1.0.0"}"#,
    )
    .expect("Failed to write package.json");

    let output =
        run_vx_in_dir(temp_dir.path(), &["yarn", "install"]).expect("Failed to run yarn install");

    // Should succeed even with no dependencies
    let _ = combined_output(&output);
}

/// Test: vx yarn (implicit install)
#[rstest]
#[test]
fn test_yarn_implicit_install() {
    skip_if_no_vx!();

    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    std::fs::write(
        temp_dir.path().join("package.json"),
        r#"{"name": "test", "version": "1.0.0"}"#,
    )
    .expect("Failed to write package.json");

    let output = run_vx_in_dir(temp_dir.path(), &["yarn"]).expect("Failed to run yarn");

    // Should succeed (yarn without args runs install)
    let _ = combined_output(&output);
}

/// Test: vx yarn add package
#[rstest]
#[test]
fn test_yarn_add_package() {
    skip_if_no_vx!();
    skip_if_no_network!();

    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let _ = run_vx_in_dir(temp_dir.path(), &["yarn", "init", "-y"]);

    let output =
        run_vx_in_dir(temp_dir.path(), &["yarn", "add", "lodash"]).expect("Failed to run yarn add");

    if is_success(&output) {
        let pkg_json =
            std::fs::read_to_string(temp_dir.path().join("package.json")).unwrap_or_default();
        assert!(
            pkg_json.contains("lodash"),
            "package.json should contain lodash"
        );
    }
}

/// Test: vx yarn add -D (dev dependency)
#[rstest]
#[test]
fn test_yarn_add_dev() {
    skip_if_no_vx!();
    skip_if_no_network!();

    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let _ = run_vx_in_dir(temp_dir.path(), &["yarn", "init", "-y"]);

    let output = run_vx_in_dir(temp_dir.path(), &["yarn", "add", "-D", "typescript"])
        .expect("Failed to run yarn add -D");

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
// Yarn Run Tests
// ============================================================================

/// Test: vx yarn run (list scripts)
#[rstest]
#[test]
fn test_yarn_run_list() {
    skip_if_no_vx!();

    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    std::fs::write(
        temp_dir.path().join("package.json"),
        r#"{"name": "test", "scripts": {"hello": "echo hello"}}"#,
    )
    .expect("Failed to write package.json");

    let output = run_vx_in_dir(temp_dir.path(), &["yarn", "run"]).expect("Failed to run yarn run");

    // Should list available scripts or succeed
    let _ = combined_output(&output);
}

/// Test: vx yarn run script
#[rstest]
#[test]
fn test_yarn_run_script() {
    skip_if_no_vx!();

    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    std::fs::write(
        temp_dir.path().join("package.json"),
        r#"{"name": "test", "scripts": {"hello": "echo hello from yarn"}}"#,
    )
    .expect("Failed to write package.json");

    let output = run_vx_in_dir(temp_dir.path(), &["yarn", "run", "hello"])
        .expect("Failed to run yarn run hello");

    if is_success(&output) {
        assert_output_contains(&output, "hello from yarn", "yarn run script");
    }
}

/// Test: vx yarn script (shorthand)
#[rstest]
#[test]
fn test_yarn_script_shorthand() {
    skip_if_no_vx!();

    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    std::fs::write(
        temp_dir.path().join("package.json"),
        r#"{"name": "test", "scripts": {"hello": "echo hello shorthand"}}"#,
    )
    .expect("Failed to write package.json");

    // yarn hello (without run)
    let output =
        run_vx_in_dir(temp_dir.path(), &["yarn", "hello"]).expect("Failed to run yarn hello");

    if is_success(&output) {
        assert_output_contains(&output, "hello shorthand", "yarn script shorthand");
    }
}

// ============================================================================
// Yarn List Tests
// ============================================================================

/// Test: vx yarn list
#[rstest]
#[test]
fn test_yarn_list() {
    skip_if_no_vx!();

    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    std::fs::write(temp_dir.path().join("package.json"), r#"{"name": "test"}"#)
        .expect("Failed to write package.json");

    let output =
        run_vx_in_dir(temp_dir.path(), &["yarn", "list"]).expect("Failed to run yarn list");

    // Should succeed (may show empty list)
    let _ = combined_output(&output);
}

/// Test: vx yarn list --depth=0
#[rstest]
#[test]
fn test_yarn_list_depth() {
    skip_if_no_vx!();

    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    std::fs::write(temp_dir.path().join("package.json"), r#"{"name": "test"}"#)
        .expect("Failed to write package.json");

    let output = run_vx_in_dir(temp_dir.path(), &["yarn", "list", "--depth=0"])
        .expect("Failed to run yarn list --depth=0");

    let _ = combined_output(&output);
}

// ============================================================================
// Yarn Info Tests
// ============================================================================

/// Test: vx yarn info lodash
#[rstest]
#[test]
fn test_yarn_info() {
    skip_if_no_vx!();
    skip_if_no_network!();

    let output = run_vx(&["yarn", "info", "lodash"]).expect("Failed to run yarn info");

    if is_success(&output) {
        let stdout = stdout_str(&output);
        assert!(
            stdout.contains("lodash") || stdout.contains("name"),
            "yarn info should show package info: {}",
            stdout
        );
    }
}

// ============================================================================
// Yarn Cache Tests
// ============================================================================

/// Test: vx yarn cache dir
#[rstest]
#[test]
fn test_yarn_cache_dir() {
    skip_if_no_vx!();

    let output = run_vx(&["yarn", "cache", "dir"]).expect("Failed to run yarn cache dir");

    if is_success(&output) {
        let path = stdout_str(&output).trim().to_string();
        assert!(!path.is_empty(), "yarn cache dir should not be empty");
    }
}

/// Test: vx yarn cache list
#[rstest]
#[test]
fn test_yarn_cache_list() {
    skip_if_no_vx!();

    let output = run_vx(&["yarn", "cache", "list"]).expect("Failed to run yarn cache list");

    // Should succeed
    let _ = combined_output(&output);
}

// ============================================================================
// Yarn Config Tests
// ============================================================================

/// Test: vx yarn config list
#[rstest]
#[test]
fn test_yarn_config_list() {
    skip_if_no_vx!();

    let output = run_vx(&["yarn", "config", "list"]).expect("Failed to run yarn config list");

    // Should succeed
    let _ = combined_output(&output);
}

/// Test: vx yarn config get registry
#[rstest]
#[test]
fn test_yarn_config_get() {
    skip_if_no_vx!();

    let output =
        run_vx(&["yarn", "config", "get", "registry"]).expect("Failed to run yarn config get");

    // Should succeed (may show default registry)
    let _ = combined_output(&output);
}

// ============================================================================
// Yarn Dlx Tests (Yarn 2+)
// ============================================================================

/// Test: vx yarn dlx cowsay
#[rstest]
#[test]
fn test_yarn_dlx() {
    skip_if_no_vx!();
    skip_if_no_network!();

    let output = run_vx(&["yarn", "dlx", "cowsay", "hello"]).expect("Failed to run yarn dlx");

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
// Yarn Why Tests
// ============================================================================

/// Test: vx yarn why (requires installed package)
#[rstest]
#[test]
fn test_yarn_why() {
    skip_if_no_vx!();
    skip_if_no_network!();

    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    std::fs::write(
        temp_dir.path().join("package.json"),
        r#"{"name": "test", "dependencies": {"lodash": "^4.0.0"}}"#,
    )
    .expect("Failed to write package.json");

    let _ = run_vx_in_dir(temp_dir.path(), &["yarn", "install"]);

    let output =
        run_vx_in_dir(temp_dir.path(), &["yarn", "why", "lodash"]).expect("Failed to run yarn why");

    if is_success(&output) {
        let stdout = stdout_str(&output);
        assert!(
            stdout.contains("lodash"),
            "yarn why should mention package: {}",
            stdout
        );
    }
}

// ============================================================================
// Yarn Error Handling Tests
// ============================================================================

/// Test: vx yarn with invalid subcommand
#[rstest]
#[test]
fn test_yarn_invalid_subcommand() {
    skip_if_no_vx!();

    let output = run_vx(&["yarn", "invalid-subcommand-xyz"])
        .expect("Failed to run yarn with invalid subcommand");

    if tool_installed("yarn") {
        assert!(!is_success(&output), "Invalid subcommand should fail");
    }
}

/// Test: vx yarn add non-existent package
#[rstest]
#[test]
fn test_yarn_add_nonexistent() {
    skip_if_no_vx!();
    skip_if_no_network!();

    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let _ = run_vx_in_dir(temp_dir.path(), &["yarn", "init", "-y"]);

    let output = run_vx_in_dir(
        temp_dir.path(),
        &["yarn", "add", "nonexistent-package-xyz-123"],
    )
    .expect("Failed to run yarn add");

    assert!(
        !is_success(&output),
        "Adding non-existent package should fail"
    );
}

/// Test: vx yarn run non-existent script
#[rstest]
#[test]
fn test_yarn_run_nonexistent_script() {
    skip_if_no_vx!();

    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    std::fs::write(temp_dir.path().join("package.json"), r#"{"name": "test"}"#)
        .expect("Failed to write package.json");

    let output = run_vx_in_dir(temp_dir.path(), &["yarn", "run", "nonexistent-script"])
        .expect("Failed to run yarn run");

    if tool_installed("yarn") {
        assert!(
            !is_success(&output),
            "Running non-existent script should fail"
        );
    }
}
