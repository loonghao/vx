//! Integration tests using examples directory
//!
//! These tests verify that vx correctly handles real-world configurations
//! from the examples/ directory.

mod common;

use common::{cleanup_test_env, init_test_env};
use rstest::*;
use std::path::PathBuf;
use vx_cli::commands::dev::build_script_environment;
use vx_cli::commands::setup::{ConfigView, parse_vx_config};
use vx_config::ScriptConfig;

// ============================================================================
// Helper Functions
// ============================================================================

/// Get the path to the examples directory
fn examples_dir() -> PathBuf {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    PathBuf::from(manifest_dir)
        .parent() // crates/
        .unwrap()
        .parent() // project root
        .unwrap()
        .join("examples")
}

/// Load ConfigView from a vx.toml file
fn load_config_from_file(config_path: &std::path::Path) -> ConfigView {
    parse_vx_config(config_path)
        .unwrap_or_else(|_| panic!("Failed to parse config: {}", config_path.display()))
}

// ============================================================================
// TaskMatrix Example Tests
// ============================================================================

mod taskmatrix_tests {
    use super::*;

    /// Test that taskmatrix vx.toml can be parsed correctly
    #[rstest]
    #[test]
    fn test_taskmatrix_config_parsing() {
        let config_path = examples_dir().join("taskmatrix").join("vx.toml");

        if !config_path.exists() {
            eprintln!("Skipping test: {} not found", config_path.display());
            return;
        }

        let config = load_config_from_file(&config_path);

        // Verify tools are parsed
        assert!(
            config.tools.contains_key("uv"),
            "taskmatrix should have uv tool"
        );

        // Verify scripts are parsed
        assert!(
            config.scripts.contains_key("install"),
            "taskmatrix should have install script"
        );
        assert!(
            config.scripts.contains_key("lint"),
            "taskmatrix should have lint script"
        );
    }

    /// Test that taskmatrix environment can be built
    #[rstest]
    #[test]
    fn test_taskmatrix_build_environment() {
        init_test_env();

        let config_path = examples_dir().join("taskmatrix").join("vx.toml");

        if !config_path.exists() {
            eprintln!("Skipping test: {} not found", config_path.display());
            cleanup_test_env();
            return;
        }

        let config = load_config_from_file(&config_path);

        // build_script_environment should succeed even if tools aren't installed
        let result = build_script_environment(&config);
        assert!(
            result.is_ok(),
            "build_script_environment should succeed: {:?}",
            result.err()
        );

        let env_vars = result.unwrap();

        // PATH should be set
        assert!(env_vars.contains_key("PATH"), "PATH should be set");

        // Custom env vars from config should be set
        assert!(
            env_vars.contains_key("PYTHONUNBUFFERED"),
            "PYTHONUNBUFFERED should be set from config"
        );

        cleanup_test_env();
    }

    /// Test that taskmatrix scripts are correctly parsed
    #[rstest]
    #[test]
    fn test_taskmatrix_scripts() {
        let config_path = examples_dir().join("taskmatrix").join("vx.toml");

        if !config_path.exists() {
            eprintln!("Skipping test: {} not found", config_path.display());
            return;
        }

        let config = load_config_from_file(&config_path);

        // Check simple scripts
        assert_eq!(
            config.scripts.get("install"),
            Some(&ScriptConfig::Simple(
                "uv pip install -r requirements.txt".to_string()
            )),
            "install script should be correct"
        );

        assert_eq!(
            config.scripts.get("lint"),
            Some(&ScriptConfig::Simple("uvx ruff check .".to_string())),
            "lint script should be correct"
        );
    }
}

// ============================================================================
// Extension Examples Tests
// ============================================================================

mod extension_tests {
    use super::*;
    use std::fs;

    /// Test that hello-world extension config can be parsed
    #[rstest]
    #[test]
    fn test_hello_world_extension_exists() {
        let extension_path = examples_dir()
            .join("extensions")
            .join("hello-world")
            .join("vx-extension.toml");

        assert!(
            extension_path.exists(),
            "hello-world extension should exist at {}",
            extension_path.display()
        );

        // Read and verify basic structure
        let content = fs::read_to_string(&extension_path).expect("Failed to read extension config");
        assert!(
            content.contains("[extension]"),
            "Should have [extension] section"
        );
        assert!(
            content.contains("name = \"hello-world\""),
            "Should have correct name"
        );
    }

    /// Test that project-info extension config can be parsed
    #[rstest]
    #[test]
    fn test_project_info_extension_exists() {
        let extension_path = examples_dir()
            .join("extensions")
            .join("project-info")
            .join("vx-extension.toml");

        assert!(
            extension_path.exists(),
            "project-info extension should exist at {}",
            extension_path.display()
        );

        // Read and verify basic structure
        let content = fs::read_to_string(&extension_path).expect("Failed to read extension config");
        assert!(
            content.contains("[extension]"),
            "Should have [extension] section"
        );
        assert!(
            content.contains("name = \"project-info\""),
            "Should have correct name"
        );
    }

    /// Test that extension scripts exist
    #[rstest]
    #[test]
    fn test_extension_scripts_exist() {
        let hello_world_dir = examples_dir().join("extensions").join("hello-world");

        // Check Python scripts
        assert!(
            hello_world_dir.join("main.py").exists(),
            "main.py should exist"
        );
        assert!(
            hello_world_dir.join("greet.py").exists(),
            "greet.py should exist"
        );
        assert!(
            hello_world_dir.join("info.py").exists(),
            "info.py should exist"
        );

        let project_info_dir = examples_dir().join("extensions").join("project-info");

        // Check JavaScript scripts
        assert!(
            project_info_dir.join("main.js").exists(),
            "main.js should exist"
        );
        assert!(
            project_info_dir.join("deps.js").exists(),
            "deps.js should exist"
        );
        assert!(
            project_info_dir.join("size.js").exists(),
            "size.js should exist"
        );
    }
}

// ============================================================================
// Config Parsing Robustness Tests
// ============================================================================

mod config_robustness_tests {
    use super::*;

    /// Test that all example configs can be parsed without errors
    #[rstest]
    #[test]
    fn test_all_example_configs_parseable() {
        let examples = examples_dir();

        // Find all vx.toml files
        let vx_toml_files = find_vx_toml_files(&examples);

        for config_path in vx_toml_files {
            let result = parse_vx_config(&config_path);
            assert!(
                result.is_ok(),
                "Failed to parse {}: {:?}",
                config_path.display(),
                result.err()
            );
        }
    }

    /// Find all vx.toml files in a directory recursively
    fn find_vx_toml_files(dir: &std::path::Path) -> Vec<PathBuf> {
        let mut files = Vec::new();

        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                if path.is_dir() {
                    files.extend(find_vx_toml_files(&path));
                } else if path.file_name() == Some(std::ffi::OsStr::new("vx.toml")) {
                    files.push(path);
                }
            }
        }

        files
    }
}
