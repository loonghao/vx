//! Comprehensive tests for configuration management functionality

use super::config_figment::*;
use std::collections::HashMap;
use std::fs;
use tempfile::TempDir;

/// Test configuration validation functionality
#[test]
fn test_config_validation() {
    let manager = FigmentConfigManager::minimal().expect("Failed to create config manager");

    // Test validation with default config
    let warnings = manager.validate().expect("Validation should succeed");

    // Default config should be valid with minimal warnings
    assert!(
        warnings.len() <= 1,
        "Too many warnings in default config: {:?}",
        warnings
    );
}

/// Test configuration validation with problematic config
#[test]
fn test_config_validation_with_issues() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let original_dir = std::env::current_dir().expect("Failed to get current dir");

    // Change to temp directory
    std::env::set_current_dir(temp_dir.path()).expect("Failed to change dir");

    // Create a problematic .vx.toml
    let config_content = r#"
[tools.empty-version]
version = ""

[tools.missing-placeholders]
custom_sources = { "bad-source" = "http://example.com/download" }

[registries.empty-url]
base_url = ""

[defaults]
update_interval = ""
"#;

    fs::write(".vx.toml", config_content).expect("Failed to write config");

    // Test validation
    let manager = FigmentConfigManager::new().expect("Failed to create config manager");
    let warnings = manager.validate().expect("Validation should succeed");

    // Should detect multiple issues
    assert!(
        warnings.len() >= 3,
        "Should detect multiple validation issues: {:?}",
        warnings
    );

    // Check specific warnings
    assert!(warnings.iter().any(|w| w.contains("empty version")));
    assert!(warnings.iter().any(|w| w.contains("empty base URL")));
    assert!(warnings
        .iter()
        .any(|w| w.contains("Update interval is empty")));

    // Restore original directory
    std::env::set_current_dir(original_dir).expect("Failed to restore dir");
}

/// Test project configuration initialization
#[test]
fn test_project_config_initialization() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let original_dir = std::env::current_dir().expect("Failed to get current dir");

    // Change to temp directory
    std::env::set_current_dir(temp_dir.path()).expect("Failed to change dir");

    let manager = FigmentConfigManager::minimal().expect("Failed to create config manager");

    // Test initialization with specific tools
    let mut tools = HashMap::new();
    tools.insert("node".to_string(), "18.17.0".to_string());
    tools.insert("python".to_string(), "3.11.5".to_string());

    let result = manager.init_project_config(Some(tools), false);
    assert!(
        result.is_ok(),
        "Project config initialization should succeed"
    );

    // Check that .vx.toml was created
    assert!(
        std::path::Path::new(".vx.toml").exists(),
        ".vx.toml should be created"
    );

    // Read and verify content
    let content = fs::read_to_string(".vx.toml").expect("Failed to read .vx.toml");
    assert!(content.contains("node = \"18.17.0\""));
    assert!(content.contains("python = \"3.11.5\""));
    assert!(content.contains("auto_install = true"));
    assert!(content.contains("VX Project Configuration"));

    // Test that second initialization fails
    let result2 = manager.init_project_config(None, false);
    assert!(result2.is_err(), "Second initialization should fail");

    // Restore original directory
    std::env::set_current_dir(original_dir).expect("Failed to restore dir");
}

/// Test project tool version retrieval
#[test]
fn test_project_tool_version_retrieval() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let original_dir = std::env::current_dir().expect("Failed to get current dir");

    // Change to temp directory
    std::env::set_current_dir(temp_dir.path()).expect("Failed to change dir");

    // Create a .vx.toml with tool versions (using project config format)
    let config_content = r#"
[tools]
node = "18.17.0"
python = "3.11.5"
go = "1.21.6"

[settings]
auto_install = true
cache_duration = "7d"
"#;

    fs::write(".vx.toml", config_content).expect("Failed to write config");

    let manager = FigmentConfigManager::new().expect("Failed to create config manager");

    // Test tool version retrieval
    assert_eq!(
        manager.get_project_tool_version("node"),
        Some("18.17.0".to_string())
    );
    assert_eq!(
        manager.get_project_tool_version("python"),
        Some("3.11.5".to_string())
    );
    assert_eq!(
        manager.get_project_tool_version("go"),
        Some("1.21.6".to_string())
    );
    assert_eq!(manager.get_project_tool_version("nonexistent"), None);

    // Restore original directory
    std::env::set_current_dir(original_dir).expect("Failed to restore dir");
}

/// Test configuration status reporting
#[test]
fn test_configuration_status() {
    let manager = FigmentConfigManager::minimal().expect("Failed to create config manager");
    let status = manager.get_status();

    // Should have at least builtin layer
    assert!(!status.layers.is_empty());
    assert!(status.layers.iter().any(|l| l.name == "builtin"));

    // Should have available tools
    assert!(!status.available_tools.is_empty());

    // Should be healthy
    assert!(status.is_healthy());

    // Test summary
    let summary = status.summary();
    assert!(!summary.is_empty());
    assert!(summary.contains("Configuration layers"));
    assert!(summary.contains("Tools"));
}

/// Test project sync functionality
#[tokio::test]
async fn test_project_sync() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let original_dir = std::env::current_dir().expect("Failed to get current dir");

    // Change to temp directory
    std::env::set_current_dir(temp_dir.path()).expect("Failed to change dir");

    // Create a .vx.toml with tool versions
    let config_content = r#"
[tools]
node = "18.17.0"
python = "3.11.5"

[settings]
auto_install = true
"#;

    fs::write(".vx.toml", config_content).expect("Failed to write config");

    let manager = FigmentConfigManager::new().expect("Failed to create config manager");

    // Test sync (this will not actually install tools in test environment)
    let result = manager.sync_project(false).await;
    assert!(result.is_ok(), "Project sync should succeed");

    let installed_tools = result.unwrap();
    // Should identify tools that would be installed
    assert!(installed_tools.iter().any(|t| t.contains("node@18.17.0")));
    assert!(installed_tools.iter().any(|t| t.contains("python@3.11.5")));

    // Restore original directory
    std::env::set_current_dir(original_dir).expect("Failed to restore dir");
}

/// Test configuration with environment variables
#[test]
fn test_config_with_environment_variables() {
    // Set environment variables
    std::env::set_var("VX_DEFAULTS_AUTO_INSTALL", "false");
    std::env::set_var("VX_TOOLS_NODE_VERSION", "20.10.0");

    let manager = FigmentConfigManager::new().expect("Failed to create config manager");
    let config = manager.config();

    // Environment variables should override defaults
    assert!(!config.defaults.auto_install);

    // Clean up environment variables
    std::env::remove_var("VX_DEFAULTS_AUTO_INSTALL");
    std::env::remove_var("VX_TOOLS_NODE_VERSION");
}

/// Test tool configuration retrieval
#[test]
fn test_tool_configuration() {
    let manager = FigmentConfigManager::minimal().expect("Failed to create config manager");

    // Test getting tool config for non-existent tool
    assert!(manager.get_tool_config("nonexistent").is_none());

    // Test available tools listing
    let available_tools = manager.get_available_tools();
    assert!(!available_tools.is_empty());
    assert!(available_tools.contains(&"node".to_string()));
    assert!(available_tools.contains(&"python".to_string()));
}

/// Test configuration error handling
#[test]
fn test_config_error_handling() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let original_dir = std::env::current_dir().expect("Failed to get current dir");

    // Change to temp directory
    std::env::set_current_dir(temp_dir.path()).expect("Failed to change dir");

    // Create invalid TOML
    let invalid_config = r#"
[tools
invalid toml syntax
"#;

    fs::write(".vx.toml", invalid_config).expect("Failed to write config");

    // Should handle invalid TOML gracefully
    let result = FigmentConfigManager::new();
    assert!(result.is_err(), "Should fail with invalid TOML");

    // Restore original directory
    std::env::set_current_dir(original_dir).expect("Failed to restore dir");
}
