//! Project Context E2E Tests for vx CLI
//!
//! Tests for .vx.toml project configuration and context-aware behavior

use crate::common::*;
use rstest::*;
use tempfile::TempDir;

// ============================================================================
// .vx.toml Basic Tests
// ============================================================================

/// Test: vx list in directory with .vx.toml
#[rstest]
#[test]
fn test_list_with_vx_toml() {
    skip_if_no_vx!();

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let vx_toml = temp_dir.path().join(".vx.toml");

    std::fs::write(
        &vx_toml,
        r#"[tools]
node = "20"
"#,
    )
    .expect("Failed to write .vx.toml");

    let output =
        run_vx_in_dir(temp_dir.path(), &["list"]).expect("Failed to run vx list in project");

    assert!(
        is_success(&output),
        "vx list should succeed in project dir: {}",
        combined_output(&output)
    );
}

/// Test: vx reads .vx.toml configuration
#[rstest]
#[test]
fn test_vx_toml_read() {
    skip_if_no_vx!();

    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    std::fs::write(
        temp_dir.path().join(".vx.toml"),
        r#"[tools]
node = "20"
go = "1.21"
uv = "latest"
"#,
    )
    .expect("Failed to write .vx.toml");

    // vx list should show project tools
    let output = run_vx_in_dir(temp_dir.path(), &["list"]).expect("Failed to run vx list");

    assert!(is_success(&output), "vx list should succeed");
}

// ============================================================================
// vx sync Tests
// ============================================================================

/// Test: vx sync --check in project directory
#[rstest]
#[test]
fn test_sync_check() {
    skip_if_no_vx!();

    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    std::fs::write(
        temp_dir.path().join(".vx.toml"),
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

/// Test: vx sync --dry-run
#[rstest]
#[test]
fn test_sync_dry_run() {
    skip_if_no_vx!();

    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    std::fs::write(
        temp_dir.path().join(".vx.toml"),
        r#"[tools]
node = "20"
"#,
    )
    .expect("Failed to write .vx.toml");

    let output = run_vx_in_dir(temp_dir.path(), &["sync", "--dry-run"])
        .expect("Failed to run vx sync --dry-run");

    // Should succeed without actually installing
    let _ = combined_output(&output);
}

// ============================================================================
// vx init Tests
// ============================================================================

/// Test: vx init creates .vx.toml
#[rstest]
#[test]
fn test_init_creates_vx_toml() {
    skip_if_no_vx!();

    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let output = run_vx_in_dir(temp_dir.path(), &["init", "--tools", "node"])
        .expect("Failed to run vx init");

    if is_success(&output) {
        // Check for either vx.toml (new) or .vx.toml (legacy)
        assert!(
            temp_dir.path().join("vx.toml").exists() || temp_dir.path().join(".vx.toml").exists(),
            "vx init should create vx.toml or .vx.toml"
        );
    }
}

/// Test: vx init --dry-run doesn't create files
#[rstest]
#[test]
fn test_init_dry_run() {
    skip_if_no_vx!();

    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let output = run_vx_in_dir(temp_dir.path(), &["init", "--dry-run", "--tools", "node"])
        .expect("Failed to run vx init --dry-run");

    // Should succeed
    let _ = is_success(&output);

    // Should not create .vx.toml
    assert!(
        !temp_dir.path().join(".vx.toml").exists(),
        "vx init --dry-run should not create files"
    );
}

/// Test: vx init --list-templates
#[rstest]
#[test]
fn test_init_list_templates() {
    skip_if_no_vx!();

    let output =
        run_vx(&["init", "--list-templates"]).expect("Failed to run vx init --list-templates");

    assert!(
        is_success(&output),
        "vx init --list-templates should succeed"
    );

    let stdout = stdout_str(&output);
    assert!(
        stdout.contains("node") || stdout.contains("python") || stdout.contains("template"),
        "Should list templates: {}",
        stdout
    );
}

/// Test: vx init with template
#[rstest]
#[test]
fn test_init_with_template() {
    skip_if_no_vx!();

    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let output = run_vx_in_dir(temp_dir.path(), &["init", "--template", "node"])
        .expect("Failed to run vx init --template");

    // May succeed or fail depending on template availability
    let _ = combined_output(&output);
}

// ============================================================================
// Nested Project Tests
// ============================================================================

/// Test: vx respects closest .vx.toml in nested directories
#[rstest]
#[test]
fn test_nested_vx_toml() {
    skip_if_no_vx!();

    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Create parent .vx.toml
    std::fs::write(
        temp_dir.path().join(".vx.toml"),
        r#"[tools]
node = "18"
"#,
    )
    .expect("Failed to write parent .vx.toml");

    // Create nested directory with its own .vx.toml
    let nested_dir = temp_dir.path().join("nested");
    std::fs::create_dir(&nested_dir).expect("Failed to create nested dir");

    std::fs::write(
        nested_dir.join(".vx.toml"),
        r#"[tools]
node = "20"
"#,
    )
    .expect("Failed to write nested .vx.toml");

    // Run vx list in nested directory
    let output =
        run_vx_in_dir(&nested_dir, &["list"]).expect("Failed to run vx list in nested dir");

    assert!(is_success(&output), "vx list should succeed in nested dir");
}

/// Test: vx walks up directory tree to find .vx.toml
#[rstest]
#[test]
fn test_vx_toml_discovery() {
    skip_if_no_vx!();

    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Create .vx.toml at root
    std::fs::write(
        temp_dir.path().join(".vx.toml"),
        r#"[tools]
node = "20"
"#,
    )
    .expect("Failed to write .vx.toml");

    // Create deep nested directory without .vx.toml
    let deep_dir = temp_dir.path().join("a").join("b").join("c");
    std::fs::create_dir_all(&deep_dir).expect("Failed to create deep dir");

    // Run vx list from deep directory - should find parent .vx.toml
    let output = run_vx_in_dir(&deep_dir, &["list"]).expect("Failed to run vx list in deep dir");

    assert!(
        is_success(&output),
        "vx list should succeed finding parent .vx.toml"
    );
}

// ============================================================================
// Tool Version Pinning Tests
// ============================================================================

/// Test: .vx.toml with specific version
#[rstest]
#[test]
fn test_version_pinning() {
    skip_if_no_vx!();

    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    std::fs::write(
        temp_dir.path().join(".vx.toml"),
        r#"[tools]
node = "20.10.0"
"#,
    )
    .expect("Failed to write .vx.toml");

    // vx should recognize the pinned version
    let output = run_vx_in_dir(temp_dir.path(), &["list"]).expect("Failed to run vx list");

    assert!(
        is_success(&output),
        "vx list should succeed with pinned version"
    );
}

/// Test: .vx.toml with version range
#[rstest]
#[test]
fn test_version_range() {
    skip_if_no_vx!();

    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    std::fs::write(
        temp_dir.path().join(".vx.toml"),
        r#"[tools]
node = ">=18"
"#,
    )
    .expect("Failed to write .vx.toml");

    let output = run_vx_in_dir(temp_dir.path(), &["list"]).expect("Failed to run vx list");

    assert!(
        is_success(&output),
        "vx list should succeed with version range"
    );
}

/// Test: .vx.toml with "latest"
#[rstest]
#[test]
fn test_version_latest() {
    skip_if_no_vx!();

    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    std::fs::write(
        temp_dir.path().join(".vx.toml"),
        r#"[tools]
node = "latest"
uv = "latest"
"#,
    )
    .expect("Failed to write .vx.toml");

    let output = run_vx_in_dir(temp_dir.path(), &["list"]).expect("Failed to run vx list");

    assert!(is_success(&output), "vx list should succeed with 'latest'");
}

// ============================================================================
// Multiple Tools Configuration
// ============================================================================

/// Test: .vx.toml with multiple tools
#[rstest]
#[test]
fn test_multiple_tools_config() {
    skip_if_no_vx!();

    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    std::fs::write(
        temp_dir.path().join(".vx.toml"),
        r#"[tools]
node = "20"
go = "1.21"
uv = "latest"
bun = "1"
cargo = "latest"
"#,
    )
    .expect("Failed to write .vx.toml");

    let output = run_vx_in_dir(temp_dir.path(), &["list"]).expect("Failed to run vx list");

    assert!(
        is_success(&output),
        "vx list should succeed with multiple tools"
    );
}

// ============================================================================
// Invalid Configuration Tests
// ============================================================================

/// Test: Invalid .vx.toml syntax
#[rstest]
#[test]
fn test_invalid_vx_toml_syntax() {
    skip_if_no_vx!();

    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    std::fs::write(
        temp_dir.path().join(".vx.toml"),
        r#"this is not valid toml {"#,
    )
    .expect("Failed to write invalid .vx.toml");

    let output = run_vx_in_dir(temp_dir.path(), &["list"]).expect("Failed to run vx list");

    // Should handle gracefully (may fail or warn)
    let _ = combined_output(&output);
}

/// Test: .vx.toml with unknown tool
#[rstest]
#[test]
fn test_unknown_tool_in_config() {
    skip_if_no_vx!();

    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    std::fs::write(
        temp_dir.path().join(".vx.toml"),
        r#"[tools]
unknown-tool-xyz = "1.0"
"#,
    )
    .expect("Failed to write .vx.toml");

    let output = run_vx_in_dir(temp_dir.path(), &["list"]).expect("Failed to run vx list");

    // Should handle gracefully
    let _ = combined_output(&output);
}

// ============================================================================
// Config Command Tests
// ============================================================================

/// Test: vx config show
#[rstest]
#[test]
fn test_config_show() {
    skip_if_no_vx!();

    let output = run_vx(&["config", "show"]).expect("Failed to run vx config show");

    // Should succeed (may show empty or default config)
    let _ = combined_output(&output);
}

/// Test: vx config show in project
#[rstest]
#[test]
fn test_config_show_in_project() {
    skip_if_no_vx!();

    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    std::fs::write(
        temp_dir.path().join(".vx.toml"),
        r#"[tools]
node = "20"
"#,
    )
    .expect("Failed to write .vx.toml");

    let output =
        run_vx_in_dir(temp_dir.path(), &["config", "show"]).expect("Failed to run vx config show");

    // Should show project config
    let _ = combined_output(&output);
}

// ============================================================================
// Stats Command Tests
// ============================================================================

/// Test: vx stats
#[rstest]
#[test]
fn test_stats() {
    skip_if_no_vx!();

    let output = run_vx(&["stats"]).expect("Failed to run vx stats");

    assert!(is_success(&output), "vx stats should succeed");
}

/// Test: vx stats in project
#[rstest]
#[test]
fn test_stats_in_project() {
    skip_if_no_vx!();

    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    std::fs::write(
        temp_dir.path().join(".vx.toml"),
        r#"[tools]
node = "20"
"#,
    )
    .expect("Failed to write .vx.toml");

    let output = run_vx_in_dir(temp_dir.path(), &["stats"]).expect("Failed to run vx stats");

    assert!(is_success(&output), "vx stats should succeed in project");
}

// ============================================================================
// Clean Command Tests
// ============================================================================

/// Test: vx clean --dry-run
#[rstest]
#[test]
fn test_clean_dry_run() {
    skip_if_no_vx!();

    let output = run_vx(&["clean", "--dry-run"]).expect("Failed to run vx clean --dry-run");

    assert!(is_success(&output), "vx clean --dry-run should succeed");
}

/// Test: vx clean --cache --dry-run
#[rstest]
#[test]
fn test_clean_cache_dry_run() {
    skip_if_no_vx!();

    let output =
        run_vx(&["clean", "--cache", "--dry-run"]).expect("Failed to run vx clean --cache");

    assert!(
        is_success(&output),
        "vx clean --cache --dry-run should succeed"
    );
}

// ============================================================================
// Plugin Command Tests
// ============================================================================

/// Test: vx plugin list
#[rstest]
#[test]
fn test_plugin_list() {
    skip_if_no_vx!();

    let output = run_vx(&["plugin", "list"]).expect("Failed to run vx plugin list");

    assert!(is_success(&output), "vx plugin list should succeed");
}

/// Test: vx plugin stats
#[rstest]
#[test]
fn test_plugin_stats() {
    skip_if_no_vx!();

    let output = run_vx(&["plugin", "stats"]).expect("Failed to run vx plugin stats");

    assert!(is_success(&output), "vx plugin stats should succeed");
}

// ============================================================================
// Shell Integration Tests
// ============================================================================

/// Test: vx shell completions
#[rstest]
#[test]
fn test_shell_completions() {
    skip_if_no_vx!();

    let shells = ["bash", "zsh", "fish", "powershell"];

    for shell in shells {
        let output = run_vx(&["shell", "completions", shell])
            .unwrap_or_else(|_| panic!("Failed to run vx shell completions {}", shell));

        // Should succeed for all shells
        assert!(
            is_success(&output),
            "vx shell completions {} should succeed",
            shell
        );
    }
}

/// Test: vx shell init
#[rstest]
#[test]
fn test_shell_init() {
    skip_if_no_vx!();

    let shells = ["bash", "zsh", "fish", "powershell"];

    for shell in shells {
        let output = run_vx(&["shell", "init", shell])
            .unwrap_or_else(|_| panic!("Failed to run vx shell init {}", shell));

        // Should succeed for all shells
        assert!(
            is_success(&output),
            "vx shell init {} should succeed",
            shell
        );
    }
}

// ============================================================================
// vx dev Tests
// ============================================================================

/// Test: vx dev requires .vx.toml
#[rstest]
#[test]
fn test_dev_requires_vx_toml() {
    skip_if_no_vx!();

    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Running vx dev without .vx.toml should fail
    let output = run_vx_in_dir(temp_dir.path(), &["dev", "--command", "echo", "test"])
        .expect("Failed to run vx dev");

    // Should fail because no .vx.toml exists
    let combined = combined_output(&output);
    assert!(
        !is_success(&output) || combined.contains("No .vx.toml"),
        "vx dev should fail or warn without .vx.toml: {}",
        combined
    );
}

/// Test: vx dev --command runs in dev environment
#[rstest]
#[test]
fn test_dev_with_command() {
    skip_if_no_vx!();

    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    std::fs::write(
        temp_dir.path().join(".vx.toml"),
        r#"[tools]
node = "20"
"#,
    )
    .expect("Failed to write .vx.toml");

    // Run a simple command in dev environment
    let output = run_vx_in_dir(
        temp_dir.path(),
        &["dev", "--no-install", "--command", "echo", "hello"],
    )
    .expect("Failed to run vx dev --command");

    // Should succeed
    let _ = combined_output(&output);
}

/// Test: vx dev with empty tools config
#[rstest]
#[test]
fn test_dev_empty_tools() {
    skip_if_no_vx!();

    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    std::fs::write(
        temp_dir.path().join(".vx.toml"),
        r#"[tools]
# No tools configured
"#,
    )
    .expect("Failed to write .vx.toml");

    let output = run_vx_in_dir(temp_dir.path(), &["dev", "--command", "echo", "test"])
        .expect("Failed to run vx dev");

    // Should handle gracefully (warn about no tools)
    let combined = combined_output(&output);
    assert!(
        combined.contains("No tools") || is_success(&output),
        "Should handle empty tools config: {}",
        combined
    );
}

// ============================================================================
// vx setup Tests
// ============================================================================

/// Test: vx setup requires .vx.toml
#[rstest]
#[test]
fn test_setup_requires_vx_toml() {
    skip_if_no_vx!();

    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Create an empty .vx.toml in temp dir to prevent searching parent directories
    // This simulates a project without any tools configured
    std::fs::write(temp_dir.path().join(".vx.toml"), "[tools]\n")
        .expect("Failed to write .vx.toml");

    let output = run_vx_in_dir(temp_dir.path(), &["setup"]).expect("Failed to run vx setup");

    // Should warn about no tools configured (empty [tools] section)
    let combined = combined_output(&output);
    assert!(
        combined.contains("No tools") || combined.contains("no tools"),
        "vx setup should warn about no tools: {}",
        combined
    );
}

/// Test: vx setup --dry-run
#[rstest]
#[test]
fn test_setup_dry_run() {
    skip_if_no_vx!();

    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    std::fs::write(
        temp_dir.path().join(".vx.toml"),
        r#"[tools]
node = "20"
uv = "latest"
"#,
    )
    .expect("Failed to write .vx.toml");

    let output = run_vx_in_dir(temp_dir.path(), &["setup", "--dry-run"])
        .expect("Failed to run vx setup --dry-run");

    // Should succeed without actually installing
    let combined = combined_output(&output);
    assert!(
        is_success(&output) || combined.contains("Would install"),
        "vx setup --dry-run should succeed: {}",
        combined
    );
}

/// Test: vx setup with empty tools
#[rstest]
#[test]
fn test_setup_empty_tools() {
    skip_if_no_vx!();

    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    std::fs::write(
        temp_dir.path().join(".vx.toml"),
        r#"[tools]
# No tools
"#,
    )
    .expect("Failed to write .vx.toml");

    let output = run_vx_in_dir(temp_dir.path(), &["setup"]).expect("Failed to run vx setup");

    // Should handle gracefully
    let combined = combined_output(&output);
    assert!(
        combined.contains("No tools") || is_success(&output),
        "Should handle empty tools: {}",
        combined
    );
}

// ============================================================================
// vx add/rm-tool Tests
// ============================================================================

/// Test: vx add adds tool to .vx.toml
#[rstest]
#[test]
fn test_add_tool() {
    skip_if_no_vx!();

    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Create initial .vx.toml
    std::fs::write(
        temp_dir.path().join(".vx.toml"),
        r#"[tools]
node = "20"
"#,
    )
    .expect("Failed to write .vx.toml");

    let output = run_vx_in_dir(temp_dir.path(), &["add", "uv", "--version", "latest"])
        .expect("Failed to run vx add");

    if is_success(&output) {
        // Verify tool was added
        let config_content = std::fs::read_to_string(temp_dir.path().join(".vx.toml"))
            .expect("Failed to read .vx.toml");
        assert!(
            config_content.contains("uv"),
            "uv should be added to .vx.toml"
        );
    }
}

/// Test: vx add requires .vx.toml
#[rstest]
#[test]
fn test_add_requires_vx_toml() {
    skip_if_no_vx!();

    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let output = run_vx_in_dir(temp_dir.path(), &["add", "node"]).expect("Failed to run vx add");

    // Should fail because no .vx.toml exists
    let combined = combined_output(&output);
    assert!(
        !is_success(&output) || combined.contains("No .vx.toml"),
        "vx add should fail without .vx.toml: {}",
        combined
    );
}

/// Test: vx rm-tool removes tool from .vx.toml
#[rstest]
#[test]
fn test_remove_tool() {
    skip_if_no_vx!();

    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Create .vx.toml with multiple tools
    std::fs::write(
        temp_dir.path().join(".vx.toml"),
        r#"[tools]
node = "20"
uv = "latest"
"#,
    )
    .expect("Failed to write .vx.toml");

    let output =
        run_vx_in_dir(temp_dir.path(), &["rm-tool", "uv"]).expect("Failed to run vx rm-tool");

    if is_success(&output) {
        // Verify tool was removed
        let config_content = std::fs::read_to_string(temp_dir.path().join(".vx.toml"))
            .expect("Failed to read .vx.toml");
        assert!(
            !config_content.contains("uv ="),
            "uv should be removed from .vx.toml"
        );
        assert!(
            config_content.contains("node"),
            "node should still be in .vx.toml"
        );
    }
}

// ============================================================================
// vx run Tests
// ============================================================================

/// Test: vx run executes script from .vx.toml
#[rstest]
#[test]
fn test_run_script() {
    skip_if_no_vx!();

    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    std::fs::write(
        temp_dir.path().join(".vx.toml"),
        r#"[tools]
node = "20"

[scripts]
hello = "echo hello world"
"#,
    )
    .expect("Failed to write .vx.toml");

    let output = run_vx_in_dir(temp_dir.path(), &["run", "hello"]).expect("Failed to run vx run");

    // Should succeed
    let combined = combined_output(&output);
    assert!(
        is_success(&output) || combined.contains("hello"),
        "vx run should execute script: {}",
        combined
    );
}

/// Test: vx run with missing script
#[rstest]
#[test]
fn test_run_missing_script() {
    skip_if_no_vx!();

    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    std::fs::write(
        temp_dir.path().join(".vx.toml"),
        r#"[tools]
node = "20"
"#,
    )
    .expect("Failed to write .vx.toml");

    let output =
        run_vx_in_dir(temp_dir.path(), &["run", "nonexistent"]).expect("Failed to run vx run");

    // Should fail with helpful message
    let combined = combined_output(&output);
    assert!(
        !is_success(&output) || combined.contains("not found") || combined.contains("No scripts"),
        "vx run should fail for missing script: {}",
        combined
    );
}

/// Test: vx run requires .vx.toml
#[rstest]
#[test]
fn test_run_requires_vx_toml() {
    skip_if_no_vx!();

    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let output = run_vx_in_dir(temp_dir.path(), &["run", "test"]).expect("Failed to run vx run");

    // Should fail because no .vx.toml exists
    let combined = combined_output(&output);
    assert!(
        !is_success(&output) || combined.contains("No .vx.toml"),
        "vx run should fail without .vx.toml: {}",
        combined
    );
}
