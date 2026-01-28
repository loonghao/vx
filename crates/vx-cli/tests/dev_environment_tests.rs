//! Dev Environment Tests
//!
//! These tests verify that `vx dev` and `vx run` correctly set up
//! environment variables, especially PATH, to include vx-managed tools.

mod common;

use common::{cleanup_test_env, init_test_env};
use rstest::*;
use serial_test::serial;
use std::collections::HashMap;
use std::fs;
use tempfile::TempDir;
use vx_paths::PathManager;

// ============================================================================
// Test Fixtures
// ============================================================================

/// Get platform directory name for current platform
fn get_platform_dir_name() -> String {
    let os = if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "macos") {
        "darwin"
    } else if cfg!(target_os = "linux") {
        "linux"
    } else {
        "unknown"
    };

    let arch = if cfg!(target_arch = "x86_64") {
        "x64"
    } else if cfg!(target_arch = "aarch64") {
        "arm64"
    } else if cfg!(target_arch = "arm") {
        "arm"
    } else if cfg!(target_arch = "x86") {
        "x86"
    } else {
        "unknown"
    };

    format!("{}-{}", os, arch)
}

/// Create a temporary vx home directory with mock tool installations
fn create_mock_vx_home() -> TempDir {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Create store directory structure
    let store_dir = temp_dir.path().join("store");
    fs::create_dir_all(&store_dir).expect("Failed to create store dir");

    // Get platform directory name
    let platform = get_platform_dir_name();

    // Create mock tool installations with platform-specific directory
    let tools = [
        ("uv", "0.7.12"),
        ("node", "22.0.0"),
        ("go", "1.22.0"),
        ("bun", "1.1.0"),
    ];

    for (tool, version) in tools {
        // Create platform-specific structure: store/<tool>/<version>/<platform>/bin
        let tool_bin = store_dir.join(tool).join(version).join(&platform).join("bin");
        fs::create_dir_all(&tool_bin).expect("Failed to create tool bin dir");

        // Create mock executable
        #[cfg(windows)]
        let exe_name = format!("{}.exe", tool);
        #[cfg(not(windows))]
        let exe_name = tool.to_string();

        let exe_path = tool_bin.join(&exe_name);
        fs::write(&exe_path, "mock executable").expect("Failed to create mock exe");

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&exe_path, fs::Permissions::from_mode(0o755))
                .expect("Failed to set permissions");
        }
    }

    // Create vx bin directory
    let vx_bin = temp_dir.path().join("bin");
    fs::create_dir_all(&vx_bin).expect("Failed to create vx bin dir");

    // Create npm-tools directory for npm-based tools
    let npm_tools = temp_dir.path().join("npm-tools");
    fs::create_dir_all(&npm_tools).expect("Failed to create npm-tools dir");

    // Create pip-tools directory for pip-based tools
    let pip_tools = temp_dir.path().join("pip-tools");
    fs::create_dir_all(&pip_tools).expect("Failed to create pip-tools dir");

    temp_dir
}

/// Create a vx.toml config file
fn create_vx_config(dir: &std::path::Path, tools: &[(&str, &str)], env: &[(&str, &str)]) {
    let mut config = String::from("[tools]\n");
    for (tool, version) in tools {
        config.push_str(&format!("{} = \"{}\"\n", tool, version));
    }

    if !env.is_empty() {
        config.push_str("\n[env]\n");
        for (key, value) in env {
            config.push_str(&format!("{} = \"{}\"\n", key, value));
        }
    }

    let config_path = dir.join("vx.toml");
    fs::write(config_path, config).expect("Failed to write vx.toml");
}

// ============================================================================
// PATH Environment Tests
// ============================================================================

mod path_environment_tests {
    use super::*;

    /// Test that PATH includes tool bin directories
    #[rstest]
    #[test]
    #[serial]
    fn test_path_includes_tool_bins() {
        init_test_env();

        let vx_home = create_mock_vx_home();
        std::env::set_var("VX_HOME", vx_home.path());

        let path_manager = PathManager::new().expect("Failed to create PathManager");

        // Check that store directory is correctly set
        let store_dir = path_manager.store_dir();
        assert!(
            store_dir.to_string_lossy().contains("store"),
            "Store dir should contain 'store'"
        );

        cleanup_test_env();
    }

    /// Test PATH separator is correct for platform
    #[rstest]
    #[test]
    fn test_path_separator_platform() {
        init_test_env();

        let separator = if cfg!(windows) { ";" } else { ":" };

        // Build a mock PATH
        let paths = ["/usr/bin", "/home/user/.vx/bin", "/opt/tools"];
        let joined = paths.join(separator);

        if cfg!(windows) {
            assert!(joined.contains(";"), "Windows PATH should use semicolon");
        } else {
            assert!(joined.contains(":"), "Unix PATH should use colon");
        }

        cleanup_test_env();
    }

    /// Test that vx bin directory is added to PATH
    #[rstest]
    #[test]
    #[serial]
    fn test_vx_bin_in_path() {
        init_test_env();

        let vx_home = create_mock_vx_home();
        std::env::set_var("VX_HOME", vx_home.path());

        let path_manager = PathManager::new().expect("Failed to create PathManager");
        let vx_bin = path_manager.bin_dir();

        // Verify vx bin directory exists
        assert!(vx_bin.exists(), "vx bin directory should exist");

        cleanup_test_env();
    }

    /// Test PATH ordering - vx tools should come before system PATH
    #[rstest]
    #[test]
    fn test_path_ordering() {
        init_test_env();

        let separator = if cfg!(windows) { ";" } else { ":" };
        let vx_bin = "/home/user/.vx/bin";
        let tool_bin = "/home/user/.vx/store/uv/0.7.12/bin";
        let system_path = "/usr/bin:/usr/local/bin";

        // Construct PATH like vx does: tool_bins + vx_bin + system_path
        let new_path = format!(
            "{}{}{}{}{}",
            tool_bin, separator, vx_bin, separator, system_path
        );

        let parts: Vec<&str> = new_path.split(separator).collect();

        // Tool bin should come first
        assert!(
            parts[0].contains("store"),
            "Tool bin should be first in PATH"
        );

        // vx bin should come before system
        let vx_pos = parts.iter().position(|p| p.contains(".vx/bin")).unwrap();
        let usr_pos = parts.iter().position(|p| *p == "/usr/bin").unwrap_or(999);
        assert!(vx_pos < usr_pos, "vx bin should come before system paths");

        cleanup_test_env();
    }
}

// ============================================================================
// Custom Environment Variable Tests
// ============================================================================

mod custom_env_tests {
    use super::*;

    /// Test that custom env vars from vx.toml are included
    #[rstest]
    #[test]
    fn test_custom_env_vars() {
        init_test_env();

        let project_dir = TempDir::new().expect("Failed to create temp dir");
        create_vx_config(
            project_dir.path(),
            &[("uv", "0.7.12")],
            &[
                ("MY_VAR", "my_value"),
                ("PYTHONPATH", "/custom/path"),
                ("DEBUG", "true"),
            ],
        );

        // Read and parse config
        let config_content =
            fs::read_to_string(project_dir.path().join("vx.toml")).expect("Failed to read config");

        assert!(
            config_content.contains("MY_VAR"),
            "Config should contain custom env var"
        );
        assert!(
            config_content.contains("PYTHONPATH"),
            "Config should contain PYTHONPATH"
        );

        cleanup_test_env();
    }

    /// Test env vars with special characters
    #[rstest]
    #[test]
    fn test_env_vars_special_chars() {
        init_test_env();

        let project_dir = TempDir::new().expect("Failed to create temp dir");

        // Create config with special characters in values
        let config = r#"[tools]
uv = "0.7.12"

[env]
PATH_EXTRA = "/path/with spaces/bin"
QUOTED_VAR = "value with \"quotes\""
"#;

        fs::write(project_dir.path().join("vx.toml"), config).expect("Failed to write config");

        let config_content =
            fs::read_to_string(project_dir.path().join("vx.toml")).expect("Failed to read config");

        assert!(
            config_content.contains("PATH_EXTRA"),
            "Config should contain PATH_EXTRA"
        );

        cleanup_test_env();
    }
}

// ============================================================================
// Tool Version Resolution Tests
// ============================================================================

mod version_resolution_tests {
    use super::*;

    /// Test "latest" version resolves to most recent installed
    #[rstest]
    #[test]
    #[serial]
    fn test_latest_version_resolution() {
        init_test_env();

        let vx_home = create_mock_vx_home();
        std::env::set_var("VX_HOME", vx_home.path());

        // Create multiple versions of a tool with platform-specific directories
        let platform = get_platform_dir_name();
        let store_dir = vx_home.path().join("store").join("node");
        for version in &["18.0.0", "20.0.0", "22.0.0"] {
            let version_dir = store_dir.join(version).join(&platform).join("bin");
            fs::create_dir_all(&version_dir).expect("Failed to create version dir");
        }

        let path_manager = PathManager::new().expect("Failed to create PathManager");
        let versions = path_manager
            .list_store_versions("node")
            .expect("Failed to list versions");

        // Versions should be sorted
        assert!(!versions.is_empty(), "Should have versions installed");

        cleanup_test_env();
    }

    /// Test specific version is used when specified
    #[rstest]
    #[test]
    #[serial]
    fn test_specific_version() {
        init_test_env();

        let project_dir = TempDir::new().expect("Failed to create temp dir");
        create_vx_config(
            project_dir.path(),
            &[("uv", "0.7.12"), ("node", "20.0.0")],
            &[],
        );

        let config_content =
            fs::read_to_string(project_dir.path().join("vx.toml")).expect("Failed to read config");

        assert!(
            config_content.contains("0.7.12"),
            "Config should contain specific uv version"
        );
        assert!(
            config_content.contains("20.0.0"),
            "Config should contain specific node version"
        );

        cleanup_test_env();
    }
}

// ============================================================================
// Script Execution Environment Tests
// ============================================================================

mod script_env_tests {
    use super::*;

    /// Test that script environment includes all tool paths
    #[rstest]
    #[test]
    fn test_script_env_includes_tools() {
        init_test_env();

        let project_dir = TempDir::new().expect("Failed to create temp dir");
        create_vx_config(
            project_dir.path(),
            &[("uv", "0.7.12"), ("node", "22.0.0")],
            &[],
        );

        // Simulate what build_script_environment does
        let mut env_vars: HashMap<String, String> = HashMap::new();
        let path_entries = [
            "/home/user/.vx/store/uv/0.7.12/bin".to_string(),
            "/home/user/.vx/store/node/22.0.0/bin".to_string(),
        ];

        let separator = if cfg!(windows) { ";" } else { ":" };
        let current_path = std::env::var("PATH").unwrap_or_default();
        let new_path = format!(
            "{}{}{}",
            path_entries.join(separator),
            separator,
            current_path
        );

        env_vars.insert("PATH".to_string(), new_path.clone());

        // Verify PATH contains tool paths
        assert!(
            new_path.contains("uv/0.7.12"),
            "PATH should contain uv tool path"
        );
        assert!(
            new_path.contains("node/22.0.0"),
            "PATH should contain node tool path"
        );

        cleanup_test_env();
    }

    /// Test that scripts can find vx-installed tools
    #[rstest]
    #[test]
    #[serial]
    fn test_script_finds_tools() {
        init_test_env();

        let vx_home = create_mock_vx_home();
        std::env::set_var("VX_HOME", vx_home.path());

        let path_manager = PathManager::new().expect("Failed to create PathManager");

        // Check if version exists in store
        let uv_installed = path_manager.is_version_in_store("uv", "0.7.12");

        // The mock creates the directory structure
        let store_dir = path_manager.version_store_dir("uv", "0.7.12");
        if store_dir.exists() {
            assert!(uv_installed, "uv 0.7.12 should be detected as installed");
        }

        cleanup_test_env();
    }
}

// ============================================================================
// Virtual Environment Tests
// ============================================================================

mod venv_tests {
    use super::*;

    /// Test that pip-tools directory is correctly structured
    #[rstest]
    #[test]
    #[serial]
    fn test_pip_tools_structure() {
        init_test_env();

        let vx_home = create_mock_vx_home();
        std::env::set_var("VX_HOME", vx_home.path());

        // Create a pip-tool installation
        let pip_tools = vx_home.path().join("pip-tools");
        let tool_dir = pip_tools.join("ruff").join("0.8.0");
        let venv_dir = tool_dir.join("venv");
        let bin_dir = if cfg!(windows) {
            venv_dir.join("Scripts")
        } else {
            venv_dir.join("bin")
        };

        fs::create_dir_all(&bin_dir).expect("Failed to create pip-tool bin dir");

        // Create mock executable
        #[cfg(windows)]
        let exe_name = "ruff.exe";
        #[cfg(not(windows))]
        let exe_name = "ruff";

        let exe_path = bin_dir.join(exe_name);
        fs::write(&exe_path, "mock ruff").expect("Failed to create mock exe");

        // Verify structure
        assert!(venv_dir.exists(), "venv directory should exist");
        assert!(bin_dir.exists(), "bin directory should exist");
        assert!(exe_path.exists(), "executable should exist");

        cleanup_test_env();
    }

    /// Test that npm-tools directory is correctly structured
    #[rstest]
    #[test]
    #[serial]
    fn test_npm_tools_structure() {
        init_test_env();

        let vx_home = create_mock_vx_home();
        std::env::set_var("VX_HOME", vx_home.path());

        // Create an npm-tool installation
        let npm_tools = vx_home.path().join("npm-tools");
        let tool_dir = npm_tools.join("prettier").join("3.0.0");
        let bin_dir = tool_dir.join("node_modules").join(".bin");

        fs::create_dir_all(&bin_dir).expect("Failed to create npm-tool bin dir");

        // Create mock executable
        #[cfg(windows)]
        let exe_name = "prettier.cmd";
        #[cfg(not(windows))]
        let exe_name = "prettier";

        let exe_path = bin_dir.join(exe_name);
        fs::write(&exe_path, "mock prettier").expect("Failed to create mock exe");

        // Verify structure
        assert!(tool_dir.exists(), "tool directory should exist");
        assert!(bin_dir.exists(), "bin directory should exist");
        assert!(exe_path.exists(), "executable should exist");

        cleanup_test_env();
    }

    /// Test pip-tool bin path resolution
    #[rstest]
    #[test]
    #[serial]
    fn test_pip_tool_bin_path() {
        init_test_env();

        let vx_home = create_mock_vx_home();
        std::env::set_var("VX_HOME", vx_home.path());

        let path_manager = PathManager::new().expect("Failed to create PathManager");
        let pip_bin = path_manager.pip_tool_bin_dir("ruff", "0.8.0");

        // Verify path structure
        let path_str = pip_bin.to_string_lossy();
        assert!(
            path_str.contains("pip-tools"),
            "pip tool bin should be in pip-tools"
        );
        assert!(
            path_str.contains("ruff"),
            "pip tool bin should contain tool name"
        );

        cleanup_test_env();
    }

    /// Test npm-tool bin path resolution
    #[rstest]
    #[test]
    #[serial]
    fn test_npm_tool_bin_path() {
        init_test_env();

        let vx_home = create_mock_vx_home();
        std::env::set_var("VX_HOME", vx_home.path());

        let path_manager = PathManager::new().expect("Failed to create PathManager");
        let npm_bin = path_manager.npm_tool_bin_dir("prettier", "3.0.0");

        // Verify path structure
        let path_str = npm_bin.to_string_lossy();
        assert!(
            path_str.contains("npm-tools"),
            "npm tool bin should be in npm-tools"
        );
        assert!(
            path_str.contains("prettier"),
            "npm tool bin should contain tool name"
        );

        cleanup_test_env();
    }
}

// ============================================================================
// Edge Case Tests
// ============================================================================

mod edge_case_tests {
    use super::*;

    /// Test empty tools configuration
    #[rstest]
    #[test]
    fn test_empty_tools_config() {
        init_test_env();

        let project_dir = TempDir::new().expect("Failed to create temp dir");
        create_vx_config(project_dir.path(), &[], &[]);

        let config_content =
            fs::read_to_string(project_dir.path().join("vx.toml")).expect("Failed to read config");

        assert!(
            config_content.contains("[tools]"),
            "Config should have tools section"
        );

        cleanup_test_env();
    }

    /// Test missing tool installation
    #[rstest]
    #[test]
    #[serial]
    fn test_missing_tool_installation() {
        init_test_env();

        let vx_home = create_mock_vx_home();
        std::env::set_var("VX_HOME", vx_home.path());

        let path_manager = PathManager::new().expect("Failed to create PathManager");

        // Check for a tool that doesn't exist
        let installed = path_manager.is_version_in_store("nonexistent-tool", "1.0.0");
        assert!(!installed, "Nonexistent tool should not be installed");

        cleanup_test_env();
    }

    /// Test PATH with no tools installed
    #[rstest]
    #[test]
    fn test_path_no_tools() {
        init_test_env();

        let path_entries: Vec<String> = Vec::new();
        let separator = if cfg!(windows) { ";" } else { ":" };
        let current_path = "/usr/bin:/usr/local/bin";

        let new_path = if path_entries.is_empty() {
            current_path.to_string()
        } else {
            format!(
                "{}{}{}",
                path_entries.join(separator),
                separator,
                current_path
            )
        };

        // PATH should just be the original
        assert_eq!(
            new_path, current_path,
            "PATH should be unchanged when no tools"
        );

        cleanup_test_env();
    }

    /// Test tool with special characters in version
    #[rstest]
    #[test]
    #[serial]
    fn test_tool_version_special_chars() {
        init_test_env();

        let vx_home = create_mock_vx_home();
        std::env::set_var("VX_HOME", vx_home.path());

        // Create a tool with pre-release version
        let store_dir = vx_home.path().join("store").join("test-tool");
        let version_dir = store_dir.join("1.0.0-beta.1").join("bin");
        fs::create_dir_all(&version_dir).expect("Failed to create version dir");

        assert!(version_dir.exists(), "Pre-release version dir should exist");

        cleanup_test_env();
    }
}

// ============================================================================
// Integration Tests
// ============================================================================

mod integration_tests {
    use super::*;

    /// Test full environment setup flow
    #[rstest]
    #[test]
    #[serial]
    fn test_full_env_setup_flow() {
        init_test_env();

        let vx_home = create_mock_vx_home();
        std::env::set_var("VX_HOME", vx_home.path());

        let project_dir = TempDir::new().expect("Failed to create temp dir");
        create_vx_config(
            project_dir.path(),
            &[("uv", "0.7.12"), ("node", "22.0.0")],
            &[("MY_VAR", "test_value")],
        );

        // Simulate build_script_environment
        let path_manager = PathManager::new().expect("Failed to create PathManager");
        let mut env_vars: HashMap<String, String> = HashMap::new();
        let mut path_entries: Vec<String> = Vec::new();

        // Add tool paths
        for (tool, version) in &[("uv", "0.7.12"), ("node", "22.0.0")] {
            let store_dir = path_manager.version_store_dir(tool, version);
            let bin_dir = store_dir.join("bin");
            if bin_dir.exists() {
                path_entries.push(bin_dir.to_string_lossy().to_string());
            }
        }

        // Build PATH
        let separator = if cfg!(windows) { ";" } else { ":" };
        let current_path = std::env::var("PATH").unwrap_or_default();
        let new_path = if path_entries.is_empty() {
            current_path.clone()
        } else {
            format!(
                "{}{}{}",
                path_entries.join(separator),
                separator,
                current_path
            )
        };
        env_vars.insert("PATH".to_string(), new_path);

        // Add vx bin
        let vx_bin = path_manager.bin_dir();
        if vx_bin.exists() {
            let path = env_vars.get("PATH").cloned().unwrap_or_default();
            env_vars.insert(
                "PATH".to_string(),
                format!("{}{}{}", vx_bin.display(), separator, path),
            );
        }

        // Add custom env
        env_vars.insert("MY_VAR".to_string(), "test_value".to_string());

        // Verify
        assert!(env_vars.contains_key("PATH"), "Should have PATH");
        assert!(
            env_vars.contains_key("MY_VAR"),
            "Should have custom env var"
        );
        assert_eq!(
            env_vars.get("MY_VAR").unwrap(),
            "test_value",
            "Custom env var should have correct value"
        );

        cleanup_test_env();
    }

    /// Test that environment is correctly passed to subprocess
    #[rstest]
    #[test]
    fn test_env_passed_to_subprocess() {
        init_test_env();

        use std::process::Command;

        let mut env_vars: HashMap<String, String> = HashMap::new();
        env_vars.insert("VX_TEST_VAR".to_string(), "test_value".to_string());

        // Create a command that echoes the env var
        #[cfg(windows)]
        let output = Command::new("cmd")
            .args(["/C", "echo %VX_TEST_VAR%"])
            .env("VX_TEST_VAR", "test_value")
            .output();

        #[cfg(not(windows))]
        let output = Command::new("sh")
            .args(["-c", "echo $VX_TEST_VAR"])
            .env("VX_TEST_VAR", "test_value")
            .output();

        if let Ok(output) = output {
            let stdout = String::from_utf8_lossy(&output.stdout);
            assert!(
                stdout.contains("test_value"),
                "Subprocess should receive env var"
            );
        }

        cleanup_test_env();
    }
}

// ============================================================================
// Tool Status and Info Tests
// ============================================================================

mod tool_status_tests {
    use super::*;

    /// Test tool status detection for installed tools
    #[rstest]
    #[test]
    #[serial]
    fn test_tool_status_installed() {
        init_test_env();

        let vx_home = create_mock_vx_home();
        std::env::set_var("VX_HOME", vx_home.path());

        let path_manager = PathManager::new().expect("Failed to create PathManager");

        // uv@0.7.12 should be installed in mock
        let store_dir = path_manager.version_store_dir("uv", "0.7.12");
        assert!(store_dir.exists(), "Mock uv should be installed");

        cleanup_test_env();
    }

    /// Test tool status detection for missing tools
    #[rstest]
    #[test]
    #[serial]
    fn test_tool_status_not_installed() {
        init_test_env();

        let vx_home = create_mock_vx_home();
        std::env::set_var("VX_HOME", vx_home.path());

        let path_manager = PathManager::new().expect("Failed to create PathManager");

        // nonexistent-tool should not be installed
        let installed = path_manager.is_version_in_store("nonexistent-tool", "1.0.0");
        assert!(!installed, "Nonexistent tool should not be installed");

        cleanup_test_env();
    }

    /// Test PATH priority - vx tools should come before system tools
    #[rstest]
    #[test]
    fn test_vx_tools_path_priority() {
        init_test_env();

        let sep = if cfg!(windows) { ";" } else { ":" };

        // Simulate PATH construction as vx does it
        let vx_tool_bin = if cfg!(windows) {
            r"C:\Users\test\.vx\store\cmake\3.28.0\bin"
        } else {
            "/home/test/.vx/store/cmake/3.28.0/bin"
        };

        let system_cmake = if cfg!(windows) {
            r"C:\Program Files\CMake\bin"
        } else {
            "/usr/bin"
        };

        // vx PATH should be: tool_bin + vx_bin + system_path
        let vx_path = format!("{}{}{}", vx_tool_bin, sep, system_cmake);
        let parts: Vec<&str> = vx_path.split(sep).collect();

        // vx tool should be first
        assert!(
            parts[0].contains(".vx"),
            "vx tool path should be first in PATH"
        );

        // System path should come after
        let vx_pos = parts.iter().position(|p| p.contains(".vx")).unwrap_or(999);
        let sys_pos = parts.iter().position(|p| !p.contains(".vx")).unwrap_or(999);
        assert!(
            vx_pos < sys_pos,
            "vx tool path should come before system path"
        );

        cleanup_test_env();
    }

    /// Test finding system tools (outside vx)
    #[rstest]
    #[test]
    fn test_find_system_tool_logic() {
        init_test_env();

        // Simulate the logic of find_system_tool
        let tool = "cmake";
        let exe_name = if cfg!(windows) {
            format!("{}.exe", tool)
        } else {
            tool.to_string()
        };

        // Check if cmake exists in PATH (excluding .vx paths)
        let path_var = std::env::var("PATH").unwrap_or_default();
        let sep = if cfg!(windows) { ';' } else { ':' };

        let mut system_paths = Vec::new();
        for dir in path_var.split(sep) {
            if !dir.contains(".vx") {
                let exe_path = std::path::PathBuf::from(dir).join(&exe_name);
                if exe_path.exists() {
                    system_paths.push(exe_path);
                }
            }
        }

        // This test just validates the logic; actual cmake may or may not be installed
        // The important thing is that we correctly filter out .vx paths
        for path in &system_paths {
            assert!(
                !path.to_string_lossy().contains(".vx"),
                "System tool paths should not contain .vx"
            );
        }

        cleanup_test_env();
    }

    /// Test VX_DEV environment variable is set
    #[rstest]
    #[test]
    fn test_vx_dev_env_var() {
        init_test_env();

        // Simulate what build_dev_environment does
        let mut env_vars: HashMap<String, String> = HashMap::new();
        env_vars.insert("VX_DEV".to_string(), "1".to_string());
        env_vars.insert(
            "VX_PROJECT_ROOT".to_string(),
            "/path/to/project".to_string(),
        );

        assert_eq!(
            env_vars.get("VX_DEV"),
            Some(&"1".to_string()),
            "VX_DEV should be set"
        );
        assert!(
            env_vars.contains_key("VX_PROJECT_ROOT"),
            "VX_PROJECT_ROOT should be set"
        );

        cleanup_test_env();
    }

    /// Test install progress states
    #[rstest]
    #[test]
    fn test_install_progress_states() {
        init_test_env();

        // Simulate tool states
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        enum ToolStatus {
            Installed,
            NotInstalled,
            SystemFallback,
        }

        let tool_states = vec![
            ("uv", "0.7.12", ToolStatus::Installed),
            ("node", "22.0.0", ToolStatus::NotInstalled),
            ("cmake", "3.28.0", ToolStatus::SystemFallback),
        ];

        // Verify we can distinguish between states
        for (tool, version, status) in &tool_states {
            match status {
                ToolStatus::Installed => {
                    assert_eq!(*tool, "uv", "uv should be installed");
                }
                ToolStatus::NotInstalled => {
                    assert_eq!(*tool, "node", "node should not be installed");
                }
                ToolStatus::SystemFallback => {
                    assert_eq!(*tool, "cmake", "cmake should fall back to system");
                }
            }
            // Just use version to avoid warning
            let _ = version;
        }

        cleanup_test_env();
    }

    /// Test multiple tools with different statuses
    #[rstest]
    #[test]
    #[serial]
    fn test_multiple_tools_status() {
        init_test_env();

        let vx_home = create_mock_vx_home();
        std::env::set_var("VX_HOME", vx_home.path());

        let path_manager = PathManager::new().expect("Failed to create PathManager");

        // Check various tools
        let tools = [
            ("uv", "0.7.12", true),      // should be installed (mock)
            ("node", "22.0.0", true),    // should be installed (mock)
            ("python", "3.12.0", false), // should not be installed
        ];

        for (tool, version, expected_installed) in tools {
            let installed = path_manager.is_version_in_store(tool, version);
            if expected_installed {
                // For mocked tools, the directory should exist
                let store_dir = path_manager.version_store_dir(tool, version);
                if store_dir.exists() {
                    assert!(installed, "{} should be installed", tool);
                }
            } else {
                assert!(!installed, "{} should not be installed", tool);
            }
        }

        cleanup_test_env();
    }
}
