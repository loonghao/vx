//! Consistency E2E Tests - Tests for command consistency
//!
//! This module tests that different commands produce consistent results:
//! - `vx where` and `vx uninstall` should agree on installed tools
//! - `vx list --installed` should match `vx where` results
//!
//! # Running Tests
//!
//! ```bash
//! # Run all consistency tests (requires network)
//! cargo test --package vx-cli --test consistency_e2e_tests -- --ignored --nocapture
//!
//! # Run specific test
//! cargo test --package vx-cli --test consistency_e2e_tests where_uninstall -- --ignored
//! ```

mod common;

use common::{
    assert_success, cleanup_test_env, combined_output, init_test_env, is_success, run_vx_with_env,
    stdout_str, vx_available,
};
use rstest::*;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

// ============================================================================
// Test Framework Helpers
// ============================================================================

/// Test context with isolated VX_HOME
struct ConsistencyTestContext {
    home: TempDir,
}

impl ConsistencyTestContext {
    fn new() -> Self {
        init_test_env();
        Self {
            home: TempDir::new().expect("Failed to create VX_HOME temp dir"),
        }
    }

    /// Run vx with isolated VX_HOME
    fn run(&self, args: &[&str]) -> std::io::Result<std::process::Output> {
        run_vx_with_env(args, &[("VX_HOME", self.home.path().to_str().unwrap())])
    }

    fn home_path(&self) -> PathBuf {
        self.home.path().to_path_buf()
    }

    /// Get the store directory path
    fn store_dir(&self) -> PathBuf {
        self.home_path().join("store")
    }

    /// Create a fake tool installation in the store directory
    fn create_fake_store_install(&self, tool: &str, version: &str) {
        let version_dir = self.store_dir().join(tool).join(version);
        fs::create_dir_all(&version_dir).expect("Failed to create store version dir");

        // Create a fake executable
        let exe_name = if cfg!(windows) {
            format!("{}.exe", tool)
        } else {
            tool.to_string()
        };
        let exe_path = version_dir.join(&exe_name);
        fs::write(&exe_path, "fake executable").expect("Failed to create fake executable");

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&exe_path).unwrap().permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&exe_path, perms).unwrap();
        }
    }
}

impl Drop for ConsistencyTestContext {
    fn drop(&mut self) {
        cleanup_test_env();
    }
}

/// Skip test if vx is not available
macro_rules! require_vx {
    () => {
        if !vx_available() {
            eprintln!("Skipping: vx binary not found");
            return;
        }
    };
}

// ============================================================================
// Where and Uninstall Consistency Tests
// ============================================================================

mod where_uninstall_consistency {
    use super::*;

    /// Test: If `vx where <tool>` finds a tool, `vx uninstall <tool>` should also find it
    #[rstest]
    #[test]
    #[ignore = "Requires network access"]
    fn where_finds_tool_uninstall_should_find_it() {
        require_vx!();
        let ctx = ConsistencyTestContext::new();

        // Install a tool
        let install_output = ctx.run(&["install", "uv"]).expect("Failed to run install");
        assert_success(&install_output, "install uv");

        // Verify `vx where` finds it
        let where_output = ctx.run(&["where", "uv"]).expect("Failed to run where");
        assert_success(&where_output, "where uv");
        let where_path = stdout_str(&where_output).trim().to_string();
        assert!(
            !where_path.is_empty() && !where_path.contains("(system)"),
            "vx where should find installed uv, got: {}",
            where_path
        );

        // Now `vx uninstall uv` (without version) should NOT say "No versions installed"
        let uninstall_output = ctx
            .run(&["uninstall", "uv"])
            .expect("Failed to run uninstall");
        let uninstall_combined = combined_output(&uninstall_output);

        // It should either:
        // 1. List versions and ask for --force, OR
        // 2. Successfully uninstall
        // It should NOT say "No versions of uv are installed"
        assert!(
            !uninstall_combined.contains("No versions of uv are installed"),
            "BUG: vx where finds uv but vx uninstall says no versions installed.\n\
             where output: {}\n\
             uninstall output: {}",
            where_path,
            uninstall_combined
        );
    }

    /// Test: Tool installed in store directory should be found by both where and uninstall
    #[rstest]
    #[test]
    fn store_install_consistency() {
        require_vx!();
        let ctx = ConsistencyTestContext::new();

        // Create fake installation in store directory
        ctx.create_fake_store_install("fake-tool", "1.0.0");

        // `vx where` should find it (or at least not crash)
        let where_output = ctx
            .run(&["where", "fake-tool"])
            .expect("Failed to run where");
        let where_stdout = stdout_str(&where_output);

        // If where finds it, uninstall should also find it
        if is_success(&where_output) && !where_stdout.contains("(system)") {
            let uninstall_output = ctx
                .run(&["uninstall", "fake-tool"])
                .expect("Failed to run uninstall");
            let uninstall_combined = combined_output(&uninstall_output);

            // Should not say "No versions installed" if where found it
            assert!(
                !uninstall_combined.contains("No versions of fake-tool are installed"),
                "Inconsistency: where finds fake-tool but uninstall doesn't.\n\
                 where: {}\n\
                 uninstall: {}",
                where_stdout,
                uninstall_combined
            );
        }
    }

    /// Test: After uninstall, `vx where` should not find the tool (in vx-managed paths)
    #[rstest]
    #[test]
    #[ignore = "Requires network access"]
    fn after_uninstall_where_should_not_find() {
        require_vx!();
        let ctx = ConsistencyTestContext::new();

        // Install a tool
        let install_output = ctx.run(&["install", "uv"]).expect("Failed to run install");
        assert_success(&install_output, "install uv");

        // Get the installed version
        let where_output = ctx.run(&["where", "uv"]).expect("Failed to run where");
        let where_path = stdout_str(&where_output).trim().to_string();

        // Extract version from path (e.g., ~/.vx/store/uv/0.7.13/uv.exe -> 0.7.13)
        // This is a simplified extraction - in real tests we'd use the `list` command
        let version = where_path
            .split(std::path::MAIN_SEPARATOR)
            .find(|s| {
                s.chars()
                    .next()
                    .map(|c| c.is_ascii_digit())
                    .unwrap_or(false)
            })
            .unwrap_or("latest");

        // Uninstall specific version
        let uninstall_output = ctx
            .run(&["uninstall", "uv", version])
            .expect("Failed to run uninstall");

        // If uninstall succeeded
        if is_success(&uninstall_output) {
            // `vx where` should now either:
            // 1. Not find it (fail or show system)
            // 2. Find a different version (if multiple installed)
            let where_after = ctx.run(&["where", "uv"]).expect("Failed to run where");
            let where_after_stdout = stdout_str(&where_after);

            // Should not find the same path
            assert!(
                !where_after_stdout.contains(&where_path)
                    || where_after_stdout.contains("(system)"),
                "After uninstall, where should not find the same path.\n\
                 Before: {}\n\
                 After: {}",
                where_path,
                where_after_stdout
            );
        }
    }
}

// ============================================================================
// List and Where Consistency Tests
// ============================================================================

mod list_where_consistency {
    use super::*;

    /// Test: `vx list --installed` should show tools that `vx where` can find
    #[rstest]
    #[test]
    #[ignore = "Requires network access"]
    fn list_installed_matches_where() {
        require_vx!();
        let ctx = ConsistencyTestContext::new();

        // Install a tool
        let install_output = ctx.run(&["install", "uv"]).expect("Failed to run install");
        assert_success(&install_output, "install uv");

        // `vx list --installed` should show uv
        let list_output = ctx
            .run(&["list", "--installed"])
            .expect("Failed to run list");
        let list_stdout = stdout_str(&list_output);

        // `vx where uv` should find it
        let where_output = ctx.run(&["where", "uv"]).expect("Failed to run where");

        if is_success(&where_output) {
            let where_stdout = stdout_str(&where_output);
            if !where_stdout.contains("(system)") {
                // If where finds vx-managed uv, list should show it
                assert!(
                    list_stdout.contains("uv"),
                    "list --installed should show uv if where finds it.\n\
                     list: {}\n\
                     where: {}",
                    list_stdout,
                    where_stdout
                );
            }
        }
    }
}

// ============================================================================
// Which (alias) Consistency Tests
// ============================================================================

mod which_alias_consistency {
    use super::*;

    /// Test: `vx which` should produce same result as `vx where`
    #[rstest]
    #[test]
    #[ignore = "Requires network access"]
    fn which_equals_where() {
        require_vx!();
        let ctx = ConsistencyTestContext::new();

        // Install a tool
        let install_output = ctx.run(&["install", "uv"]).expect("Failed to run install");
        assert_success(&install_output, "install uv");

        // Both commands should produce the same output
        let where_output = ctx.run(&["where", "uv"]).expect("Failed to run where");
        let which_output = ctx.run(&["which", "uv"]).expect("Failed to run which");

        let where_stdout = stdout_str(&where_output).trim().to_string();
        let which_stdout = stdout_str(&which_output).trim().to_string();

        assert_eq!(
            where_stdout, which_stdout,
            "vx where and vx which should produce identical output"
        );
    }
}

// ============================================================================
// Install Path Consistency Tests
// ============================================================================

mod install_path_consistency {
    use super::*;

    /// Test: After install, the tool should be in the expected directory structure
    #[rstest]
    #[test]
    #[ignore = "Requires network access"]
    fn install_uses_store_directory() {
        require_vx!();
        let ctx = ConsistencyTestContext::new();

        // Install a tool
        let install_output = ctx.run(&["install", "uv"]).expect("Failed to run install");
        assert_success(&install_output, "install uv");

        // Check where it was installed
        let where_output = ctx.run(&["where", "uv"]).expect("Failed to run where");
        let where_path = stdout_str(&where_output).trim().to_string();

        // New installations should go to store directory, not legacy tools directory
        let home_str = ctx.home_path().to_string_lossy().to_string();
        let store_path = format!("{}{}store", home_str, std::path::MAIN_SEPARATOR);

        assert!(
            where_path.contains(&store_path) || where_path.contains("store"),
            "New installations should use store directory.\n\
             Expected path to contain: {}\n\
             Actual path: {}",
            store_path,
            where_path
        );
    }
}

// ============================================================================
// Uninstall All Versions Consistency Tests
// ============================================================================

mod uninstall_all_consistency {
    use super::*;

    /// Test: `vx uninstall <tool> --force` should remove all versions
    #[rstest]
    #[test]
    #[ignore = "Requires network access"]
    fn uninstall_force_removes_all() {
        require_vx!();
        let ctx = ConsistencyTestContext::new();

        // Install a tool
        let install_output = ctx.run(&["install", "uv"]).expect("Failed to run install");
        assert_success(&install_output, "install uv");

        // Verify installed
        let where_before = ctx.run(&["where", "uv"]).expect("Failed to run where");
        assert_success(&where_before, "where uv before uninstall");

        // Uninstall all versions with --force
        let uninstall_output = ctx
            .run(&["uninstall", "uv", "--force"])
            .expect("Failed to run uninstall");

        // Should succeed (not say "no versions installed")
        let uninstall_combined = combined_output(&uninstall_output);
        assert!(
            !uninstall_combined.contains("No versions of uv are installed"),
            "uninstall --force should find installed versions.\n\
             Output: {}",
            uninstall_combined
        );

        // After uninstall, where should not find vx-managed uv
        let where_after = ctx.run(&["where", "uv"]).expect("Failed to run where");
        let where_after_stdout = stdout_str(&where_after);

        // Should either fail or show system path
        assert!(
            !is_success(&where_after) || where_after_stdout.contains("(system)"),
            "After uninstall --force, where should not find vx-managed uv.\n\
             Output: {}",
            where_after_stdout
        );
    }
}
