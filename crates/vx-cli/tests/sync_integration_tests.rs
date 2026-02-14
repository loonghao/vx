//! Integration test for sync command with tool installation
//!
//! This test verifies that sync correctly installs tools with different version formats.

use std::fs;
use tempfile::TempDir;

#[macro_use]
mod common;
use common::run_vx_in_dir;

/// Test that sync correctly handles tool@version format
#[test]
fn test_sync_with_tool_at_version_format() {
    skip_if_no_vx!();

    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Create a vx.toml with tools
    fs::write(
        temp_dir.path().join("vx.toml"),
        r#"[tools]
just = "latest"
node = "20"
python = "3.11"
"#,
    )
    .expect("Failed to write vx.toml");

    // Run sync in check mode (no actual installation)
    let output =
        run_vx_in_dir(temp_dir.path(), &["sync", "--check"]).expect("Failed to run vx sync");

    // Verify it doesn't fail with parsing errors
    // Note: Actual installation will fail because tools aren't installed,
    // but it shouldn't fail with "Tool 'version_number' is not supported"
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should NOT see errors like "Tool '3.11' is not supported"
    // or "Tool '20' is not supported"
    assert!(
        !stdout.contains("Tool '3.11' is not supported")
            && !stderr.contains("Tool '3.11' is not supported"),
        "Should not see 'Tool 3.11 is not supported' error. Output:\nstdout:\n{}\nstderr:\n{}",
        stdout,
        stderr
    );

    assert!(
        !stdout.contains("Tool '20' is not supported")
            && !stderr.contains("Tool '20' is not supported"),
        "Should not see 'Tool 20 is not supported' error. Output:\nstdout:\n{}\nstderr:\n{}",
        stdout,
        stderr
    );
}

/// Test that sync handles version formats with dots
#[test]
fn test_sync_with_dotted_versions() {
    skip_if_no_vx!();

    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Create a vx.toml with dotted version numbers
    fs::write(
        temp_dir.path().join("vx.toml"),
        r#"[tools]
python = "3.11"
rust = "1.90.0"
"#,
    )
    .expect("Failed to write vx.toml");

    // Run sync in check mode
    let output =
        run_vx_in_dir(temp_dir.path(), &["sync", "--check"]).expect("Failed to run vx sync");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should NOT see version numbers as tool names
    assert!(
        !stdout.contains("Tool '3.11' is not supported")
            && !stderr.contains("Tool '3.11' is not supported"),
        "Should not treat '3.11' as a tool name"
    );

    assert!(
        !stdout.contains("Tool '1.90.0' is not supported")
            && !stderr.contains("Tool '1.90.0' is not supported"),
        "Should not treat '1.90.0' as a tool name"
    );
}

/// Test that sync skips tools restricted to a different OS.
///
/// When a tool has `os = ["windows"]` in vx.toml but we're running on a
/// different platform, sync --check should NOT attempt to install it.
#[test]
fn test_sync_skips_platform_specific_tools() {
    skip_if_no_vx!();

    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Create a vx.toml with a platform-specific tool that doesn't match current OS
    let non_current_os = if cfg!(target_os = "windows") {
        "linux"
    } else {
        "windows"
    };

    let config = format!(
        r#"[tools]
just = "latest"

[tools.fake-platform-tool]
version = "latest"
os = ["{}"]
"#,
        non_current_os
    );

    fs::write(temp_dir.path().join("vx.toml"), &config).expect("Failed to write vx.toml");

    // Run sync in check mode with verbose
    let output = run_vx_in_dir(temp_dir.path(), &["sync", "--check", "--verbose"])
        .expect("Failed to run vx sync");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{}\n{}", stdout, stderr);

    // The platform-specific tool should NOT appear in the missing tools list
    // It should have been filtered out before the status check
    assert!(
        !combined.contains("fake-platform-tool (missing)"),
        "Platform-incompatible tool should not appear as missing.\nOutput:\n{}",
        combined
    );

    // 'just' should still appear (it's cross-platform)
    assert!(
        combined.contains("just"),
        "Cross-platform tool 'just' should appear in status.\nOutput:\n{}",
        combined
    );
}

/// Test that sync includes tools matching the current OS.
#[test]
fn test_sync_includes_current_platform_tools() {
    skip_if_no_vx!();

    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let current_os = if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "macos") {
        "darwin"
    } else {
        "linux"
    };

    let config = format!(
        r#"[tools]
just = "latest"

[tools.platform-tool]
version = "latest"
os = ["{}"]
"#,
        current_os
    );

    fs::write(temp_dir.path().join("vx.toml"), &config).expect("Failed to write vx.toml");

    // Run sync in check mode with verbose
    let output = run_vx_in_dir(temp_dir.path(), &["sync", "--check", "--verbose"])
        .expect("Failed to run vx sync");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{}\n{}", stdout, stderr);

    // The tool matching current platform should appear (may show as missing since not installed)
    assert!(
        combined.contains("platform-tool"),
        "Tool matching current platform should be included.\nOutput:\n{}",
        combined
    );
}
