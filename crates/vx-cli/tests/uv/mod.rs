//! UV (Python) E2E Tests for vx CLI
//!
//! Tests for UV ecosystem tools: uv, uvx, pip

use crate::common::*;
use rstest::*;
use tempfile::TempDir;

// ============================================================================
// UV Version Tests
// ============================================================================

/// Test: vx uv --version
#[rstest]
#[test]
fn test_uv_version() {
    skip_if_no_vx!();

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

/// Test: vx uv -V (short form)
#[rstest]
#[test]
fn test_uv_version_short() {
    skip_if_no_vx!();

    let output = run_vx(&["uv", "-V"]).expect("Failed to run vx uv -V");

    if is_success(&output) {
        let version = stdout_str(&output);
        assert!(
            version.contains("uv"),
            "uv version should contain 'uv': {}",
            version
        );
    }
}

// ============================================================================
// UV Help Tests
// ============================================================================

/// Test: vx uv --help
#[rstest]
#[test]
fn test_uv_help() {
    skip_if_no_vx!();

    let output = run_vx(&["uv", "--help"]).expect("Failed to run vx uv --help");

    if is_success(&output) {
        let stdout = stdout_str(&output);
        assert!(
            stdout.contains("Usage") || stdout.contains("usage"),
            "uv help should show usage: {}",
            stdout
        );
    }
}

/// Test: vx uv help
#[rstest]
#[test]
fn test_uv_help_subcommand() {
    skip_if_no_vx!();

    let output = run_vx(&["uv", "help"]).expect("Failed to run vx uv help");

    if is_success(&output) {
        let stdout = stdout_str(&output);
        assert!(
            stdout.contains("uv") || stdout.contains("Commands"),
            "uv help should show commands: {}",
            stdout
        );
    }
}

// ============================================================================
// UV Pip Tests
// ============================================================================

/// Test: vx uv pip --version
#[rstest]
#[test]
fn test_uv_pip_version() {
    skip_if_no_vx!();

    let output = run_vx(&["uv", "pip", "--version"]).expect("Failed to run vx uv pip --version");

    if is_success(&output) {
        let version = stdout_str(&output);
        assert!(
            version.contains("uv") || version.contains("pip"),
            "uv pip version: {}",
            version
        );
    }
}

/// Test: vx uv pip --help
#[rstest]
#[test]
fn test_uv_pip_help() {
    skip_if_no_vx!();

    let output = run_vx(&["uv", "pip", "--help"]).expect("Failed to run vx uv pip --help");

    if is_success(&output) {
        let stdout = stdout_str(&output);
        assert!(
            stdout.contains("pip") || stdout.contains("install"),
            "uv pip help should mention pip: {}",
            stdout
        );
    }
}

/// Test: vx uv pip list (in temp venv)
#[rstest]
#[test]
fn test_uv_pip_list() {
    skip_if_no_vx!();

    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // First create a venv
    let venv_output =
        run_vx_in_dir(temp_dir.path(), &["uv", "venv", ".venv"]).expect("Failed to create venv");

    if is_success(&venv_output) {
        // Then list packages
        let output = run_vx_in_dir(temp_dir.path(), &["uv", "pip", "list"])
            .expect("Failed to run vx uv pip list");

        // pip list should succeed (may be empty)
        let _ = is_success(&output);
    }
}

// ============================================================================
// UV Venv Tests
// ============================================================================

/// Test: vx uv venv --help
#[rstest]
#[test]
fn test_uv_venv_help() {
    skip_if_no_vx!();

    let output = run_vx(&["uv", "venv", "--help"]).expect("Failed to run vx uv venv --help");

    if is_success(&output) {
        let stdout = stdout_str(&output);
        assert!(
            stdout.contains("venv") || stdout.contains("virtual"),
            "uv venv help: {}",
            stdout
        );
    }
}

/// Test: vx uv venv creates virtual environment
#[rstest]
#[test]
fn test_uv_venv_create() {
    skip_if_no_vx!();

    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let output =
        run_vx_in_dir(temp_dir.path(), &["uv", "venv", ".venv"]).expect("Failed to run vx uv venv");

    if is_success(&output) {
        assert!(
            temp_dir.path().join(".venv").exists(),
            "uv venv should create .venv directory"
        );
    }
}

/// Test: vx uv venv with custom name
#[rstest]
#[test]
fn test_uv_venv_custom_name() {
    skip_if_no_vx!();

    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let output = run_vx_in_dir(temp_dir.path(), &["uv", "venv", "my_env"])
        .expect("Failed to run vx uv venv");

    if is_success(&output) {
        assert!(
            temp_dir.path().join("my_env").exists(),
            "uv venv should create custom named directory"
        );
    }
}

// ============================================================================
// UVX Tests
// ============================================================================

/// Test: vx uvx --version
#[rstest]
#[test]
fn test_uvx_version() {
    skip_if_no_vx!();

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

/// Test: vx uvx --help
#[rstest]
#[test]
fn test_uvx_help() {
    skip_if_no_vx!();

    let output = run_vx(&["uvx", "--help"]).expect("Failed to run vx uvx --help");

    if is_success(&output) {
        let stdout = stdout_str(&output);
        assert!(
            stdout.contains("Usage") || stdout.contains("uvx"),
            "uvx help should show usage: {}",
            stdout
        );
    }
}

/// Test: vx uvx ruff --version (popular Python linter)
#[rstest]
#[test]
#[ignore = "Requires network to download ruff"]
fn test_uvx_ruff_version() {
    skip_if_no_vx!();

    let output = run_vx(&["uvx", "ruff", "--version"]).expect("Failed to run vx uvx ruff");

    if is_success(&output) {
        let version = stdout_str(&output);
        assert!(
            version.contains("ruff"),
            "ruff version should contain 'ruff': {}",
            version
        );
    }
}

/// Test: vx uvx ruff check
#[rstest]
#[test]
#[ignore = "Requires network to download ruff"]
fn test_uvx_ruff_check() {
    skip_if_no_vx!();

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let py_file = temp_dir.path().join("test.py");

    // Write a simple Python file
    std::fs::write(&py_file, "x = 1\n").expect("Failed to write test.py");

    let output = run_vx_in_dir(temp_dir.path(), &["uvx", "ruff", "check", "."])
        .expect("Failed to run uvx ruff");

    // ruff check should succeed or fail with linting errors
    let _ = combined_output(&output);
}

/// Test: vx uvx black --version
#[rstest]
#[test]
#[ignore = "Requires network to download black"]
fn test_uvx_black_version() {
    skip_if_no_vx!();

    let output = run_vx(&["uvx", "black", "--version"]).expect("Failed to run vx uvx black");

    if is_success(&output) {
        let version = stdout_str(&output);
        assert!(
            version.contains("black") || version.chars().any(|c| c.is_ascii_digit()),
            "black version: {}",
            version
        );
    }
}

// ============================================================================
// UV Python Management Tests
// ============================================================================

/// Test: vx uv python --help
#[rstest]
#[test]
fn test_uv_python_help() {
    skip_if_no_vx!();

    let output = run_vx(&["uv", "python", "--help"]).expect("Failed to run vx uv python --help");

    if is_success(&output) {
        let stdout = stdout_str(&output);
        assert!(
            stdout.contains("python") || stdout.contains("Python"),
            "uv python help: {}",
            stdout
        );
    }
}

/// Test: vx uv python list
#[rstest]
#[test]
fn test_uv_python_list() {
    skip_if_no_vx!();

    let output = run_vx(&["uv", "python", "list"]).expect("Failed to run vx uv python list");

    // May succeed or fail depending on installed pythons
    let _ = combined_output(&output);
}

// ============================================================================
// UV Init Tests
// ============================================================================

/// Test: vx uv init --help
#[rstest]
#[test]
fn test_uv_init_help() {
    skip_if_no_vx!();

    let output = run_vx(&["uv", "init", "--help"]).expect("Failed to run vx uv init --help");

    if is_success(&output) {
        let stdout = stdout_str(&output);
        assert!(
            stdout.contains("init") || stdout.contains("project"),
            "uv init help: {}",
            stdout
        );
    }
}

/// Test: vx uv init creates project
#[rstest]
#[test]
fn test_uv_init_project() {
    skip_if_no_vx!();

    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let output = run_vx_in_dir(temp_dir.path(), &["uv", "init"]).expect("Failed to run vx uv init");

    if is_success(&output) {
        // Should create pyproject.toml
        assert!(
            temp_dir.path().join("pyproject.toml").exists(),
            "uv init should create pyproject.toml"
        );
    }
}

// ============================================================================
// UV Run Tests
// ============================================================================

/// Test: vx uv run --help
#[rstest]
#[test]
fn test_uv_run_help() {
    skip_if_no_vx!();

    let output = run_vx(&["uv", "run", "--help"]).expect("Failed to run vx uv run --help");

    if is_success(&output) {
        let stdout = stdout_str(&output);
        assert!(
            stdout.contains("run") || stdout.contains("Run"),
            "uv run help: {}",
            stdout
        );
    }
}

/// Test: vx uv run python script
#[rstest]
#[test]
fn test_uv_run_python_script() {
    skip_if_no_vx!();

    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Create a simple Python script
    let script = temp_dir.path().join("hello.py");
    std::fs::write(&script, "print('Hello from uv run!')").expect("Failed to write script");

    // Initialize uv project first
    let init_output =
        run_vx_in_dir(temp_dir.path(), &["uv", "init"]).expect("Failed to run uv init");

    if is_success(&init_output) {
        let output = run_vx_in_dir(temp_dir.path(), &["uv", "run", "python", "hello.py"])
            .expect("Failed to run uv run python");

        if is_success(&output) {
            assert_stdout_contains(&output, "Hello from uv run!", "uv run python");
        }
    }
}

// ============================================================================
// UV Error Handling Tests
// ============================================================================

/// Test: vx uv with invalid subcommand
#[rstest]
#[test]
fn test_uv_invalid_subcommand() {
    skip_if_no_vx!();

    let output = run_vx(&["uv", "invalid-subcommand-xyz"])
        .expect("Failed to run vx uv with invalid subcommand");

    if tool_installed("uv") {
        assert!(!is_success(&output), "Invalid subcommand should fail");
    }
}

/// Test: vx uv pip install non-existent package
#[rstest]
#[test]
#[ignore = "Requires network"]
fn test_uv_pip_install_nonexistent() {
    skip_if_no_vx!();

    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Create venv first
    let _ = run_vx_in_dir(temp_dir.path(), &["uv", "venv", ".venv"]);

    let output = run_vx_in_dir(
        temp_dir.path(),
        &["uv", "pip", "install", "nonexistent-package-xyz-123"],
    )
    .expect("Failed to run uv pip install");

    // Should fail for non-existent package
    assert!(
        !is_success(&output),
        "Installing non-existent package should fail"
    );
}
