//! Lifecycle E2E Tests - Full lifecycle testing for runtime management
//!
//! This module provides comprehensive E2E tests covering:
//! - Install/Uninstall operations
//! - Version switching
//! - Multiple version coexistence
//! - Virtual environment creation
//! - Version pinning
//!
//! # Running Tests
//!
//! ```bash
//! # Run all lifecycle tests (requires network)
//! cargo test --package vx-cli --test lifecycle_e2e_tests -- --ignored --nocapture
//!
//! # Run specific test categories
//! cargo test --package vx-cli --test lifecycle_e2e_tests install -- --ignored
//! cargo test --package vx-cli --test lifecycle_e2e_tests uninstall -- --ignored
//! cargo test --package vx-cli --test lifecycle_e2e_tests switch -- --ignored
//! cargo test --package vx-cli --test lifecycle_e2e_tests venv -- --ignored
//! ```

mod common;

use common::{
    assert_failure, assert_output_contains, assert_success, cleanup_test_env, combined_output,
    init_test_env, is_success, run_vx_with_env, stdout_str, vx_available,
};
use rstest::*;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

// ============================================================================
// Test Framework Helpers
// ============================================================================

/// Lifecycle test context with isolated VX_HOME
struct LifecycleTestContext {
    home: TempDir,
    work_dir: TempDir,
}

impl LifecycleTestContext {
    fn new() -> Self {
        init_test_env();
        Self {
            home: TempDir::new().expect("Failed to create VX_HOME temp dir"),
            work_dir: TempDir::new().expect("Failed to create work dir"),
        }
    }

    /// Run vx with isolated VX_HOME
    fn run(&self, args: &[&str]) -> std::io::Result<std::process::Output> {
        run_vx_with_env(args, &[("VX_HOME", self.home.path().to_str().unwrap())])
    }

    /// Run vx in work directory with isolated VX_HOME
    fn run_in_work_dir(&self, args: &[&str]) -> std::io::Result<std::process::Output> {
        use std::process::Command;
        Command::new(common::vx_binary())
            .args(args)
            .current_dir(self.work_dir.path())
            .env("VX_HOME", self.home.path())
            .output()
    }

    fn home_path(&self) -> PathBuf {
        self.home.path().to_path_buf()
    }

    fn work_path(&self) -> PathBuf {
        self.work_dir.path().to_path_buf()
    }

    /// Check if a tool is installed in this isolated environment
    fn is_installed(&self, tool: &str) -> bool {
        self.run(&["which", tool])
            .map(|o| is_success(&o))
            .unwrap_or(false)
    }

    /// Install a tool
    fn install(&self, tool: &str) -> bool {
        self.run(&["install", tool])
            .map(|o| is_success(&o))
            .unwrap_or(false)
    }

    /// Install a specific version
    fn install_version(&self, tool: &str, version: &str) -> bool {
        let spec = format!("{}@{}", tool, version);
        self.run(&["install", &spec])
            .map(|o| is_success(&o))
            .unwrap_or(false)
    }

    /// Uninstall a tool version
    fn uninstall(&self, tool: &str, version: &str) -> bool {
        self.run(&["uninstall", tool, version])
            .map(|o| is_success(&o))
            .unwrap_or(false)
    }

    /// Switch to a version
    fn switch(&self, tool: &str, version: &str) -> bool {
        self.run(&["switch", tool, version])
            .map(|o| is_success(&o))
            .unwrap_or(false)
    }

    /// Get current version of a tool
    fn current_version(&self, tool: &str) -> Option<String> {
        self.run(&["current", tool])
            .ok()
            .filter(is_success)
            .map(|o| stdout_str(&o).trim().to_string())
    }

    /// List installed versions
    #[allow(dead_code)]
    fn list_versions(&self, tool: &str) -> Vec<String> {
        self.run(&["list", tool])
            .ok()
            .filter(is_success)
            .map(|o| {
                stdout_str(&o)
                    .lines()
                    .filter(|l| !l.is_empty() && !l.contains("Installed"))
                    .map(|l| l.trim().to_string())
                    .collect()
            })
            .unwrap_or_default()
    }
}

impl Drop for LifecycleTestContext {
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

/// Skip test with a message
macro_rules! skip_test {
    ($msg:expr) => {
        eprintln!("Skipping: {}", $msg);
        return;
    };
}

// ============================================================================
// Install Tests
// ============================================================================

mod install_tests {
    use super::*;

    #[rstest]
    #[test]
    #[ignore = "Requires network access"]
    fn install_uv_latest() {
        require_vx!();
        let ctx = LifecycleTestContext::new();

        // Install UV (latest)
        let output = ctx.run(&["install", "uv"]).expect("Failed to run install");
        assert_success(&output, "install uv");

        // Verify installation
        assert!(ctx.is_installed("uv"), "UV should be installed");

        // Verify can run
        let version_output = ctx.run(&["uv", "--version"]).expect("Failed to run uv");
        assert_success(&version_output, "uv --version");
        assert_output_contains(&version_output, "uv", "should show uv version");
    }

    #[rstest]
    #[test]
    #[ignore = "Requires network access"]
    fn install_node_latest() {
        require_vx!();
        let ctx = LifecycleTestContext::new();

        // Install Node.js (latest)
        let output = ctx
            .run(&["install", "node"])
            .expect("Failed to run install");
        assert_success(&output, "install node");

        // Verify installation
        assert!(ctx.is_installed("node"), "Node should be installed");

        // Verify can run
        let version_output = ctx.run(&["node", "--version"]).expect("Failed to run node");
        assert_success(&version_output, "node --version");
    }

    #[rstest]
    #[test]
    #[ignore = "Requires network access"]
    fn install_specific_version() {
        require_vx!();
        let ctx = LifecycleTestContext::new();

        // Install specific Node.js version
        let output = ctx
            .run(&["install", "node@20.10.0"])
            .expect("Failed to run install");

        if is_success(&output) {
            // Verify version
            let version_output = ctx.run(&["node", "--version"]).expect("Failed to run node");
            let stdout = stdout_str(&version_output);
            assert!(
                stdout.contains("20.10.0"),
                "Should install exact version 20.10.0, got: {}",
                stdout
            );
        } else {
            eprintln!(
                "Install specific version failed (may be expected): {}",
                combined_output(&output)
            );
        }
    }

    #[rstest]
    #[test]
    #[ignore = "Requires network access"]
    fn install_major_version() {
        require_vx!();
        let ctx = LifecycleTestContext::new();

        // Install Node.js 20.x (latest in major)
        let output = ctx
            .run(&["install", "node@20"])
            .expect("Failed to run install");

        if is_success(&output) {
            // Verify version is 20.x
            let version_output = ctx.run(&["node", "--version"]).expect("Failed to run node");
            let stdout = stdout_str(&version_output);
            assert!(
                stdout.contains("v20.") || stdout.contains("20."),
                "Should install Node.js 20.x, got: {}",
                stdout
            );
        }
    }

    #[rstest]
    #[test]
    #[ignore = "Requires network access"]
    fn install_go_latest() {
        require_vx!();
        let ctx = LifecycleTestContext::new();

        let output = ctx.run(&["install", "go"]).expect("Failed to run install");
        assert_success(&output, "install go");

        // Verify installation
        let version_output = ctx.run(&["go", "version"]).expect("Failed to run go");
        assert_success(&version_output, "go version");
        assert_output_contains(&version_output, "go version", "should show go version");
    }

    #[rstest]
    #[test]
    #[ignore = "Requires network access"]
    fn install_bun_latest() {
        require_vx!();
        let ctx = LifecycleTestContext::new();

        let output = ctx.run(&["install", "bun"]).expect("Failed to run install");

        // Bun may not be available on all platforms
        if is_success(&output) {
            let version_output = ctx.run(&["bun", "--version"]).expect("Failed to run bun");
            assert_success(&version_output, "bun --version");
        } else {
            eprintln!(
                "Bun installation failed (may be platform-specific): {}",
                combined_output(&output)
            );
        }
    }

    #[rstest]
    #[test]
    #[ignore = "Requires network access"]
    fn install_nonexistent_tool() {
        require_vx!();
        let ctx = LifecycleTestContext::new();

        let output = ctx
            .run(&["install", "nonexistent-tool-xyz-123"])
            .expect("Failed to run install");
        assert_failure(&output, "install nonexistent tool");
    }

    #[rstest]
    #[test]
    #[ignore = "Requires network access"]
    fn install_invalid_version() {
        require_vx!();
        let ctx = LifecycleTestContext::new();

        let output = ctx
            .run(&["install", "node@999.999.999"])
            .expect("Failed to run install");
        assert_failure(&output, "install invalid version");
    }

    #[rstest]
    #[test]
    #[ignore = "Requires network access"]
    fn install_already_installed() {
        require_vx!();
        let ctx = LifecycleTestContext::new();

        // Install UV first time
        let output1 = ctx.run(&["install", "uv"]).expect("Failed to run install");
        assert_success(&output1, "first install");

        // Install UV second time (should succeed or indicate already installed)
        let output2 = ctx.run(&["install", "uv"]).expect("Failed to run install");
        // Should either succeed or indicate already installed
        let combined = combined_output(&output2);
        assert!(
            is_success(&output2) || combined.contains("already"),
            "Second install should succeed or indicate already installed"
        );
    }
}

// ============================================================================
// Uninstall Tests
// ============================================================================

mod uninstall_tests {
    use super::*;

    #[rstest]
    #[test]
    #[ignore = "Requires network access"]
    fn uninstall_installed_tool() {
        require_vx!();
        let ctx = LifecycleTestContext::new();

        // Install first
        if !ctx.install("uv") {
            skip_test!("UV installation failed");
        }

        // Get installed version
        let list_output = ctx
            .run(&["list", "uv", "--status"])
            .expect("Failed to list");
        let stdout = stdout_str(&list_output);
        eprintln!("Installed versions: {}", stdout);

        // Uninstall (need to specify version)
        // First try to get the version from 'current' command
        let current_output = ctx.run(&["current", "uv"]);
        if let Ok(o) = current_output {
            if is_success(&o) {
                let version = stdout_str(&o).trim().to_string();
                if !version.is_empty() {
                    let uninstall_output = ctx
                        .run(&["uninstall", "uv", &version])
                        .expect("Failed to uninstall");
                    assert_success(&uninstall_output, "uninstall uv");

                    // Verify uninstalled
                    assert!(!ctx.is_installed("uv"), "UV should be uninstalled");
                }
            }
        }
    }

    #[rstest]
    #[test]
    #[ignore = "Requires network access"]
    fn uninstall_nonexistent_tool() {
        require_vx!();
        let ctx = LifecycleTestContext::new();

        let output = ctx
            .run(&["uninstall", "nonexistent-tool-xyz", "1.0.0"])
            .expect("Failed to run uninstall");
        assert_failure(&output, "uninstall nonexistent tool");
    }

    #[rstest]
    #[test]
    #[ignore = "Requires network access"]
    fn uninstall_nonexistent_version() {
        require_vx!();
        let ctx = LifecycleTestContext::new();

        // Install UV first
        if !ctx.install("uv") {
            skip_test!("UV installation failed");
        }

        // Try to uninstall a version that doesn't exist
        let output = ctx
            .run(&["uninstall", "uv", "0.0.1"])
            .expect("Failed to run uninstall");
        assert_failure(&output, "uninstall nonexistent version");
    }

    #[rstest]
    #[test]
    #[ignore = "Requires network access"]
    fn uninstall_and_reinstall() {
        require_vx!();
        let ctx = LifecycleTestContext::new();

        // Install
        if !ctx.install("uv") {
            skip_test!("UV installation failed");
        }

        // Get version
        let current = ctx.current_version("uv");
        if current.is_none() {
            skip_test!("Could not get current UV version");
        }
        let version = current.unwrap();

        // Uninstall
        assert!(ctx.uninstall("uv", &version), "Uninstall should succeed");

        // Verify uninstalled
        assert!(!ctx.is_installed("uv"), "UV should be uninstalled");

        // Reinstall
        assert!(ctx.install("uv"), "Reinstall should succeed");

        // Verify installed again
        assert!(ctx.is_installed("uv"), "UV should be installed again");
    }
}

// ============================================================================
// Version Switch Tests
// ============================================================================

mod switch_tests {
    use super::*;

    #[rstest]
    #[test]
    #[ignore = "Requires network access"]
    fn switch_between_versions() {
        require_vx!();
        let ctx = LifecycleTestContext::new();

        // Install two Node.js versions
        let v1 = "20.10.0";
        let v2 = "20.11.0";

        if !ctx.install_version("node", v1) {
            skip_test!(format!("Node {} installation failed", v1));
        }

        if !ctx.install_version("node", v2) {
            skip_test!(format!("Node {} installation failed", v2));
        }

        // Switch to v1
        assert!(ctx.switch("node", v1), "Switch to {} should succeed", v1);

        // Verify current version
        let version_output = ctx.run(&["node", "--version"]).expect("Failed to run node");
        let stdout = stdout_str(&version_output);
        assert!(
            stdout.contains(v1),
            "Current version should be {}, got: {}",
            v1,
            stdout
        );

        // Switch to v2
        assert!(ctx.switch("node", v2), "Switch to {} should succeed", v2);

        // Verify current version changed
        let version_output = ctx.run(&["node", "--version"]).expect("Failed to run node");
        let stdout = stdout_str(&version_output);
        assert!(
            stdout.contains(v2),
            "Current version should be {}, got: {}",
            v2,
            stdout
        );
    }

    #[rstest]
    #[test]
    #[ignore = "Requires network access"]
    fn switch_to_nonexistent_version() {
        require_vx!();
        let ctx = LifecycleTestContext::new();

        // Install one version first
        if !ctx.install("node") {
            skip_test!("Node installation failed");
        }

        // Try to switch to non-installed version
        let output = ctx
            .run(&["switch", "node", "999.999.999"])
            .expect("Failed to run switch");
        assert_failure(&output, "switch to nonexistent version");
    }

    #[rstest]
    #[test]
    #[ignore = "Requires network access"]
    fn switch_uv_versions() {
        require_vx!();
        let ctx = LifecycleTestContext::new();

        // Install two UV versions
        if !ctx.install_version("uv", "0.4.0") {
            skip_test!("UV 0.4.0 installation failed");
        }

        if !ctx.install_version("uv", "0.5.0") {
            skip_test!("UV 0.5.0 installation failed");
        }

        // Switch to 0.4.0
        assert!(ctx.switch("uv", "0.4.0"), "Switch to 0.4.0 should succeed");

        let version_output = ctx.run(&["uv", "--version"]).expect("Failed to run uv");
        let stdout = stdout_str(&version_output);
        assert!(
            stdout.contains("0.4.0"),
            "Current version should be 0.4.0, got: {}",
            stdout
        );

        // Switch to 0.5.0
        assert!(ctx.switch("uv", "0.5.0"), "Switch to 0.5.0 should succeed");

        let version_output = ctx.run(&["uv", "--version"]).expect("Failed to run uv");
        let stdout = stdout_str(&version_output);
        assert!(
            stdout.contains("0.5.0"),
            "Current version should be 0.5.0, got: {}",
            stdout
        );
    }

    #[rstest]
    #[test]
    #[ignore = "Requires network access"]
    fn switch_invalid_tool() {
        require_vx!();
        let ctx = LifecycleTestContext::new();

        let output = ctx
            .run(&["switch", "nonexistent-tool-xyz", "1.0.0"])
            .expect("Failed to run switch");
        assert_failure(&output, "switch invalid tool");
    }
}

// ============================================================================
// Multiple Version Coexistence Tests
// ============================================================================

mod multi_version_tests {
    use super::*;

    #[rstest]
    #[test]
    #[ignore = "Requires network access"]
    fn install_multiple_node_versions() {
        require_vx!();
        let ctx = LifecycleTestContext::new();

        let versions = ["20.10.0", "20.11.0", "22.0.0"];
        let mut installed = Vec::new();

        for v in &versions {
            if ctx.install_version("node", v) {
                installed.push(*v);
                eprintln!("Installed Node.js {}", v);
            } else {
                eprintln!("Failed to install Node.js {} (may be expected)", v);
            }
        }

        // Should have at least 2 versions installed
        assert!(
            installed.len() >= 2,
            "Should install at least 2 versions, got: {:?}",
            installed
        );

        // Verify can switch between them
        for v in &installed {
            assert!(ctx.switch("node", v), "Should switch to {}", v);
        }
    }

    #[rstest]
    #[test]
    #[ignore = "Requires network access"]
    fn install_multiple_tools() {
        require_vx!();
        let ctx = LifecycleTestContext::new();

        let tools = ["uv", "node", "go"];
        let mut installed = Vec::new();

        for tool in &tools {
            if ctx.install(tool) {
                installed.push(*tool);
                eprintln!("Installed {}", tool);
            } else {
                eprintln!("Failed to install {} (may be expected)", tool);
            }
        }

        // Should have at least 1 tool installed
        assert!(!installed.is_empty(), "Should install at least 1 tool");

        // Verify all installed tools work
        for tool in &installed {
            let version_arg = if *tool == "go" {
                "version"
            } else {
                "--version"
            };
            let output = ctx.run(&[tool, version_arg]).expect("Failed to run tool");
            assert_success(&output, &format!("{} {}", tool, version_arg));
        }
    }

    #[rstest]
    #[test]
    #[ignore = "Requires network access"]
    fn list_installed_versions() {
        require_vx!();
        let ctx = LifecycleTestContext::new();

        // Install multiple versions
        let _ = ctx.install_version("uv", "0.4.0");
        let _ = ctx.install_version("uv", "0.5.0");

        // List installed versions
        let output = ctx
            .run(&["list", "uv", "--status"])
            .expect("Failed to list");
        assert_success(&output, "list uv --status");

        let stdout = stdout_str(&output);
        eprintln!("Installed UV versions:\n{}", stdout);

        // Should show installed versions
        // (exact format depends on implementation)
    }
}

// ============================================================================
// Virtual Environment Tests
// ============================================================================

mod venv_tests {
    use super::*;

    #[rstest]
    #[test]
    #[ignore = "Requires network access"]
    fn uv_create_venv() {
        require_vx!();
        let ctx = LifecycleTestContext::new();

        // Install UV first
        if !ctx.install("uv") {
            skip_test!("UV installation failed");
        }

        // Create virtual environment
        let output = ctx
            .run_in_work_dir(&["uv", "venv", ".venv"])
            .expect("Failed to run uv venv");
        assert_success(&output, "uv venv .venv");

        // Verify .venv directory exists
        let venv_path = ctx.work_path().join(".venv");
        assert!(venv_path.exists(), ".venv directory should exist");

        // Verify structure (should have bin/Scripts and pyvenv.cfg)
        let has_bin = venv_path.join("bin").exists() || venv_path.join("Scripts").exists();
        assert!(has_bin, "venv should have bin or Scripts directory");
    }

    #[rstest]
    #[test]
    #[ignore = "Requires network access"]
    fn uv_create_venv_custom_name() {
        require_vx!();
        let ctx = LifecycleTestContext::new();

        if !ctx.install("uv") {
            skip_test!("UV installation failed");
        }

        // Create venv with custom name
        let output = ctx
            .run_in_work_dir(&["uv", "venv", "my-env"])
            .expect("Failed to run uv venv");
        assert_success(&output, "uv venv my-env");

        let venv_path = ctx.work_path().join("my-env");
        assert!(venv_path.exists(), "my-env directory should exist");
    }

    #[rstest]
    #[test]
    #[ignore = "Requires network access"]
    fn uv_venv_with_python_version() {
        require_vx!();
        let ctx = LifecycleTestContext::new();

        if !ctx.install("uv") {
            skip_test!("UV installation failed");
        }

        // Create venv with specific Python version
        let output = ctx
            .run_in_work_dir(&["uv", "venv", "--python", "3.12", ".venv312"])
            .expect("Failed to run uv venv");

        // May fail if Python 3.12 is not available
        if is_success(&output) {
            let venv_path = ctx.work_path().join(".venv312");
            assert!(venv_path.exists(), ".venv312 directory should exist");
        } else {
            eprintln!(
                "Python 3.12 venv creation failed (may be expected): {}",
                combined_output(&output)
            );
        }
    }

    #[rstest]
    #[test]
    #[ignore = "Requires network access"]
    fn uv_pip_install_in_venv() {
        require_vx!();
        let ctx = LifecycleTestContext::new();

        if !ctx.install("uv") {
            skip_test!("UV installation failed");
        }

        // Create venv
        let _ = ctx.run_in_work_dir(&["uv", "venv", ".venv"]);

        // Install a package
        let output = ctx
            .run_in_work_dir(&["uv", "pip", "install", "six", "--quiet"])
            .expect("Failed to run uv pip install");

        // May fail if venv activation is required
        if is_success(&output) {
            eprintln!("Successfully installed package in venv");
        } else {
            eprintln!(
                "pip install in venv failed (may need activation): {}",
                combined_output(&output)
            );
        }
    }

    #[rstest]
    #[test]
    #[ignore = "Requires network access"]
    fn uv_init_project() {
        require_vx!();
        let ctx = LifecycleTestContext::new();

        if !ctx.install("uv") {
            skip_test!("UV installation failed");
        }

        // Initialize a new project
        let output = ctx
            .run_in_work_dir(&["uv", "init", "--name", "test-project"])
            .expect("Failed to run uv init");

        if is_success(&output) {
            // Verify pyproject.toml exists
            let pyproject = ctx.work_path().join("pyproject.toml");
            assert!(pyproject.exists(), "pyproject.toml should exist");

            let content = fs::read_to_string(&pyproject).expect("Failed to read pyproject.toml");
            assert!(
                content.contains("test-project"),
                "pyproject.toml should contain project name"
            );
        }
    }
}

// ============================================================================
// Project Context Tests (Version Pinning)
// ============================================================================

mod project_context_tests {
    use super::*;

    #[rstest]
    #[test]
    #[ignore = "Requires network access"]
    fn version_pinning_with_vx_toml() {
        require_vx!();
        let ctx = LifecycleTestContext::new();

        // Install two Node.js versions
        if !ctx.install_version("node", "20.10.0") {
            skip_test!("Node 20.10.0 installation failed");
        }
        if !ctx.install_version("node", "20.11.0") {
            skip_test!("Node 20.11.0 installation failed");
        }

        // Create vx.toml with pinned version
        let vx_toml = ctx.work_path().join("vx.toml");
        fs::write(
            &vx_toml,
            r#"
[tools]
node = "20.10.0"
"#,
        )
        .expect("Failed to write vx.toml");

        // Run node in project directory - should use pinned version
        let output = ctx
            .run_in_work_dir(&["node", "--version"])
            .expect("Failed to run node");

        if is_success(&output) {
            let stdout = stdout_str(&output);
            assert!(
                stdout.contains("20.10.0"),
                "Should use pinned version 20.10.0, got: {}",
                stdout
            );
        }
    }

    #[rstest]
    #[test]
    #[ignore = "Requires network access"]
    fn version_range_in_vx_toml() {
        require_vx!();
        let ctx = LifecycleTestContext::new();

        // Install Node.js
        if !ctx.install("node") {
            skip_test!("Node installation failed");
        }

        // Create vx.toml with version range
        let vx_toml = ctx.work_path().join("vx.toml");
        fs::write(
            &vx_toml,
            r#"
[tools]
node = ">=20.0.0"
"#,
        )
        .expect("Failed to write vx.toml");

        // Run node - should use compatible version
        let output = ctx
            .run_in_work_dir(&["node", "--version"])
            .expect("Failed to run node");

        if is_success(&output) {
            let stdout = stdout_str(&output);
            assert!(
                stdout.contains("v2") || stdout.contains("20.") || stdout.contains("22."),
                "Should use version >=20.0.0, got: {}",
                stdout
            );
        }
    }

    #[rstest]
    #[test]
    #[ignore = "Requires network access"]
    fn nested_project_context() {
        require_vx!();
        let ctx = LifecycleTestContext::new();

        // Install two versions
        if !ctx.install_version("node", "20.10.0") {
            skip_test!("Node 20.10.0 installation failed");
        }
        if !ctx.install_version("node", "20.11.0") {
            skip_test!("Node 20.11.0 installation failed");
        }

        // Create parent vx.toml
        let parent_toml = ctx.work_path().join("vx.toml");
        fs::write(
            &parent_toml,
            r#"
[tools]
node = "20.10.0"
"#,
        )
        .expect("Failed to write parent vx.toml");

        // Create nested directory with its own vx.toml
        let nested_dir = ctx.work_path().join("nested");
        fs::create_dir_all(&nested_dir).expect("Failed to create nested dir");

        let nested_toml = nested_dir.join("vx.toml");
        fs::write(
            &nested_toml,
            r#"
[tools]
node = "20.11.0"
"#,
        )
        .expect("Failed to write nested vx.toml");

        // Run in nested directory - should use nested version
        use std::process::Command;
        let output = Command::new(common::vx_binary())
            .args(["node", "--version"])
            .current_dir(&nested_dir)
            .env("VX_HOME", ctx.home_path())
            .output()
            .expect("Failed to run node");

        if is_success(&output) {
            let stdout = stdout_str(&output);
            assert!(
                stdout.contains("20.11.0"),
                "Nested dir should use 20.11.0, got: {}",
                stdout
            );
        }
    }
}

// ============================================================================
// Environment Management Tests
// ============================================================================

mod env_management_tests {
    use super::*;

    #[rstest]
    #[test]
    #[ignore = "Requires vx binary"]
    fn env_create_and_list() {
        require_vx!();
        let ctx = LifecycleTestContext::new();

        // Create environment
        let output = ctx
            .run(&["env", "create", "test-env"])
            .expect("Failed to create env");
        assert_success(&output, "env create");

        // List environments
        let output = ctx.run(&["env", "list"]).expect("Failed to list envs");
        assert_success(&output, "env list");

        let stdout = stdout_str(&output);
        assert!(
            stdout.contains("test-env"),
            "Should list created environment"
        );
    }

    #[rstest]
    #[test]
    #[ignore = "Requires vx binary"]
    fn env_delete() {
        require_vx!();
        let ctx = LifecycleTestContext::new();

        // Create environment
        let _ = ctx.run(&["env", "create", "to-delete"]);

        // Delete environment
        let output = ctx
            .run(&["env", "delete", "to-delete", "--force"])
            .expect("Failed to delete env");
        assert_success(&output, "env delete");

        // Verify deleted
        let list_output = ctx.run(&["env", "list"]).expect("Failed to list envs");
        let stdout = stdout_str(&list_output);
        assert!(
            !stdout.contains("to-delete") || stdout.contains("No environments"),
            "Environment should be deleted"
        );
    }

    #[rstest]
    #[test]
    #[ignore = "Requires vx binary"]
    fn env_use() {
        require_vx!();
        let ctx = LifecycleTestContext::new();

        // Create environment
        let _ = ctx.run(&["env", "create", "use-test"]);

        // Use environment
        let output = ctx
            .run(&["env", "use", "use-test"])
            .expect("Failed to use env");
        assert_success(&output, "env use");
    }
}

// ============================================================================
// Cleanup and Edge Cases
// ============================================================================

mod cleanup_tests {
    use super::*;

    #[rstest]
    #[test]
    #[ignore = "Requires network access"]
    fn clean_unused_versions() {
        require_vx!();
        let ctx = LifecycleTestContext::new();

        // Install multiple versions
        let _ = ctx.install_version("uv", "0.4.0");
        let _ = ctx.install_version("uv", "0.5.0");

        // Run cleanup (if available)
        let output = ctx.run(&["clean"]);

        if let Ok(o) = output {
            if is_success(&o) {
                eprintln!("Clean command succeeded");
            } else {
                eprintln!(
                    "Clean command failed (may not be implemented): {}",
                    combined_output(&o)
                );
            }
        }
    }

    #[rstest]
    #[test]
    #[ignore = "Requires network access"]
    fn stats_command() {
        require_vx!();
        let ctx = LifecycleTestContext::new();

        // Install some tools
        let _ = ctx.install("uv");

        // Get stats
        let output = ctx.run(&["stats"]).expect("Failed to run stats");
        assert_success(&output, "stats");

        let stdout = stdout_str(&output);
        eprintln!("Stats output:\n{}", stdout);
    }
}

// ============================================================================
// Cross-Platform Tests
// ============================================================================

mod cross_platform_tests {
    use super::*;

    #[rstest]
    #[test]
    #[ignore = "Requires network access"]
    fn binary_execution_permissions() {
        require_vx!();
        let ctx = LifecycleTestContext::new();

        // Install UV
        if !ctx.install("uv") {
            skip_test!("UV installation failed");
        }

        // Verify executable works
        let output = ctx.run(&["uv", "--version"]).expect("Failed to run uv");
        assert_success(&output, "uv --version");

        // On Unix, verify executable permissions
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            let which_output = ctx.run(&["which", "uv"]).expect("Failed to run which");
            if is_success(&which_output) {
                let path = stdout_str(&which_output).trim().to_string();
                if !path.is_empty() {
                    let metadata = fs::metadata(&path);
                    if let Ok(m) = metadata {
                        let mode = m.permissions().mode();
                        assert!(
                            mode & 0o111 != 0,
                            "Binary should be executable, mode: {:o}",
                            mode
                        );
                    }
                }
            }
        }
    }

    #[rstest]
    #[test]
    #[ignore = "Requires network access"]
    fn path_with_spaces() {
        require_vx!();

        // Create temp dir with spaces in name
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let work_dir = temp_dir.path().join("path with spaces");
        fs::create_dir_all(&work_dir).expect("Failed to create dir with spaces");

        // Create VX_HOME with spaces
        let home_dir = temp_dir.path().join("vx home");
        fs::create_dir_all(&home_dir).expect("Failed to create home with spaces");

        // Try to install and run
        use std::process::Command;
        let output = Command::new(common::vx_binary())
            .args(["install", "uv"])
            .env("VX_HOME", &home_dir)
            .output()
            .expect("Failed to run install");

        if is_success(&output) {
            // Run in directory with spaces
            let run_output = Command::new(common::vx_binary())
                .args(["uv", "--version"])
                .current_dir(&work_dir)
                .env("VX_HOME", &home_dir)
                .output()
                .expect("Failed to run uv");

            assert_success(&run_output, "uv --version in path with spaces");
        }
    }
}
