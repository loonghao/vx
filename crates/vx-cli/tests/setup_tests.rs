//! Tests for setup command functionality
//!
//! These tests verify the vx add/rm-tool/update-tool commands
//! correctly handle TOML value types (booleans, numbers, strings).

use rstest::rstest;
use std::fs;
use tempfile::TempDir;

#[macro_use]
mod common;
use common::{combined_output, is_success, run_vx_in_dir};

// ============================================================================
// TOML Value Preservation Tests
// ============================================================================

/// Test: vx add preserves boolean values (not quoted as strings)
#[rstest]
#[test]
fn test_add_preserves_boolean_true() {
    skip_if_no_vx!();

    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Create initial config with boolean setting
    fs::write(
        temp_dir.path().join("vx.toml"),
        r#"[tools]
python = "3.11"

[settings]
auto_install = true
"#,
    )
    .expect("Failed to write vx.toml");

    let output = run_vx_in_dir(temp_dir.path(), &["add", "uv", "--version", "latest"])
        .expect("Failed to run vx add");

    if is_success(&output) {
        let content =
            fs::read_to_string(temp_dir.path().join("vx.toml")).expect("Failed to read vx.toml");

        // Boolean should NOT be quoted
        assert!(
            !content.contains(r#"auto_install = "true""#),
            "Boolean true should not be quoted: {}",
            content
        );
        assert!(
            content.contains("auto_install = true"),
            "Boolean true should be preserved: {}",
            content
        );
    }
}

/// Test: vx add preserves boolean false values
#[rstest]
#[test]
fn test_add_preserves_boolean_false() {
    skip_if_no_vx!();

    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    fs::write(
        temp_dir.path().join("vx.toml"),
        r#"[tools]
python = "3.11"

[settings]
parallel_install = false
"#,
    )
    .expect("Failed to write vx.toml");

    let output = run_vx_in_dir(temp_dir.path(), &["add", "go", "--version", "latest"])
        .expect("Failed to run vx add");

    if is_success(&output) {
        let content =
            fs::read_to_string(temp_dir.path().join("vx.toml")).expect("Failed to read vx.toml");

        assert!(
            !content.contains(r#"parallel_install = "false""#),
            "Boolean false should not be quoted: {}",
            content
        );
        assert!(
            content.contains("parallel_install = false"),
            "Boolean false should be preserved: {}",
            content
        );
    }
}

/// Test: vx add preserves string values with quotes
#[rstest]
#[test]
fn test_add_preserves_string_values() {
    skip_if_no_vx!();

    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    fs::write(
        temp_dir.path().join("vx.toml"),
        r#"[tools]
python = "3.11"

[settings]
cache_duration = "7d"
"#,
    )
    .expect("Failed to write vx.toml");

    let output = run_vx_in_dir(temp_dir.path(), &["add", "node", "--version", "20"])
        .expect("Failed to run vx add");

    if is_success(&output) {
        let content =
            fs::read_to_string(temp_dir.path().join("vx.toml")).expect("Failed to read vx.toml");

        // String should be quoted
        assert!(
            content.contains(r#"cache_duration = "7d""#),
            "String value should be quoted: {}",
            content
        );
    }
}

/// Test: Config remains parseable after vx add with boolean settings
#[rstest]
#[test]
fn test_config_parseable_after_add() {
    skip_if_no_vx!();

    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    fs::write(
        temp_dir.path().join("vx.toml"),
        r#"[tools]
python = "3.11"

[settings]
auto_install = true
cache_duration = "7d"
"#,
    )
    .expect("Failed to write vx.toml");

    // Add a tool
    let output = run_vx_in_dir(temp_dir.path(), &["add", "uv", "--version", "latest"])
        .expect("Failed to run vx add");

    if is_success(&output) {
        // Verify config can still be parsed
        let output2 = run_vx_in_dir(temp_dir.path(), &["config", "show"])
            .expect("Failed to run vx config show");

        assert!(
            is_success(&output2),
            "Config should be parseable after add: {}",
            combined_output(&output2)
        );
    }
}

/// Test: Multiple tools can be added without corrupting boolean settings
#[rstest]
#[test]
fn test_multiple_adds_preserve_settings() {
    skip_if_no_vx!();

    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    fs::write(
        temp_dir.path().join("vx.toml"),
        r#"[tools]
python = "3.11"

[settings]
auto_install = true
parallel_install = false
cache_duration = "7d"
"#,
    )
    .expect("Failed to write vx.toml");

    // Add first tool
    let _ = run_vx_in_dir(temp_dir.path(), &["add", "uv", "--version", "latest"]);

    // Add second tool
    let _ = run_vx_in_dir(temp_dir.path(), &["add", "node", "--version", "20"]);

    // Verify config is still valid
    let output =
        run_vx_in_dir(temp_dir.path(), &["config", "show"]).expect("Failed to run vx config show");

    let combined = combined_output(&output);
    assert!(
        is_success(&output) || !combined.contains("invalid type"),
        "Config should remain valid after multiple adds: {}",
        combined
    );

    // Check the actual content
    let content =
        fs::read_to_string(temp_dir.path().join("vx.toml")).expect("Failed to read vx.toml");

    assert!(
        content.contains("auto_install = true"),
        "auto_install should be preserved: {}",
        content
    );
    assert!(
        content.contains("parallel_install = false"),
        "parallel_install should be preserved: {}",
        content
    );
}
