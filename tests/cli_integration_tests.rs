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
// Update Command (REMOVED - command no longer exists)
// ============================================

// #[test]
// fn test_update_help() {
//     vx().args(["update", "--help"])
//         .assert()
//         .success()
//         .stdout(predicate::str::contains("Update"));
// }

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
// Clean Command (REMOVED - command no longer exists)
// ============================================

// #[test]
// fn test_clean_help() {
//     vx().args(["clean", "--help"])
//         .assert()
//         .success()
//         .stdout(predicate::str::contains("Clean"));
// }

// #[test]
// fn test_clean_dry_run() {
//     vx().args(["clean", "--dry-run"]).assert().success();
// }

// ============================================
// Stats Command (REMOVED - command no longer exists)
// ============================================

// #[test]
// fn test_stats_command() {
//     vx().arg("stats").assert().success();
// }

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
// Venv Command (REMOVED - command no longer exists)
// ============================================

// #[test]
// fn test_venv_help() {
//     vx().args(["venv", "--help"])
//         .assert()
//         .success()
//         .stdout(predicate::str::contains("Virtual environment"));
// }

// ============================================
// Global Command (REMOVED - command no longer exists)
// ============================================

// #[test]
// fn test_global_help() {
//     vx().args(["global", "--help"])
//         .assert()
//         .success()
//         .stdout(predicate::str::contains("Global"));
// }

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

// ============================================
// Add Command
// ============================================

#[test]
fn test_add_help() {
    vx().args(["add", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Add"))
        .stdout(predicate::str::contains("tool"));
}

#[test]
fn test_add_no_tool_error() {
    vx().arg("add")
        .assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

// ============================================
// Remove Command
// ============================================

#[test]
fn test_remove_help() {
    vx().args(["remove", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Remove"));
}

#[test]
fn test_remove_no_tool_error() {
    vx().arg("remove")
        .assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

// ============================================
// Lock Command
// ============================================

#[test]
fn test_lock_help() {
    vx().args(["lock", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("lock"));
}

// ============================================
// Check Command
// ============================================

#[test]
fn test_check_help() {
    vx().args(["check", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Check"));
}

// ============================================
// Run Command
// ============================================

#[test]
fn test_run_help() {
    vx().args(["run", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Run"));
}

#[test]
fn test_run_list_scripts() {
    vx().args(["run", "--list"]).assert().success();
}

// ============================================
// Analyze Command
// ============================================

#[test]
fn test_analyze_help() {
    vx().args(["analyze", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Analyze"));
}

// ============================================
// Dev Command
// ============================================

#[test]
fn test_dev_help() {
    vx().args(["dev", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("development"))
        .stdout(predicate::str::contains("environment"));
}

// ============================================
// Setup Command
// ============================================

#[test]
fn test_setup_help() {
    vx().args(["setup", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Setup"));
}

#[test]
fn test_setup_dry_run() {
    vx().args(["setup", "--dry-run"]).assert().success();
}

// ============================================
// Cache Command
// ============================================

#[test]
fn test_cache_help() {
    vx().args(["cache", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Cache"));
}

#[test]
fn test_cache_info() {
    vx().args(["cache", "info"]).assert().success();
}

#[test]
fn test_cache_list() {
    vx().args(["cache", "list"]).assert().success();
}

#[test]
fn test_cache_dir() {
    vx().args(["cache", "dir"]).assert().success();
}

#[test]
fn test_cache_prune_dry_run() {
    vx().args(["cache", "prune", "--dry-run"])
        .assert()
        .success();
}

// ============================================
// Ext Command
// ============================================

#[test]
fn test_ext_help() {
    vx().args(["ext", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Extension"));
}

#[test]
fn test_ext_list() {
    vx().args(["ext", "list"]).assert().success();
}

#[test]
fn test_extension_alias() {
    vx().args(["extension", "list"]).assert().success();
}

// ============================================
// Hook Command
// ============================================

#[test]
fn test_hook_help() {
    vx().args(["hook", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("hook"));
}

// ============================================
// Services Command
// ============================================

#[test]
fn test_services_help() {
    vx().args(["services", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("service"));
}

#[test]
fn test_services_status() {
    vx().args(["services", "status"]).assert().success();
}

// ============================================
// Container Command
// ============================================

#[test]
fn test_container_help() {
    vx().args(["container", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Container"));
}

// ============================================
// Migrate Command
// ============================================

#[test]
fn test_migrate_help() {
    vx().args(["migrate", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Migrate"));
}

#[test]
fn test_migrate_check() {
    vx().args(["migrate", "--check"]).assert().success();
}

// ============================================
// Auth Command
// ============================================

#[test]
fn test_auth_help() {
    vx().args(["auth", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Auth"));
}

#[test]
fn test_auth_status() {
    vx().args(["auth", "status"]).assert().success();
}

// ============================================
// Info Command
// ============================================

#[test]
fn test_info_command() {
    vx().arg("info")
        .assert()
        .success()
        .stdout(predicate::str::contains("vx"));
}

#[test]
fn test_info_json() {
    vx().args(["info", "--json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("{"));
}

// ============================================
// Global Command
// ============================================

#[test]
fn test_global_help() {
    vx().args(["global", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("global"));
}

#[test]
fn test_global_list() {
    vx().args(["global", "list"]).assert().success();
}

#[test]
fn test_global_alias_g() {
    vx().args(["g", "list"]).assert().success();
}

// ============================================
// Test Command
// ============================================

#[test]
fn test_test_help() {
    vx().args(["test", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("test"));
}

// ============================================
// Bundle Command
// ============================================

#[test]
fn test_bundle_help() {
    vx().args(["bundle", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("bundle"));
}

// ============================================
// Cache Mode Flag
// ============================================

#[test]
fn test_cache_mode_normal() {
    vx().args(["--cache-mode", "normal", "version"])
        .assert()
        .success();
}

#[test]
fn test_cache_mode_offline() {
    vx().args(["--cache-mode", "offline", "list"])
        .assert()
        .success();
}

#[test]
fn test_cache_mode_refresh() {
    vx().args(["--cache-mode", "refresh", "version"])
        .assert()
        .success();
}

// ============================================
// Use System Path Flag
// ============================================

#[test]
fn test_use_system_path_flag() {
    vx().args(["--use-system-path", "list"]).assert().success();
}

// ============================================
// Inherit Env Flag
// ============================================

#[test]
fn test_inherit_env_flag() {
    vx().args(["--inherit-env", "list"]).assert().success();
}
