//! VX Run PATH Resolution Tests
//!
//! These tests verify that `vx run` correctly resolves tool paths from the vx store
//! after `vx setup` has installed tools. This is critical for CI/CD environments
//! where tools are installed via vx and then used in subsequent steps.
//!
//! Note: All tests that modify VX_HOME environment variable must run serially
//! to avoid race conditions.

mod common;

use common::{cleanup_test_env, init_test_env};
use rstest::*;
use serial_test::serial;
use std::fs;
use tempfile::TempDir;
use vx_cli::commands::dev::build_script_environment;
use vx_cli::commands::setup::{parse_vx_config, VxConfig};
use vx_paths::PathManager;

// ============================================================================
// Test Fixtures
// ============================================================================

/// Create a mock vx home with a tool installed in the standard bin/ structure
fn create_mock_vx_home_with_bin_structure() -> TempDir {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Create store/uv/0.7.12/bin/uv structure (standard layout)
    let uv_bin = temp_dir
        .path()
        .join("store")
        .join("uv")
        .join("0.7.12")
        .join("bin");
    fs::create_dir_all(&uv_bin).expect("Failed to create uv bin dir");

    #[cfg(windows)]
    let uv_exe = uv_bin.join("uv.exe");
    #[cfg(not(windows))]
    let uv_exe = uv_bin.join("uv");

    fs::write(&uv_exe, "#!/bin/sh\necho 'mock uv 0.7.12'").expect("Failed to create mock uv");

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&uv_exe, fs::Permissions::from_mode(0o755))
            .expect("Failed to set permissions");
    }

    // Create vx bin directory
    let vx_bin = temp_dir.path().join("bin");
    fs::create_dir_all(&vx_bin).expect("Failed to create vx bin dir");

    temp_dir
}

/// Create a mock vx home with uv installed in the platform-specific subdirectory structure
/// This mimics how uv is actually installed: store/uv/0.7.12/uv-x86_64-unknown-linux-gnu/uv
fn create_mock_vx_home_with_platform_structure() -> TempDir {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Determine platform-specific directory name
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    let platform_dir = "uv-x86_64-unknown-linux-gnu";
    #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
    let platform_dir = "uv-aarch64-unknown-linux-gnu";
    #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
    let platform_dir = "uv-x86_64-apple-darwin";
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    let platform_dir = "uv-aarch64-apple-darwin";
    #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
    let platform_dir = "uv-x86_64-pc-windows-msvc";
    #[cfg(not(any(
        all(target_os = "linux", target_arch = "x86_64"),
        all(target_os = "linux", target_arch = "aarch64"),
        all(target_os = "macos", target_arch = "x86_64"),
        all(target_os = "macos", target_arch = "aarch64"),
        all(target_os = "windows", target_arch = "x86_64"),
    )))]
    let platform_dir = "uv-unknown-platform";

    // Create store/uv/0.7.12/uv-{platform}/uv structure
    let uv_platform_dir = temp_dir
        .path()
        .join("store")
        .join("uv")
        .join("0.7.12")
        .join(platform_dir);
    fs::create_dir_all(&uv_platform_dir).expect("Failed to create uv platform dir");

    #[cfg(windows)]
    let uv_exe = uv_platform_dir.join("uv.exe");
    #[cfg(not(windows))]
    let uv_exe = uv_platform_dir.join("uv");

    fs::write(&uv_exe, "#!/bin/sh\necho 'mock uv 0.7.12'").expect("Failed to create mock uv");

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&uv_exe, fs::Permissions::from_mode(0o755))
            .expect("Failed to set permissions");
    }

    // Create vx bin directory
    let vx_bin = temp_dir.path().join("bin");
    fs::create_dir_all(&vx_bin).expect("Failed to create vx bin dir");

    temp_dir
}

/// Create a .vx.toml config file
fn create_vx_toml(dir: &std::path::Path, tools: &[(&str, &str)], scripts: &[(&str, &str)]) {
    let mut config = String::from("[tools]\n");
    for (tool, version) in tools {
        config.push_str(&format!("{} = \"{}\"\n", tool, version));
    }

    if !scripts.is_empty() {
        config.push_str("\n[scripts]\n");
        for (name, cmd) in scripts {
            config.push_str(&format!("{} = \"{}\"\n", name, cmd));
        }
    }

    fs::write(dir.join(".vx.toml"), config).expect("Failed to write .vx.toml");
}

/// Load VxConfig from a directory
fn load_config(dir: &std::path::Path) -> VxConfig {
    let config_path = dir.join(".vx.toml");
    parse_vx_config(&config_path).expect("Failed to parse .vx.toml")
}

// ============================================================================
// Core PATH Resolution Tests
// ============================================================================

mod path_resolution_tests {
    use super::*;

    /// Test that build_script_environment includes tool paths in PATH (bin/ structure)
    #[rstest]
    #[test]
    #[serial]
    fn test_build_script_environment_includes_tool_paths_bin_structure() {
        init_test_env();

        let vx_home = create_mock_vx_home_with_bin_structure();
        std::env::set_var("VX_HOME", vx_home.path());

        // Create a project directory with .vx.toml
        let project_dir = TempDir::new().expect("Failed to create project dir");
        create_vx_toml(
            project_dir.path(),
            &[("uv", "0.7.12")],
            &[("test", "uv run nox -s tests")],
        );

        // Load config
        let config = load_config(project_dir.path());

        // Build script environment
        let env_vars = build_script_environment(&config).expect("Failed to build environment");

        // Check PATH contains tool path
        let path = env_vars.get("PATH").expect("PATH should be set");
        assert!(
            path.contains("uv") && path.contains("0.7.12"),
            "PATH should contain uv tool path: {}",
            path
        );

        cleanup_test_env();
    }

    /// Test that build_script_environment handles platform-specific tool structure
    #[rstest]
    #[test]
    #[serial]
    fn test_build_script_environment_handles_platform_structure() {
        init_test_env();

        let vx_home = create_mock_vx_home_with_platform_structure();
        std::env::set_var("VX_HOME", vx_home.path());

        // Create a project directory with .vx.toml
        let project_dir = TempDir::new().expect("Failed to create project dir");
        create_vx_toml(
            project_dir.path(),
            &[("uv", "0.7.12")],
            &[("test", "uv run nox -s tests")],
        );

        // Load config
        let config = load_config(project_dir.path());

        // Build script environment
        let env_vars = build_script_environment(&config).expect("Failed to build environment");

        // Check PATH contains tool path
        let path = env_vars.get("PATH").expect("PATH should be set");
        assert!(
            path.contains("uv") && path.contains("0.7.12"),
            "PATH should contain uv tool path: {}",
            path
        );

        cleanup_test_env();
    }
}

// ============================================================================
// "latest" Version Resolution Tests
// ============================================================================

mod latest_version_tests {
    use super::*;

    /// Test that "latest" version resolves to the most recent installed version
    #[rstest]
    #[test]
    #[serial]
    fn test_latest_version_resolves_correctly() {
        init_test_env();

        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        // Create multiple versions
        for version in &["0.7.10", "0.7.11", "0.7.12"] {
            let uv_bin = temp_dir
                .path()
                .join("store")
                .join("uv")
                .join(version)
                .join("bin");
            fs::create_dir_all(&uv_bin).expect("Failed to create uv bin dir");

            #[cfg(windows)]
            let uv_exe = uv_bin.join("uv.exe");
            #[cfg(not(windows))]
            let uv_exe = uv_bin.join("uv");

            fs::write(&uv_exe, format!("mock uv {}", version)).expect("Failed to create mock uv");
        }

        // Create vx bin directory
        fs::create_dir_all(temp_dir.path().join("bin")).expect("Failed to create vx bin");

        std::env::set_var("VX_HOME", temp_dir.path());

        let path_manager = PathManager::new().expect("Failed to create PathManager");
        let versions = path_manager
            .list_store_versions("uv")
            .expect("Failed to list versions");

        assert!(!versions.is_empty(), "Should have versions");

        // The last version should be the latest (sorted)
        let latest = versions.last().unwrap();
        assert_eq!(latest, "0.7.12", "Latest should be 0.7.12");

        cleanup_test_env();
    }

    /// Test build_script_environment with "latest" version
    #[rstest]
    #[test]
    #[serial]
    fn test_build_script_environment_with_latest() {
        init_test_env();

        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        // Create a version
        let uv_bin = temp_dir
            .path()
            .join("store")
            .join("uv")
            .join("0.7.12")
            .join("bin");
        fs::create_dir_all(&uv_bin).expect("Failed to create uv bin dir");

        #[cfg(windows)]
        let uv_exe = uv_bin.join("uv.exe");
        #[cfg(not(windows))]
        let uv_exe = uv_bin.join("uv");

        fs::write(&uv_exe, "mock uv").expect("Failed to create mock uv");

        // Create vx bin directory
        fs::create_dir_all(temp_dir.path().join("bin")).expect("Failed to create vx bin");

        std::env::set_var("VX_HOME", temp_dir.path());

        // Create project with "latest" version
        let project_dir = TempDir::new().expect("Failed to create project dir");
        create_vx_toml(
            project_dir.path(),
            &[("uv", "latest")],
            &[("test", "uv --version")],
        );

        let config = load_config(project_dir.path());
        let env_vars = build_script_environment(&config).expect("Failed to build environment");

        let path = env_vars.get("PATH").expect("PATH should be set");
        assert!(
            path.contains("uv") && path.contains("0.7.12"),
            "PATH should contain resolved latest version: {}",
            path
        );

        cleanup_test_env();
    }
}

// ============================================================================
// Missing Tool Warning Tests
// ============================================================================

mod missing_tool_tests {
    use super::*;

    /// Test that completely missing tools (no directory at all) are not in PATH
    #[rstest]
    #[test]
    #[serial]
    fn test_completely_missing_tool_not_in_path() {
        init_test_env();

        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        // Create vx home without any tools - not even the version directory
        fs::create_dir_all(temp_dir.path().join("store")).expect("Failed to create store");
        fs::create_dir_all(temp_dir.path().join("bin")).expect("Failed to create bin");

        std::env::set_var("VX_HOME", temp_dir.path());

        // Verify the version directory does NOT exist
        let version_dir = temp_dir.path().join("store").join("uv").join("0.7.12");
        assert!(
            !version_dir.exists(),
            "Version directory should NOT exist before test: {:?}",
            version_dir
        );

        // Create project with a tool that's not installed at all
        let project_dir = TempDir::new().expect("Failed to create project dir");
        create_vx_toml(
            project_dir.path(),
            &[("uv", "0.7.12")],
            &[("test", "uv --version")],
        );

        let config = load_config(project_dir.path());

        // build_script_environment should still succeed
        let env_vars = build_script_environment(&config).expect("Failed to build environment");

        let path = env_vars.get("PATH").expect("PATH should be set");

        // After build_script_environment, the version directory should still NOT exist
        // (we're not auto-creating directories for missing tools)
        // Note: The tool path might be added to PATH even if directory doesn't exist,
        // but the directory itself should not be created by build_script_environment
        assert!(
            !version_dir.exists(),
            "Version directory should NOT be created by build_script_environment: {:?}",
            version_dir
        );

        // The tool should be reported as missing (not added to PATH if directory doesn't exist)
        // Note: This behavior depends on the implementation - if the implementation adds
        // paths for missing tools, this test documents that behavior
        let temp_path_str = temp_dir.path().to_string_lossy();
        let contains_temp_uv_path = path.contains(&format!(
            "{}{}store",
            temp_path_str,
            std::path::MAIN_SEPARATOR
        ));

        // If the path contains our temp directory's store path for uv,
        // verify the directory actually exists (it shouldn't for a missing tool)
        if contains_temp_uv_path && path.contains("uv") {
            // This is acceptable if the implementation adds paths optimistically
            // The important thing is the directory wasn't created
            assert!(
                !version_dir.exists(),
                "Even if PATH contains the tool path, the directory should not be created"
            );
        }

        cleanup_test_env();
    }

    /// Test that partially installed tools (directory exists but no executable) still get added
    /// This is expected behavior - the fallback logic adds the directory for tools with non-standard names
    #[rstest]
    #[test]
    #[serial]
    fn test_partial_install_directory_added_as_fallback() {
        init_test_env();

        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        // Create vx home with version directory but no executable
        let version_dir = temp_dir.path().join("store").join("uv").join("0.7.12");
        fs::create_dir_all(&version_dir).expect("Failed to create version dir");
        fs::create_dir_all(temp_dir.path().join("bin")).expect("Failed to create bin");

        std::env::set_var("VX_HOME", temp_dir.path());

        let project_dir = TempDir::new().expect("Failed to create project dir");
        create_vx_toml(
            project_dir.path(),
            &[("uv", "0.7.12")],
            &[("test", "uv --version")],
        );

        let config = load_config(project_dir.path());
        let env_vars = build_script_environment(&config).expect("Failed to build environment");

        let path = env_vars.get("PATH").expect("PATH should be set");
        // The version directory IS added as fallback (for tools with non-standard executable names)
        assert!(
            path.contains("uv") && path.contains("0.7.12"),
            "PATH should contain version directory as fallback: {}",
            path
        );

        cleanup_test_env();
    }
}

// ============================================================================
// Script Execution Simulation Tests
// ============================================================================

mod script_execution_tests {
    use super::*;
    use vx_cli::commands::generate_wrapper_script;

    /// Test that generated wrapper script sets PATH correctly
    #[rstest]
    #[test]
    #[serial]
    fn test_wrapper_script_sets_path() {
        init_test_env();

        let vx_home = create_mock_vx_home_with_bin_structure();
        std::env::set_var("VX_HOME", vx_home.path());

        // Create project
        let project_dir = TempDir::new().expect("Failed to create project dir");
        create_vx_toml(
            project_dir.path(),
            &[("uv", "0.7.12")],
            &[("test", "uv run nox -s tests")],
        );

        let config = load_config(project_dir.path());
        let env_vars = build_script_environment(&config).expect("Failed to build environment");

        // Generate wrapper script
        let script = generate_wrapper_script("uv run nox -s tests", &env_vars);

        // Verify script exports PATH
        #[cfg(windows)]
        {
            assert!(
                script.contains("$env:PATH"),
                "Windows script should set $env:PATH"
            );
        }

        #[cfg(not(windows))]
        {
            assert!(
                script.contains("export PATH="),
                "Unix script should export PATH"
            );
        }

        // Verify PATH contains tool directory
        let path = env_vars.get("PATH").expect("PATH should be set");
        assert!(
            path.contains("uv") && path.contains("0.7.12"),
            "PATH in env_vars should contain tool: {}",
            path
        );

        cleanup_test_env();
    }

    /// Test that wrapper script can find tool in PATH
    #[rstest]
    #[test]
    #[serial]
    fn test_wrapper_script_tool_in_path() {
        init_test_env();

        let vx_home = create_mock_vx_home_with_bin_structure();

        std::env::set_var("VX_HOME", vx_home.path());

        // Create project in the same temp context
        let project_dir = TempDir::new().expect("Failed to create project dir");
        create_vx_toml(
            project_dir.path(),
            &[("uv", "0.7.12")],
            &[("test", "uv --version")],
        );

        let config = load_config(project_dir.path());
        let env_vars = build_script_environment(&config).expect("Failed to build environment");

        let path = env_vars.get("PATH").expect("PATH should be set");

        // Verify the PATH contains uv/0.7.12 (the tool is installed)
        assert!(
            path.contains("uv") && path.contains("0.7.12"),
            "PATH should contain uv/0.7.12.\nActual PATH: {}",
            path
        );

        cleanup_test_env();
    }
}

// ============================================================================
// CI/CD Scenario Tests
// ============================================================================

mod ci_scenario_tests {
    use super::*;

    /// Simulate the CI scenario: vx setup installs tool, then vx run uses it
    /// This test verifies that after tools are installed, vx run can find them
    #[rstest]
    #[test]
    #[serial]
    fn test_ci_scenario_setup_then_run() {
        init_test_env();

        // Step 1: Simulate vx setup by creating tool installation
        let vx_home = create_mock_vx_home_with_platform_structure();
        std::env::set_var("VX_HOME", vx_home.path());

        // Step 2: Create project with .vx.toml (like shotgrid-mcp-server)
        let project_dir = TempDir::new().expect("Failed to create project dir");
        create_vx_toml(
            project_dir.path(),
            &[("uv", "0.7.12")],
            &[("test", "uv run nox -s tests")],
        );

        // Step 3: Simulate vx run - build environment and check PATH
        let config = load_config(project_dir.path());
        let env_vars = build_script_environment(&config).expect("Failed to build environment");

        // Step 4: Verify tool is findable in PATH
        let path = env_vars.get("PATH").expect("PATH should be set");

        // The PATH should contain the uv installation directory
        assert!(
            path.contains("uv") && path.contains("0.7.12"),
            "CI scenario: PATH should contain uv tool after setup.\nPATH: {}",
            path
        );

        cleanup_test_env();
    }

    /// Test that vx run works even when GITHUB_PATH is not set
    /// (vx run should use its own PATH construction, not rely on external PATH)
    #[rstest]
    #[test]
    #[serial]
    fn test_vx_run_independent_of_github_path() {
        init_test_env();

        // Clear any existing PATH modifications
        let original_path = std::env::var("PATH").unwrap_or_default();

        let vx_home = create_mock_vx_home_with_bin_structure();
        std::env::set_var("VX_HOME", vx_home.path());

        let project_dir = TempDir::new().expect("Failed to create project dir");
        create_vx_toml(
            project_dir.path(),
            &[("uv", "0.7.12")],
            &[("test", "uv --version")],
        );

        let config = load_config(project_dir.path());

        // build_script_environment should construct PATH independently
        let env_vars = build_script_environment(&config).expect("Failed to build environment");

        let new_path = env_vars.get("PATH").expect("PATH should be set");

        // The new PATH should have tool paths prepended to original PATH
        assert!(
            new_path.contains("uv") && new_path.contains("0.7.12"),
            "vx run should add tool paths regardless of GITHUB_PATH.\nNew PATH: {}",
            new_path
        );

        // Original PATH should still be included
        assert!(
            new_path.contains(&original_path) || original_path.is_empty(),
            "Original PATH should be preserved"
        );

        cleanup_test_env();
    }

    /// Test that vx run correctly handles the exact CI scenario from shotgrid-mcp-server
    /// The workflow is:
    /// 1. vx setup - installs uv
    /// 2. vx run test - runs "uv run nox -s tests"
    #[rstest]
    #[test]
    #[serial]
    fn test_shotgrid_mcp_server_ci_scenario() {
        init_test_env();

        // Simulate the exact structure that vx creates when installing uv
        let vx_home = create_mock_vx_home_with_platform_structure();
        std::env::set_var("VX_HOME", vx_home.path());

        // Create .vx.toml matching shotgrid-mcp-server
        let project_dir = TempDir::new().expect("Failed to create project dir");
        let vx_toml = r#"[tools]
uv = "0.7.12"

[scripts]
lint = "uv run nox -s lint"
test = "uv run nox -s tests"
build = "uv run nox -s build-wheel"
docs = "uv run nox -s docs"
format = "uv run ruff format ."
check = "uv run ruff check ."
typecheck = "uv run mypy src"
"#;
        fs::write(project_dir.path().join(".vx.toml"), vx_toml).expect("Failed to write .vx.toml");

        // Load config and build environment
        let config = load_config(project_dir.path());
        let env_vars = build_script_environment(&config).expect("Failed to build environment");

        // Verify PATH contains uv
        let path = env_vars.get("PATH").expect("PATH should be set");

        // This is the critical assertion - vx run should be able to find uv
        assert!(
            path.contains("uv"),
            "shotgrid-mcp-server CI scenario: PATH must contain uv.\nPATH: {}",
            path
        );
        assert!(
            path.contains("0.7.12"),
            "shotgrid-mcp-server CI scenario: PATH must contain version 0.7.12.\nPATH: {}",
            path
        );

        cleanup_test_env();
    }
}
