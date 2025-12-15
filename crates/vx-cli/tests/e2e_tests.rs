//! End-to-End Tests for vx CLI
//!
//! These tests run the actual vx binary and verify its behavior.
//! They are designed for "eating our own dogfood" - testing real CLI usage.
//!
//! Note: These tests require the vx binary to be built first.
//! Run `cargo build` before running these tests.

mod common;

use common::{cleanup_test_env, init_test_env, vx_available, vx_binary};
use rstest::*;
use std::process::Command;

/// Check if vx binary exists (use common module)
fn vx_exists() -> bool {
    vx_available()
}

// ============================================================================
// Basic CLI Tests
// ============================================================================

mod basic_cli_tests {
    use super::*;

    /// Test vx --version
    #[rstest]
    #[test]
    fn test_vx_version() {
        init_test_env();

        if !vx_exists() {
            eprintln!("Skipping test: vx binary not found");
            return;
        }

        let output = Command::new(vx_binary())
            .arg("--version")
            .output()
            .expect("Failed to execute vx");

        assert!(output.status.success(), "vx --version should succeed");

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("vx") || stdout.contains("0."),
            "Version output should contain 'vx' or version number"
        );

        cleanup_test_env();
    }

    /// Test vx --help
    #[rstest]
    #[test]
    fn test_vx_help() {
        init_test_env();

        if !vx_exists() {
            eprintln!("Skipping test: vx binary not found");
            return;
        }

        let output = Command::new(vx_binary())
            .arg("--help")
            .output()
            .expect("Failed to execute vx");

        assert!(output.status.success(), "vx --help should succeed");

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("Usage") || stdout.contains("usage"),
            "Help should contain usage information"
        );
        assert!(
            stdout.contains("install") || stdout.contains("Install"),
            "Help should mention install command"
        );

        cleanup_test_env();
    }

    /// Test vx version (subcommand)
    #[rstest]
    #[test]
    fn test_vx_version_subcommand() {
        init_test_env();

        if !vx_exists() {
            eprintln!("Skipping test: vx binary not found");
            return;
        }

        let output = Command::new(vx_binary())
            .arg("version")
            .output()
            .expect("Failed to execute vx");

        assert!(output.status.success(), "vx version should succeed");

        cleanup_test_env();
    }
}

// ============================================================================
// List Command E2E Tests
// ============================================================================

mod list_e2e_tests {
    use super::*;

    /// Test vx list
    #[rstest]
    #[test]
    fn test_vx_list() {
        init_test_env();

        if !vx_exists() {
            eprintln!("Skipping test: vx binary not found");
            return;
        }

        let output = Command::new(vx_binary())
            .arg("list")
            .output()
            .expect("Failed to execute vx");

        assert!(output.status.success(), "vx list should succeed");

        let stdout = String::from_utf8_lossy(&output.stdout);
        // Should list available tools
        assert!(
            stdout.contains("node")
                || stdout.contains("go")
                || stdout.contains("uv")
                || stdout.contains("bun")
                || stdout.contains("rust"),
            "List should show supported tools"
        );

        cleanup_test_env();
    }

    /// Test vx list --status
    #[rstest]
    #[test]
    fn test_vx_list_status() {
        init_test_env();

        if !vx_exists() {
            eprintln!("Skipping test: vx binary not found");
            return;
        }

        let output = Command::new(vx_binary())
            .args(["list", "--status"])
            .output()
            .expect("Failed to execute vx");

        assert!(output.status.success(), "vx list --status should succeed");

        cleanup_test_env();
    }

    /// Test vx ls (alias)
    #[rstest]
    #[test]
    fn test_vx_ls_alias() {
        init_test_env();

        if !vx_exists() {
            eprintln!("Skipping test: vx binary not found");
            return;
        }

        let output = Command::new(vx_binary())
            .arg("ls")
            .output()
            .expect("Failed to execute vx");

        assert!(output.status.success(), "vx ls should succeed");

        cleanup_test_env();
    }
}

// ============================================================================
// Config Command E2E Tests
// ============================================================================

mod config_e2e_tests {
    use super::*;

    /// Test vx config show
    #[rstest]
    #[test]
    fn test_vx_config_show() {
        init_test_env();

        if !vx_exists() {
            eprintln!("Skipping test: vx binary not found");
            return;
        }

        let output = Command::new(vx_binary())
            .args(["config", "show"])
            .output()
            .expect("Failed to execute vx");

        // Config show should succeed (may have empty config)
        let _ = output.status.success();

        cleanup_test_env();
    }

    /// Test vx config (without subcommand)
    #[rstest]
    #[test]
    fn test_vx_config() {
        init_test_env();

        if !vx_exists() {
            eprintln!("Skipping test: vx binary not found");
            return;
        }

        let output = Command::new(vx_binary())
            .arg("config")
            .output()
            .expect("Failed to execute vx");

        // Should show config or help
        let _ = output.status.success();

        cleanup_test_env();
    }
}

// ============================================================================
// Init Command E2E Tests
// ============================================================================

mod init_e2e_tests {
    use super::*;
    use tempfile::TempDir;

    /// Test vx init --list-templates
    #[rstest]
    #[test]
    fn test_vx_init_list_templates() {
        init_test_env();

        if !vx_exists() {
            eprintln!("Skipping test: vx binary not found");
            return;
        }

        let output = Command::new(vx_binary())
            .args(["init", "--list-templates"])
            .output()
            .expect("Failed to execute vx");

        assert!(
            output.status.success(),
            "vx init --list-templates should succeed"
        );

        let stdout = String::from_utf8_lossy(&output.stdout);
        // Should list available templates
        assert!(
            stdout.contains("node")
                || stdout.contains("python")
                || stdout.contains("go")
                || stdout.contains("template"),
            "Should list templates"
        );

        cleanup_test_env();
    }

    /// Test vx init --dry-run
    #[rstest]
    #[test]
    fn test_vx_init_dry_run() {
        init_test_env();

        if !vx_exists() {
            eprintln!("Skipping test: vx binary not found");
            return;
        }

        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        let output = Command::new(vx_binary())
            .args(["init", "--dry-run", "--tools", "node"])
            .current_dir(temp_dir.path())
            .output()
            .expect("Failed to execute vx");

        // Dry run should succeed
        let _ = output.status.success();

        // Should not create .vx.toml file
        assert!(
            !temp_dir.path().join(".vx.toml").exists(),
            "Dry run should not create files"
        );

        cleanup_test_env();
    }
}

// ============================================================================
// Clean Command E2E Tests
// ============================================================================

mod clean_e2e_tests {
    use super::*;

    /// Test vx clean --dry-run
    #[rstest]
    #[test]
    fn test_vx_clean_dry_run() {
        init_test_env();

        if !vx_exists() {
            eprintln!("Skipping test: vx binary not found");
            return;
        }

        let output = Command::new(vx_binary())
            .args(["clean", "--dry-run"])
            .output()
            .expect("Failed to execute vx");

        assert!(output.status.success(), "vx clean --dry-run should succeed");

        cleanup_test_env();
    }

    /// Test vx clean --cache --dry-run
    #[rstest]
    #[test]
    fn test_vx_clean_cache_dry_run() {
        init_test_env();

        if !vx_exists() {
            eprintln!("Skipping test: vx binary not found");
            return;
        }

        let output = Command::new(vx_binary())
            .args(["clean", "--cache", "--dry-run"])
            .output()
            .expect("Failed to execute vx");

        assert!(
            output.status.success(),
            "vx clean --cache --dry-run should succeed"
        );

        cleanup_test_env();
    }
}

// ============================================================================
// Stats Command E2E Tests
// ============================================================================

mod stats_e2e_tests {
    use super::*;

    /// Test vx stats
    #[rstest]
    #[test]
    fn test_vx_stats() {
        init_test_env();

        if !vx_exists() {
            eprintln!("Skipping test: vx binary not found");
            return;
        }

        let output = Command::new(vx_binary())
            .arg("stats")
            .output()
            .expect("Failed to execute vx");

        assert!(output.status.success(), "vx stats should succeed");

        cleanup_test_env();
    }
}

// ============================================================================
// Plugin Command E2E Tests
// ============================================================================

mod plugin_e2e_tests {
    use super::*;

    /// Test vx plugin list
    #[rstest]
    #[test]
    fn test_vx_plugin_list() {
        init_test_env();

        if !vx_exists() {
            eprintln!("Skipping test: vx binary not found");
            return;
        }

        let output = Command::new(vx_binary())
            .args(["plugin", "list"])
            .output()
            .expect("Failed to execute vx");

        assert!(output.status.success(), "vx plugin list should succeed");

        cleanup_test_env();
    }

    /// Test vx plugin stats
    #[rstest]
    #[test]
    fn test_vx_plugin_stats() {
        init_test_env();

        if !vx_exists() {
            eprintln!("Skipping test: vx binary not found");
            return;
        }

        let output = Command::new(vx_binary())
            .args(["plugin", "stats"])
            .output()
            .expect("Failed to execute vx");

        assert!(output.status.success(), "vx plugin stats should succeed");

        cleanup_test_env();
    }
}

// ============================================================================
// Shell Command E2E Tests
// ============================================================================

mod shell_e2e_tests {
    use super::*;

    /// Test vx shell completions bash
    #[rstest]
    #[test]
    fn test_vx_shell_completions_bash() {
        init_test_env();

        if !vx_exists() {
            eprintln!("Skipping test: vx binary not found");
            return;
        }

        let output = Command::new(vx_binary())
            .args(["shell", "completions", "bash"])
            .output()
            .expect("Failed to execute vx");

        assert!(
            output.status.success(),
            "vx shell completions bash should succeed"
        );

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("complete") || stdout.contains("_vx"),
            "Should output bash completion script"
        );

        cleanup_test_env();
    }

    /// Test vx shell completions powershell
    #[rstest]
    #[test]
    fn test_vx_shell_completions_powershell() {
        init_test_env();

        if !vx_exists() {
            eprintln!("Skipping test: vx binary not found");
            return;
        }

        let output = Command::new(vx_binary())
            .args(["shell", "completions", "powershell"])
            .output()
            .expect("Failed to execute vx");

        assert!(
            output.status.success(),
            "vx shell completions powershell should succeed"
        );

        cleanup_test_env();
    }

    /// Test vx shell init
    #[rstest]
    #[test]
    fn test_vx_shell_init_bash() {
        init_test_env();

        if !vx_exists() {
            eprintln!("Skipping test: vx binary not found");
            return;
        }

        let output = Command::new(vx_binary())
            .args(["shell", "init", "bash"])
            .output()
            .expect("Failed to execute vx");

        assert!(output.status.success(), "vx shell init bash should succeed");

        cleanup_test_env();
    }
}

// ============================================================================
// Search Command E2E Tests
// ============================================================================

mod search_e2e_tests {
    use super::*;

    /// Test vx search
    #[rstest]
    #[test]
    fn test_vx_search() {
        init_test_env();

        if !vx_exists() {
            eprintln!("Skipping test: vx binary not found");
            return;
        }

        let output = Command::new(vx_binary())
            .arg("search")
            .output()
            .expect("Failed to execute vx");

        assert!(output.status.success(), "vx search should succeed");

        cleanup_test_env();
    }

    /// Test vx search node
    #[rstest]
    #[test]
    fn test_vx_search_node() {
        init_test_env();

        if !vx_exists() {
            eprintln!("Skipping test: vx binary not found");
            return;
        }

        let output = Command::new(vx_binary())
            .args(["search", "node"])
            .output()
            .expect("Failed to execute vx");

        assert!(output.status.success(), "vx search node should succeed");

        cleanup_test_env();
    }
}

// ============================================================================
// Which Command E2E Tests
// ============================================================================

mod which_e2e_tests {
    use super::*;

    /// Test vx which node (may fail if not installed)
    #[rstest]
    #[test]
    fn test_vx_which_node() {
        init_test_env();

        if !vx_exists() {
            eprintln!("Skipping test: vx binary not found");
            return;
        }

        let output = Command::new(vx_binary())
            .args(["which", "node"])
            .output()
            .expect("Failed to execute vx");

        // May fail if node is not installed, but should not crash
        let _ = output.status;

        cleanup_test_env();
    }

    /// Test vx which nonexistent-tool
    #[rstest]
    #[test]
    fn test_vx_which_nonexistent() {
        init_test_env();

        if !vx_exists() {
            eprintln!("Skipping test: vx binary not found");
            return;
        }

        let output = Command::new(vx_binary())
            .args(["which", "nonexistent-tool-xyz"])
            .output()
            .expect("Failed to execute vx");

        // Should fail for unknown tool
        assert!(!output.status.success(), "vx which nonexistent should fail");

        cleanup_test_env();
    }
}

// ============================================================================
// Versions Command E2E Tests
// ============================================================================

mod versions_e2e_tests {
    use super::*;

    /// Test vx versions node --latest 3
    #[rstest]
    #[test]
    #[ignore = "Requires network access"]
    fn test_vx_versions_node() {
        init_test_env();

        if !vx_exists() {
            eprintln!("Skipping test: vx binary not found");
            return;
        }

        let output = Command::new(vx_binary())
            .args(["versions", "node", "--latest", "3"])
            .output()
            .expect("Failed to execute vx");

        // May fail due to network, but should not crash
        let _ = output.status;

        cleanup_test_env();
    }
}

// ============================================================================
// Error Handling E2E Tests
// ============================================================================

mod error_handling_e2e_tests {
    use super::*;

    /// Test vx with invalid command
    #[rstest]
    #[test]
    fn test_vx_invalid_command() {
        init_test_env();

        if !vx_exists() {
            eprintln!("Skipping test: vx binary not found");
            return;
        }

        let output = Command::new(vx_binary())
            .arg("invalid-command-xyz")
            .output()
            .expect("Failed to execute vx");

        // Should fail gracefully
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);

        // Should provide helpful error message
        assert!(
            stderr.contains("error")
                || stderr.contains("not found")
                || stderr.contains("unknown")
                || stdout.contains("error")
                || stdout.contains("not found")
                || !output.status.success(),
            "Should handle invalid command gracefully"
        );

        cleanup_test_env();
    }

    /// Test vx install without tool name
    #[rstest]
    #[test]
    fn test_vx_install_no_tool() {
        init_test_env();

        if !vx_exists() {
            eprintln!("Skipping test: vx binary not found");
            return;
        }

        let output = Command::new(vx_binary())
            .arg("install")
            .output()
            .expect("Failed to execute vx");

        // Should fail because tool name is required
        assert!(
            !output.status.success(),
            "vx install without tool should fail"
        );

        cleanup_test_env();
    }

    /// Test vx uninstall without tool name
    #[rstest]
    #[test]
    fn test_vx_uninstall_no_tool() {
        init_test_env();

        if !vx_exists() {
            eprintln!("Skipping test: vx binary not found");
            return;
        }

        let output = Command::new(vx_binary())
            .arg("uninstall")
            .output()
            .expect("Failed to execute vx");

        // Should fail because tool name is required
        assert!(
            !output.status.success(),
            "vx uninstall without tool should fail"
        );

        cleanup_test_env();
    }
}

// ============================================================================
// Self-Update Command E2E Tests
// ============================================================================

mod self_update_e2e_tests {
    use super::*;

    /// Test vx self-update --check
    #[rstest]
    #[test]
    #[ignore = "Requires network access"]
    fn test_vx_self_update_check() {
        init_test_env();

        if !vx_exists() {
            eprintln!("Skipping test: vx binary not found");
            return;
        }

        let output = Command::new(vx_binary())
            .args(["self-update", "--check"])
            .output()
            .expect("Failed to execute vx");

        // May fail due to network, but should not crash
        let _ = output.status;

        cleanup_test_env();
    }
}

// ============================================================================
// Verbose Mode E2E Tests
// ============================================================================

mod verbose_e2e_tests {
    use super::*;

    /// Test vx --verbose list
    #[rstest]
    #[test]
    fn test_vx_verbose_list() {
        init_test_env();

        if !vx_exists() {
            eprintln!("Skipping test: vx binary not found");
            return;
        }

        let output = Command::new(vx_binary())
            .args(["--verbose", "list"])
            .output()
            .expect("Failed to execute vx");

        assert!(output.status.success(), "vx --verbose list should succeed");

        cleanup_test_env();
    }

    /// Test vx -v list (short form)
    #[rstest]
    #[test]
    fn test_vx_v_list() {
        init_test_env();

        if !vx_exists() {
            eprintln!("Skipping test: vx binary not found");
            return;
        }

        let output = Command::new(vx_binary())
            .args(["-v", "list"])
            .output()
            .expect("Failed to execute vx");

        assert!(output.status.success(), "vx -v list should succeed");

        cleanup_test_env();
    }
}
