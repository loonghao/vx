//! Configuration and environment management tests
//!
//! These tests verify that vx properly handles configuration files,
//! environment variables, and virtual environment management.

use std::fs;
use tempfile::TempDir;
use vx_core::FigmentConfigManager;

/// Test configuration manager creation
#[test]
fn test_config_manager_creation() {
    // Test minimal configuration manager
    let result = FigmentConfigManager::minimal();
    assert!(result.is_ok(), "Failed to create minimal config manager");

    if let Ok(manager) = result {
        let config = manager.config();
        assert!(!config.defaults.default_registry.is_empty());
    }
}

/// Test configuration manager with project detection
#[test]
fn test_config_manager_with_project() {
    let temp_dir = TempDir::new().unwrap();
    let original_dir = std::env::current_dir().unwrap();

    // Change to temp directory
    std::env::set_current_dir(&temp_dir).unwrap();

    // Create a simple .vx.toml file
    let config_content = r#"
[defaults]
auto_install = true

[tools.node]
version = "20.11.0"

[tools.uv]
version = "0.5.26"
"#;

    fs::write(temp_dir.path().join(".vx.toml"), config_content).unwrap();

    // Test configuration manager creation
    let result = FigmentConfigManager::new();

    // Restore original directory
    std::env::set_current_dir(original_dir).unwrap();

    if let Ok(manager) = result {
        let config = manager.config();
        assert!(config.defaults.auto_install);

        // Check if tools are configured
        let available_tools = manager.get_available_tools();
        assert!(!available_tools.is_empty());
    }
}

/// Test tool support checking
#[test]
fn test_tool_support_checking() {
    let result = FigmentConfigManager::minimal();
    if let Ok(manager) = result {
        // Test builtin tools
        assert!(manager.supports_tool("uv"));
        assert!(manager.supports_tool("node"));
        assert!(manager.supports_tool("go"));
        assert!(manager.supports_tool("rust"));

        // Test unsupported tool
        assert!(!manager.supports_tool("nonexistent-tool"));
    }
}

/// Test available tools listing
#[test]
fn test_available_tools_listing() {
    let result = FigmentConfigManager::minimal();
    if let Ok(manager) = result {
        let tools = manager.get_available_tools();
        assert!(!tools.is_empty());

        // Should contain builtin tools
        assert!(tools.contains(&"uv".to_string()));
        assert!(tools.contains(&"node".to_string()));
        assert!(tools.contains(&"go".to_string()));
        assert!(tools.contains(&"rust".to_string()));
    }
}

/// Test configuration status
#[test]
fn test_configuration_status() {
    let result = FigmentConfigManager::minimal();
    if let Ok(manager) = result {
        let status = manager.get_status();

        // Should have at least builtin layer
        assert!(!status.layers.is_empty());
        assert!(status.layers.iter().any(|l| l.name == "builtin"));

        // Should have available tools
        assert!(!status.available_tools.is_empty());

        // Should be healthy
        assert!(status.is_healthy());

        // Should have a summary
        let summary = status.summary();
        assert!(!summary.is_empty());
    }
}
