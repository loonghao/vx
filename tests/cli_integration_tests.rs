//! Cross-platform CLI Integration Tests using assert_cmd
//!
//! These tests verify CLI behavior across all platforms (Windows, macOS, Linux).
//! Unlike snapshot tests, these use pattern matching for flexible verification.
//!
//! Run all tests:
//! ```bash
//! cargo test --test cli_integration_tests
//! ```

use assert_cmd::Command;
use predicates::prelude::*;

/// Get a Command for the vx binary
fn vx() -> Command {
    #[allow(deprecated)]
    Command::cargo_bin("vx").unwrap()
}

// ============================================
// Basic Commands
// ============================================

#[test]
fn test_version_flag() {
    vx().arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::starts_with("vx "));
}

#[test]
fn test_version_subcommand() {
    vx().arg("version")
        .assert()
        .success()
        .stdout(predicate::str::contains("vx"));
}

#[test]
fn test_help_flag() {
    vx().arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Universal"))
        .stdout(predicate::str::contains("Usage:"))
        .stdout(predicate::str::contains("Commands:"))
        .stdout(predicate::str::contains("Options:"));
}

#[test]
fn test_help_subcommand() {
    vx().arg("help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage:"));
}

// ============================================
// List Command
// ============================================

#[test]
fn test_list_command() {
    vx().arg("list").assert().success();
}

#[test]
fn test_list_alias_ls() {
    vx().arg("ls").assert().success();
}

#[test]
fn test_list_all_flag() {
    vx().args(["list", "--all"]).assert().success();
}

#[test]
fn test_list_status_flag() {
    vx().args(["list", "--status"]).assert().success();
}

// ============================================
// Search Command
// ============================================

#[test]
fn test_search_no_args() {
    vx().arg("search")
        .assert()
        .success()
        // Should list some common tools
        .stdout(predicate::str::contains("node"))
        .stdout(predicate::str::contains("go"))
        .stdout(predicate::str::contains("uv"));
}

#[test]
fn test_search_node() {
    vx().args(["search", "node"])
        .assert()
        .success()
        .stdout(predicate::str::contains("node"));
}

// ============================================
// Config Command
// ============================================

#[test]
fn test_config_help() {
    vx().args(["config", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Configuration"));
}

#[test]
fn test_config_show() {
    vx().args(["config", "show"]).assert().success();
}

// ============================================
// Init Command
// ============================================

#[test]
fn test_init_help() {
    vx().args(["init", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Initialize"));
}

#[test]
fn test_init_list_templates() {
    vx().args(["init", "--list-templates"])
        .assert()
        .success()
        .stdout(predicate::str::contains("node"))
        .stdout(predicate::str::contains("python"))
        .stdout(predicate::str::contains("rust"))
        .stdout(predicate::str::contains("go"));
}

// ============================================
// Install Command
// ============================================

#[test]
fn test_install_help() {
    vx().args(["install", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Install"));
}

#[test]
fn test_install_no_tool_error() {
    vx().arg("install")
        .assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

// ============================================
// Uninstall Command
// ============================================

#[test]
fn test_uninstall_help() {
    vx().args(["uninstall", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Uninstall"));
}

#[test]
fn test_uninstall_no_tool_error() {
    vx().arg("uninstall")
        .assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

#[test]
fn test_uninstall_alias_rm() {
    vx().args(["rm", "--help"]).assert().success();
}

#[test]
fn test_uninstall_alias_remove() {
    vx().args(["remove", "--help"]).assert().success();
}

// ============================================
// Update Command
// ============================================

#[test]
fn test_update_help() {
    vx().args(["update", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Update"));
}

// ============================================
// Self-Update Command
// ============================================

#[test]
fn test_self_update_help() {
    vx().args(["self-update", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Update vx"));
}

// ============================================
// Which Command
// ============================================

#[test]
fn test_which_help() {
    vx().args(["which", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("which"));
}

#[test]
fn test_which_no_tool_error() {
    vx().arg("which")
        .assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

// ============================================
// Versions Command
// ============================================

#[test]
fn test_versions_help() {
    vx().args(["versions", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("version"));
}

#[test]
fn test_versions_no_tool_error() {
    vx().arg("versions")
        .assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

// ============================================
// Switch Command
// ============================================

#[test]
fn test_switch_help() {
    vx().args(["switch", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Switch"));
}

#[test]
fn test_switch_no_tool_error() {
    vx().arg("switch")
        .assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

// ============================================
// Sync Command
// ============================================

#[test]
fn test_sync_help() {
    vx().args(["sync", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Sync"));
}

// ============================================
// Clean Command
// ============================================

#[test]
fn test_clean_help() {
    vx().args(["clean", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Clean"));
}

#[test]
fn test_clean_dry_run() {
    vx().args(["clean", "--dry-run"]).assert().success();
}

// ============================================
// Stats Command
// ============================================

#[test]
fn test_stats_command() {
    vx().arg("stats").assert().success();
}

// ============================================
// Plugin Command
// ============================================

#[test]
fn test_plugin_help() {
    vx().args(["plugin", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Plugin"));
}

#[test]
fn test_plugin_list() {
    vx().args(["plugin", "list"]).assert().success();
}

// ============================================
// Shell Command
// ============================================

#[test]
fn test_shell_help() {
    vx().args(["shell", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Shell"));
}

#[test]
fn test_shell_completions_bash() {
    vx().args(["shell", "completions", "bash"])
        .assert()
        .success()
        .stdout(predicate::str::contains("_vx"));
}

#[test]
fn test_shell_completions_powershell() {
    vx().args(["shell", "completions", "powershell"])
        .assert()
        .success()
        .stdout(predicate::str::contains("vx"));
}

#[test]
fn test_shell_init_bash() {
    vx().args(["shell", "init", "bash"])
        .assert()
        .success()
        .stdout(predicate::str::contains("vx"));
}

// ============================================
// Venv Command
// ============================================

#[test]
fn test_venv_help() {
    vx().args(["venv", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Virtual environment"));
}

// ============================================
// Global Command
// ============================================

#[test]
fn test_global_help() {
    vx().args(["global", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Global"));
}

// ============================================
// Env Command
// ============================================

#[test]
fn test_env_help() {
    vx().args(["env", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Environment"));
}

// ============================================
// Error Handling
// ============================================

#[test]
fn test_invalid_command() {
    vx().arg("invalid-command-xyz-12345")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Unknown runtime"));
}

#[test]
fn test_verbose_flag() {
    vx().args(["--verbose", "list"]).assert().success();
}

#[test]
fn test_debug_flag() {
    vx().args(["--debug", "version"]).assert().success();
}
