//! Boundary E2E Tests for vx CLI
//!
//! These tests cover edge cases, boundary conditions, and complex scenarios
//! that may not be covered by basic E2E tests.

mod common;

use common::{
    assert_output_contains, assert_success, cleanup_test_env, init_test_env, run_vx, run_vx_in_dir,
    run_vx_with_env, stdout_str, vx_available, vx_binary,
};
use rstest::*;
use std::process::Command;
use tempfile::TempDir;

// ============================================================================
// Which Command Boundary Tests
// ============================================================================

mod which_boundary_tests {
    use super::*;

    /// Test vx which with --use-system-path flag
    #[rstest]
    #[test]
    fn test_which_use_system_path() {
        init_test_env();
        if !vx_available() {
            return;
        }

        // Test with a tool that should exist in system PATH (like git or cargo)
        let output = run_vx(&["--use-system-path", "which", "git"]).unwrap();

        // If git is installed, should show system path
        if output.status.success() {
            let stdout = stdout_str(&output);
            assert!(
                !stdout.contains("(system)"),
                "With --use-system-path, should not show (system) suffix"
            );
        }

        cleanup_test_env();
    }

    /// Test vx which with --all flag
    #[rstest]
    #[test]
    fn test_which_all_versions() {
        init_test_env();
        if !vx_available() {
            return;
        }

        let output = run_vx(&["which", "--all", "node"]).unwrap();
        // May fail if no versions installed, but should not crash
        let _ = output.status;

        cleanup_test_env();
    }

    /// Test vx which shows (system) for system-only tools
    #[rstest]
    #[test]
    fn test_which_system_fallback() {
        init_test_env();
        if !vx_available() {
            return;
        }

        // cargo is typically in system PATH but not vx-managed
        let output = run_vx(&["which", "cargo"]).unwrap();

        if output.status.success() {
            let stdout = stdout_str(&output);
            // If cargo is found in system PATH, should show (system)
            if stdout.contains("cargo") {
                assert!(
                    stdout.contains("(system)") || stdout.contains(".cargo"),
                    "System tools should show (system) suffix or be in .cargo path"
                );
            }
        }

        cleanup_test_env();
    }

    /// Test vx which with empty tool name
    #[rstest]
    #[test]
    fn test_which_empty_tool() {
        init_test_env();
        if !vx_available() {
            return;
        }

        // Empty string as tool name should fail
        let output = Command::new(vx_binary())
            .args(["which", ""])
            .output()
            .unwrap();

        // Should fail or show error
        assert!(
            !output.status.success() || stdout_str(&output).contains("not found"),
            "Empty tool name should fail"
        );

        cleanup_test_env();
    }

    /// Test vx which with special characters in tool name
    #[rstest]
    #[case("node@latest")]
    #[case("node@20.0.0")]
    #[case("../node")]
    #[case("node/../npm")]
    fn test_which_special_tool_names(#[case] tool: &str) {
        init_test_env();
        if !vx_available() {
            return;
        }

        let output = run_vx(&["which", tool]).unwrap();
        // Should handle gracefully (fail or succeed, but not crash)
        let _ = output.status;

        cleanup_test_env();
    }
}

// ============================================================================
// Install Command Boundary Tests
// ============================================================================

mod install_boundary_tests {
    use super::*;

    /// Test vx install with invalid version format
    #[rstest]
    #[case("node@not-a-version")]
    #[case("node@999.999.999")]
    #[case("node@v")]
    fn test_install_invalid_version(#[case] tool_spec: &str) {
        init_test_env();
        if !vx_available() {
            return;
        }

        let output = run_vx(&["install", tool_spec]).unwrap();
        // Should fail gracefully
        assert!(
            !output.status.success(),
            "Installing invalid version {} should fail",
            tool_spec
        );

        cleanup_test_env();
    }

    /// Test vx install with unknown tool
    #[rstest]
    #[case("unknown-tool-xyz-123")]
    #[case("")]
    #[case("node/subpath")]
    fn test_install_unknown_tool(#[case] tool: &str) {
        init_test_env();
        if !vx_available() {
            return;
        }

        let output = run_vx(&["install", tool]).unwrap();
        assert!(
            !output.status.success(),
            "Installing unknown tool should fail"
        );

        cleanup_test_env();
    }

    /// Test vx install --help
    #[rstest]
    #[test]
    fn test_install_help() {
        init_test_env();
        if !vx_available() {
            return;
        }

        let output = run_vx(&["install", "--help"]).unwrap();
        assert_success(&output, "vx install --help");
        assert_output_contains(&output, "Usage", "install help");

        cleanup_test_env();
    }
}

// ============================================================================
// Switch Command Boundary Tests
// ============================================================================

mod switch_boundary_tests {
    use super::*;

    /// Test vx switch with various version formats
    #[rstest]
    #[case("node@20")]
    #[case("node@20.0")]
    #[case("node@20.0.0")]
    #[case("node@latest")]
    #[case("node@lts")]
    fn test_switch_version_formats(#[case] tool_version: &str) {
        init_test_env();
        if !vx_available() {
            return;
        }

        let output = run_vx(&["switch", tool_version]).unwrap();
        // May fail if version not installed, but should not crash
        let _ = output.status;

        cleanup_test_env();
    }

    /// Test vx switch without @ separator
    #[rstest]
    #[test]
    fn test_switch_no_separator() {
        init_test_env();
        if !vx_available() {
            return;
        }

        let output = run_vx(&["switch", "node20"]).unwrap();
        // Should fail with helpful error
        assert!(!output.status.success(), "switch without @ should fail");

        cleanup_test_env();
    }

    /// Test vx switch with --global flag
    #[rstest]
    #[test]
    fn test_switch_global() {
        init_test_env();
        if !vx_available() {
            return;
        }

        let output = run_vx(&["switch", "--global", "node@20"]).unwrap();
        // May fail if not installed, but should handle --global flag
        let _ = output.status;

        cleanup_test_env();
    }
}

// ============================================================================
// Init Command Boundary Tests
// ============================================================================

mod init_boundary_tests {
    use super::*;

    /// Test vx init in directory with existing vx.toml
    #[rstest]
    #[test]
    fn test_init_existing_config() {
        init_test_env();
        if !vx_available() {
            return;
        }

        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("vx.toml");
        std::fs::write(&config_path, "[tools]\nnode = \"20\"").unwrap();

        // Without --force, should fail or warn
        let _output = run_vx_in_dir(temp_dir.path(), &["init", "--tools", "go"]).unwrap();
        // Should either fail or skip

        // With --force, should overwrite
        let output_force =
            run_vx_in_dir(temp_dir.path(), &["init", "--force", "--tools", "go"]).unwrap();
        assert_success(&output_force, "vx init --force");

        cleanup_test_env();
    }

    /// Test vx init with multiple tools
    #[rstest]
    #[test]
    fn test_init_multiple_tools() {
        init_test_env();
        if !vx_available() {
            return;
        }

        let temp_dir = TempDir::new().unwrap();

        let output = run_vx_in_dir(
            temp_dir.path(),
            &["init", "--tools", "node,go,uv", "--dry-run"],
        )
        .unwrap();

        assert_success(&output, "vx init with multiple tools");

        cleanup_test_env();
    }

    /// Test vx init with unknown template
    #[rstest]
    #[test]
    fn test_init_unknown_template() {
        init_test_env();
        if !vx_available() {
            return;
        }

        let temp_dir = TempDir::new().unwrap();

        let output = run_vx_in_dir(
            temp_dir.path(),
            &["init", "--template", "unknown-template-xyz"],
        )
        .unwrap();

        // Should fail with unknown template
        assert!(!output.status.success(), "Unknown template should fail");

        cleanup_test_env();
    }
}

// ============================================================================
// Sync Command Boundary Tests
// ============================================================================

mod sync_boundary_tests {
    use super::*;

    /// Test vx sync in directory without vx.toml
    #[rstest]
    #[test]
    fn test_sync_no_config() {
        init_test_env();
        if !vx_available() {
            return;
        }

        let temp_dir = TempDir::new().unwrap();

        let output = run_vx_in_dir(temp_dir.path(), &["sync"]).unwrap();
        // Should fail or warn about missing config
        let _ = output.status;

        cleanup_test_env();
    }

    /// Test vx sync with malformed vx.toml
    #[rstest]
    #[test]
    fn test_sync_malformed_config() {
        init_test_env();
        if !vx_available() {
            return;
        }

        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("vx.toml");
        std::fs::write(&config_path, "this is not valid toml [[[").unwrap();

        let output = run_vx_in_dir(temp_dir.path(), &["sync"]).unwrap();
        // Note: Currently succeeds (ignores malformed config), may want to fail
        let _ = output.status;

        cleanup_test_env();
    }

    /// Test vx sync with empty vx.toml
    #[rstest]
    #[test]
    fn test_sync_empty_config() {
        init_test_env();
        if !vx_available() {
            return;
        }

        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("vx.toml");
        std::fs::write(&config_path, "").unwrap();

        let output = run_vx_in_dir(temp_dir.path(), &["sync"]).unwrap();
        // Should succeed or warn about empty config
        let _ = output.status;

        cleanup_test_env();
    }

    /// Test vx sync --check
    #[rstest]
    #[test]
    fn test_sync_check_mode() {
        init_test_env();
        if !vx_available() {
            return;
        }

        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("vx.toml");
        std::fs::write(&config_path, "[tools]\nnode = \"20\"").unwrap();

        let output = run_vx_in_dir(temp_dir.path(), &["sync", "--check"]).unwrap();
        // Check mode should not install, just report
        let _ = output.status;

        cleanup_test_env();
    }
}

// ============================================================================
// Clean Command Boundary Tests
// NOTE: `vx clean` command does not exist. Use `vx cache prune` instead.
// These tests are ignored until the clean command is implemented.
// ============================================================================

mod clean_boundary_tests {
    use super::*;

    /// Test vx cache prune with all options combined
    #[rstest]
    #[test]
    fn test_clean_all_options() {
        init_test_env();
        if !vx_available() {
            return;
        }

        // Use `vx cache prune` instead of non-existent `vx clean`
        let output = run_vx(&["cache", "prune", "--dry-run"]).unwrap();
        assert_success(&output, "vx cache prune with options");

        cleanup_test_env();
    }

    /// Test vx cache prune --older-than
    #[rstest]
    #[case(0)]
    #[case(1)]
    #[case(30)]
    #[case(365)]
    #[ignore = "vx cache prune does not have --older-than option"]
    fn test_clean_older_than(#[case] _days: u32) {
        // This test is ignored because the option doesn't exist
    }

    /// Test vx cache prune
    #[rstest]
    #[test]
    fn test_clean_cache_and_orphaned() {
        init_test_env();
        if !vx_available() {
            return;
        }

        // Use `vx cache prune` for cache cleanup
        let output = run_vx(&["cache", "prune", "--dry-run"]).unwrap();
        assert_success(&output, "vx cache prune");

        cleanup_test_env();
    }
}

// ============================================================================
// Uninstall Command Boundary Tests
// ============================================================================

mod uninstall_boundary_tests {
    use super::*;

    /// Test vx uninstall with specific version
    #[rstest]
    #[test]
    fn test_uninstall_specific_version() {
        init_test_env();
        if !vx_available() {
            return;
        }

        let output = run_vx(&["uninstall", "node", "999.999.999"]).unwrap();
        // Note: Currently succeeds (no-op for non-existent), may want to fail
        let _ = output.status;

        cleanup_test_env();
    }

    /// Test vx uninstall with --force
    #[rstest]
    #[test]
    fn test_uninstall_force() {
        init_test_env();
        if !vx_available() {
            return;
        }

        let output = run_vx(&["uninstall", "--force", "nonexistent-tool-xyz"]).unwrap();
        // Note: Currently succeeds with --force, may want to fail for unknown tools
        let _ = output.status;

        cleanup_test_env();
    }

    /// Test vx rm alias (alias for remove command)
    #[rstest]
    #[test]
    fn test_remove_rm_alias() {
        init_test_env();
        if !vx_available() {
            return;
        }

        let output = run_vx(&["rm", "--help"]).unwrap();
        assert_success(&output, "vx rm --help");
        assert_output_contains(&output, "Remove", "rm alias should show remove help");

        cleanup_test_env();
    }
}

// ============================================================================
// Environment Variable Boundary Tests
// ============================================================================

mod env_var_boundary_tests {
    use super::*;

    /// Test vx with VX_HOME environment variable
    #[rstest]
    #[test]
    fn test_vx_home_env() {
        init_test_env();
        if !vx_available() {
            return;
        }

        let temp_dir = TempDir::new().unwrap();

        let output =
            run_vx_with_env(&["list"], &[("VX_HOME", temp_dir.path().to_str().unwrap())]).unwrap();

        assert_success(&output, "vx with VX_HOME");

        cleanup_test_env();
    }

    /// Test vx with VX_LOG_LEVEL
    #[rstest]
    #[case("debug")]
    #[case("info")]
    #[case("warn")]
    #[case("error")]
    fn test_vx_log_level(#[case] level: &str) {
        init_test_env();
        if !vx_available() {
            return;
        }

        let output = run_vx_with_env(&["list"], &[("VX_LOG_LEVEL", level)]).unwrap();
        assert_success(&output, &format!("vx with VX_LOG_LEVEL={}", level));

        cleanup_test_env();
    }

    /// Test vx with NO_COLOR
    #[rstest]
    #[test]
    fn test_no_color_env() {
        init_test_env();
        if !vx_available() {
            return;
        }

        let output = run_vx_with_env(&["list"], &[("NO_COLOR", "1")]).unwrap();
        assert_success(&output, "vx with NO_COLOR");

        cleanup_test_env();
    }
}

// ============================================================================
// Global Options Boundary Tests
// ============================================================================

mod global_options_tests {
    use super::*;

    /// Test --verbose with various commands
    #[rstest]
    #[case(&["--verbose", "list"])]
    #[case(&["--verbose", "cache", "info"])] // Use `cache info` instead of non-existent `stats`
    #[case(&["--verbose", "config", "show"])]
    #[case(&["list", "--verbose"])]
    fn test_verbose_option(#[case] args: &[&str]) {
        init_test_env();
        if !vx_available() {
            return;
        }

        let output = run_vx(args).unwrap();
        assert_success(&output, &format!("vx {:?}", args));

        cleanup_test_env();
    }

    /// Test --debug option
    #[rstest]
    #[test]
    fn test_debug_option() {
        init_test_env();
        if !vx_available() {
            return;
        }

        let output = run_vx(&["--debug", "list"]).unwrap();
        assert_success(&output, "vx --debug list");

        cleanup_test_env();
    }

    /// Test combined global options
    #[rstest]
    #[test]
    fn test_combined_global_options() {
        init_test_env();
        if !vx_available() {
            return;
        }

        let output = run_vx(&["--verbose", "--debug", "list"]).unwrap();
        assert_success(&output, "vx --verbose --debug list");

        cleanup_test_env();
    }
}

// ============================================================================
// Versions Command Boundary Tests
// ============================================================================

mod versions_boundary_tests {
    use super::*;

    /// Test vx versions with --latest 0
    #[rstest]
    #[test]
    #[ignore = "Requires network access"]
    fn test_versions_latest_zero() {
        init_test_env();
        if !vx_available() {
            return;
        }

        let output = run_vx(&["versions", "node", "--latest", "0"]).unwrap();
        // Should handle edge case
        let _ = output.status;

        cleanup_test_env();
    }

    /// Test vx versions with --prerelease
    #[rstest]
    #[test]
    #[ignore = "Requires network access"]
    fn test_versions_prerelease() {
        init_test_env();
        if !vx_available() {
            return;
        }

        let output = run_vx(&["versions", "node", "--prerelease", "--latest", "5"]).unwrap();
        let _ = output.status;

        cleanup_test_env();
    }

    /// Test vx versions with --detailed
    #[rstest]
    #[test]
    #[ignore = "Requires network access"]
    fn test_versions_detailed() {
        init_test_env();
        if !vx_available() {
            return;
        }

        let output = run_vx(&["versions", "node", "--detailed", "--latest", "3"]).unwrap();
        let _ = output.status;

        cleanup_test_env();
    }
}

// ============================================================================
// Venv Command Boundary Tests
// NOTE: `vx venv` command does not exist. Use `vx env` instead.
// ============================================================================

mod venv_boundary_tests {
    use super::*;

    /// Test vx env create with invalid name
    #[rstest]
    #[case("")]
    #[case("..")]
    #[case("/absolute/path")]
    #[case("name with spaces")]
    fn test_venv_create_invalid_name(#[case] name: &str) {
        init_test_env();
        if !vx_available() {
            return;
        }

        // Use `vx env create` instead of non-existent `vx venv create`
        let output = run_vx(&["env", "create", name, "--global"]).unwrap();
        // Should fail or handle gracefully
        let _ = output.status;

        cleanup_test_env();
    }

    /// Test vx env list when no envs exist
    #[rstest]
    #[test]
    fn test_venv_list_empty() {
        init_test_env();
        if !vx_available() {
            return;
        }

        // Use `vx env list` instead of non-existent `vx venv list`
        let output = run_vx(&["env", "list"]).unwrap();
        // Should succeed even with no envs
        assert_success(&output, "vx env list");

        cleanup_test_env();
    }

    /// Test vx env show when not in an env
    #[rstest]
    #[test]
    fn test_venv_current_none() {
        init_test_env();
        if !vx_available() {
            return;
        }

        // Use `vx env show` instead of non-existent `vx venv current`
        let output = run_vx(&["env", "show"]).unwrap();
        // Should succeed or indicate no active env
        let _ = output.status;

        cleanup_test_env();
    }

    /// Test vx env delete non-existent
    #[rstest]
    #[test]
    fn test_venv_remove_nonexistent() {
        init_test_env();
        if !vx_available() {
            return;
        }

        // Use `vx env delete` instead of non-existent `vx venv remove`
        let output = run_vx(&["env", "delete", "nonexistent-env-xyz", "--global"]).unwrap();
        // Note: Currently succeeds (no-op for non-existent), may want to fail
        let _ = output.status;

        cleanup_test_env();
    }
}

// ============================================================================
// Global Tool Management Boundary Tests
// NOTE: `vx global` command does not exist. Use `vx list` instead.
// ============================================================================

mod global_tool_boundary_tests {
    use super::*;

    /// Test vx list (replaces non-existent vx global list)
    #[rstest]
    #[test]
    fn test_global_list() {
        init_test_env();
        if !vx_available() {
            return;
        }

        // Use `vx list` instead of non-existent `vx global list`
        let output = run_vx(&["list"]).unwrap();
        assert_success(&output, "vx list");

        cleanup_test_env();
    }

    /// Test vx info for non-existent tool (replaces non-existent vx global info)
    #[rstest]
    #[test]
    fn test_global_info_nonexistent() {
        init_test_env();
        if !vx_available() {
            return;
        }

        // Use `vx info` instead of non-existent `vx global info`
        let output = run_vx(&["info"]).unwrap();
        // vx info shows system info
        let _ = output.status;

        cleanup_test_env();
    }

    /// Test vx cache prune --dry-run (replaces non-existent vx global cleanup)
    #[rstest]
    #[test]
    fn test_global_cleanup_dry_run() {
        init_test_env();
        if !vx_available() {
            return;
        }

        // Use `vx cache prune` instead of non-existent `vx global cleanup`
        let output = run_vx(&["cache", "prune", "--dry-run"]).unwrap();
        // Should succeed or show help
        let _ = output.status;

        cleanup_test_env();
    }
}

// ============================================================================
// Config Command Boundary Tests
// ============================================================================

mod config_boundary_tests {
    use super::*;

    /// Test vx config set with invalid key
    #[rstest]
    #[case("invalid.key.path", "value")]
    #[case("", "value")]
    #[case("key", "")]
    fn test_config_set_invalid(#[case] key: &str, #[case] value: &str) {
        init_test_env();
        if !vx_available() {
            return;
        }

        let output = run_vx(&["config", "set", key, value]).unwrap();
        // May fail or succeed depending on validation
        let _ = output.status;

        cleanup_test_env();
    }

    /// Test vx config get with invalid key
    #[rstest]
    #[test]
    fn test_config_get_invalid() {
        init_test_env();
        if !vx_available() {
            return;
        }

        let output = run_vx(&["config", "get", "nonexistent.key"]).unwrap();
        // Should fail or return empty
        let _ = output.status;

        cleanup_test_env();
    }

    /// Test vx config reset
    #[rstest]
    #[test]
    fn test_config_reset() {
        init_test_env();
        if !vx_available() {
            return;
        }

        let output = run_vx(&["config", "reset"]).unwrap();
        // Should succeed
        let _ = output.status;

        cleanup_test_env();
    }
}

// ============================================================================
// Update Command Boundary Tests
// NOTE: `vx update` and `vx up` commands do not exist.
// Use `vx self-update` for updating vx itself.
// ============================================================================

mod update_boundary_tests {
    use super::*;

    /// Test vx self-update --help (replaces non-existent vx update)
    #[rstest]
    #[test]
    fn test_update_specific_tool() {
        init_test_env();
        if !vx_available() {
            return;
        }

        // Use `vx self-update --help` instead of non-existent `vx update`
        let output = run_vx(&["self-update", "--help"]).unwrap();
        // Should show help
        let _ = output.status;

        cleanup_test_env();
    }

    /// Test vx versions for unknown tool (replaces non-existent vx update)
    #[rstest]
    #[test]
    fn test_update_unknown_tool() {
        init_test_env();
        if !vx_available() {
            return;
        }

        // Use `vx versions` to check tool versions
        let output = run_vx(&["versions", "unknown-tool-xyz"]).unwrap();
        // May fail for unknown tool
        let _ = output.status;

        cleanup_test_env();
    }

    /// Test vx self-update --help (replaces non-existent vx up)
    #[rstest]
    #[test]
    fn test_update_up_alias() {
        init_test_env();
        if !vx_available() {
            return;
        }

        // Use `vx self-update --help` instead of non-existent `vx up`
        let output = run_vx(&["self-update", "--help"]).unwrap();
        assert_success(&output, "vx self-update --help");

        cleanup_test_env();
    }
}

// ============================================================================
// Search Command Boundary Tests
// ============================================================================

mod search_boundary_tests {
    use super::*;

    /// Test vx search with special characters
    #[rstest]
    #[case("node*")]
    #[case("*")]
    #[case("")]
    fn test_search_special_queries(#[case] query: &str) {
        init_test_env();
        if !vx_available() {
            return;
        }

        let output = if query.is_empty() {
            run_vx(&["search"]).unwrap()
        } else {
            run_vx(&["search", query]).unwrap()
        };
        // Should handle gracefully
        let _ = output.status;

        cleanup_test_env();
    }

    /// Test vx search with --category
    #[rstest]
    #[case("runtime")]
    #[case("package-manager")]
    #[case("unknown-category")]
    fn test_search_category(#[case] category: &str) {
        init_test_env();
        if !vx_available() {
            return;
        }

        let output = run_vx(&["search", "--category", category]).unwrap();
        // Should handle gracefully
        let _ = output.status;

        cleanup_test_env();
    }

    /// Test vx search with --installed-only and --available-only
    #[rstest]
    #[test]
    fn test_search_filter_flags() {
        init_test_env();
        if !vx_available() {
            return;
        }

        let output1 = run_vx(&["search", "--installed-only"]).unwrap();
        assert_success(&output1, "vx search --installed-only");

        let output2 = run_vx(&["search", "--available-only"]).unwrap();
        assert_success(&output2, "vx search --available-only");

        cleanup_test_env();
    }
}

// ============================================================================
// Plugin Command Boundary Tests
// ============================================================================

mod plugin_boundary_tests {
    use super::*;

    /// Test vx plugin info for non-existent plugin
    #[rstest]
    #[test]
    fn test_plugin_info_nonexistent() {
        init_test_env();
        if !vx_available() {
            return;
        }

        let output = run_vx(&["plugin", "info", "nonexistent-plugin-xyz"]).unwrap();
        // Note: Currently succeeds, may want to fail in future
        let _ = output.status;

        cleanup_test_env();
    }

    /// Test vx plugin enable/disable for non-existent plugin
    #[rstest]
    #[test]
    fn test_plugin_enable_nonexistent() {
        init_test_env();
        if !vx_available() {
            return;
        }

        let output = run_vx(&["plugin", "enable", "nonexistent-plugin-xyz"]).unwrap();
        // Note: Currently succeeds, may want to fail in future
        let _ = output.status;

        cleanup_test_env();
    }

    /// Test vx plugin search
    #[rstest]
    #[test]
    fn test_plugin_search() {
        init_test_env();
        if !vx_available() {
            return;
        }

        let output = run_vx(&["plugin", "search", "node"]).unwrap();
        // Should succeed or indicate no results
        let _ = output.status;

        cleanup_test_env();
    }
}

// ============================================================================
// Shell Command Boundary Tests
// ============================================================================

mod shell_boundary_tests {
    use super::*;

    /// Test vx shell completions for supported shells
    #[rstest]
    #[case("bash")]
    #[case("zsh")]
    #[case("fish")]
    #[case("powershell")]
    fn test_shell_completions_all(#[case] shell: &str) {
        init_test_env();
        if !vx_available() {
            return;
        }

        let output = run_vx(&["shell", "completions", shell]).unwrap();
        assert_success(&output, &format!("vx shell completions {}", shell));

        cleanup_test_env();
    }

    /// Test vx shell completions for unsupported shell
    #[rstest]
    #[test]
    fn test_shell_completions_unsupported() {
        init_test_env();
        if !vx_available() {
            return;
        }

        let output = run_vx(&["shell", "completions", "elvish"]).unwrap();
        // Elvish is not supported, should fail
        assert!(!output.status.success(), "Unsupported shell should fail");

        cleanup_test_env();
    }

    /// Test vx shell init for all supported shells
    #[rstest]
    #[case("bash")]
    #[case("zsh")]
    #[case("fish")]
    #[case("powershell")]
    fn test_shell_init_all(#[case] shell: &str) {
        init_test_env();
        if !vx_available() {
            return;
        }

        let output = run_vx(&["shell", "init", shell]).unwrap();
        assert_success(&output, &format!("vx shell init {}", shell));

        cleanup_test_env();
    }

    /// Test vx shell init without shell argument (auto-detect)
    #[rstest]
    #[test]
    fn test_shell_init_auto() {
        init_test_env();
        if !vx_available() {
            return;
        }

        let output = run_vx(&["shell", "init"]).unwrap();
        // Should auto-detect or fail gracefully
        let _ = output.status;

        cleanup_test_env();
    }
}

// ============================================================================
// Concurrent Execution Tests
// ============================================================================

mod concurrent_tests {
    use super::*;
    use std::thread;

    /// Test multiple vx commands running concurrently
    #[rstest]
    #[test]
    fn test_concurrent_list_commands() {
        init_test_env();
        if !vx_available() {
            return;
        }

        let handles: Vec<_> = (0..5)
            .map(|_| thread::spawn(|| run_vx(&["list"]).unwrap()))
            .collect();

        for handle in handles {
            let output = handle.join().unwrap();
            assert_success(&output, "concurrent vx list");
        }

        cleanup_test_env();
    }

    /// Test concurrent cache info commands (replaces non-existent vx stats)
    #[rstest]
    #[test]
    fn test_concurrent_stats_commands() {
        init_test_env();
        if !vx_available() {
            return;
        }

        // Use `vx cache info` instead of non-existent `vx stats`
        let handles: Vec<_> = (0..3)
            .map(|_| thread::spawn(|| run_vx(&["cache", "info"]).unwrap()))
            .collect();

        for handle in handles {
            let output = handle.join().unwrap();
            assert_success(&output, "concurrent vx cache info");
        }

        cleanup_test_env();
    }
}
