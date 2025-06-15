//! Comprehensive tests for virtual environment management

use super::venv::*;
use std::fs;
use tempfile::TempDir;

/// Test VenvManager creation and basic functionality
#[test]
fn test_venv_manager_creation() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    
    // Set VX_HOME to temp directory
    std::env::set_var("VX_HOME", temp_dir.path());
    
    let result = VenvManager::new();
    assert!(result.is_ok(), "VenvManager creation should succeed");
    
    // Clean up
    std::env::remove_var("VX_HOME");
}

/// Test project configuration loading
#[test]
fn test_project_config_loading() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let original_dir = std::env::current_dir().expect("Failed to get current dir");
    
    // Change to temp directory
    std::env::set_current_dir(temp_dir.path()).expect("Failed to change dir");
    
    // Set VX_HOME to temp directory
    std::env::set_var("VX_HOME", temp_dir.path());
    
    let manager = VenvManager::new().expect("Failed to create VenvManager");
    
    // Test loading non-existent config
    let config = manager.load_project_config().expect("Should handle missing config");
    assert!(config.is_none(), "Should return None for missing config");
    
    // Create a .vx.toml file
    let config_content = r#"
[tools]
node = "18.17.0"
python = "3.11.5"

[settings]
auto_install = true
cache_duration = "7d"
"#;
    
    fs::write(".vx.toml", config_content).expect("Failed to write config");
    
    // Test loading existing config
    let config = manager.load_project_config().expect("Should load config");
    assert!(config.is_some(), "Should return Some for existing config");
    
    let config = config.unwrap();
    assert_eq!(config.tools.get("node"), Some(&"18.17.0".to_string()));
    assert_eq!(config.tools.get("python"), Some(&"3.11.5".to_string()));
    assert!(config.settings.auto_install);
    assert_eq!(config.settings.cache_duration, "7d");
    
    // Restore original directory and clean up
    std::env::set_current_dir(original_dir).expect("Failed to restore dir");
    std::env::remove_var("VX_HOME");
}

/// Test project tool version retrieval
#[tokio::test]
async fn test_project_tool_version() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let original_dir = std::env::current_dir().expect("Failed to get current dir");
    
    // Change to temp directory
    std::env::set_current_dir(temp_dir.path()).expect("Failed to change dir");
    
    // Set VX_HOME to temp directory
    std::env::set_var("VX_HOME", temp_dir.path());
    
    let manager = VenvManager::new().expect("Failed to create VenvManager");
    
    // Test with no config
    let version = manager.get_project_tool_version("node").await.expect("Should handle missing config");
    assert!(version.is_none(), "Should return None for missing config");
    
    // Create a .vx.toml file
    let config_content = r#"
[tools]
node = "18.17.0"
python = "3.11.5"

[settings]
auto_install = true
"#;
    
    fs::write(".vx.toml", config_content).expect("Failed to write config");
    
    // Test with existing config
    let version = manager.get_project_tool_version("node").await.expect("Should load version");
    assert_eq!(version, Some("18.17.0".to_string()));
    
    let version = manager.get_project_tool_version("python").await.expect("Should load version");
    assert_eq!(version, Some("3.11.5".to_string()));
    
    let version = manager.get_project_tool_version("nonexistent").await.expect("Should handle missing tool");
    assert!(version.is_none(), "Should return None for non-existent tool");
    
    // Restore original directory and clean up
    std::env::set_current_dir(original_dir).expect("Failed to restore dir");
    std::env::remove_var("VX_HOME");
}

/// Test ProjectConfig default implementation
#[test]
fn test_project_config_defaults() {
    let config = ProjectConfig::default();
    
    assert!(config.tools.is_empty(), "Default tools should be empty");
    assert!(config.settings.auto_install, "Default auto_install should be true");
    assert_eq!(config.settings.cache_duration, "7d", "Default cache_duration should be 7d");
}

/// Test ProjectSettings default implementation
#[test]
fn test_project_settings_defaults() {
    let settings = ProjectSettings::default();
    
    assert!(settings.auto_install, "Default auto_install should be true");
    assert_eq!(settings.cache_duration, "7d", "Default cache_duration should be 7d");
}

/// Test VenvConfig creation and serialization
#[test]
fn test_venv_config() {
    use std::collections::HashMap;
    
    let mut tools = HashMap::new();
    tools.insert("node".to_string(), "18.17.0".to_string());
    tools.insert("python".to_string(), "3.11.5".to_string());
    
    let config = VenvConfig {
        name: "test-env".to_string(),
        tools,
        created_at: chrono::Utc::now(),
        modified_at: chrono::Utc::now(),
        is_active: false,
    };
    
    // Test serialization
    let serialized = toml::to_string(&config);
    assert!(serialized.is_ok(), "VenvConfig should serialize to TOML");
    
    // Test deserialization
    let toml_str = serialized.unwrap();
    let deserialized: Result<VenvConfig, _> = toml::from_str(&toml_str);
    assert!(deserialized.is_ok(), "VenvConfig should deserialize from TOML");
    
    let deserialized = deserialized.unwrap();
    assert_eq!(deserialized.name, "test-env");
    assert_eq!(deserialized.tools.get("node"), Some(&"18.17.0".to_string()));
    assert_eq!(deserialized.tools.get("python"), Some(&"3.11.5".to_string()));
}

/// Test invalid configuration handling
#[test]
fn test_invalid_config_handling() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let original_dir = std::env::current_dir().expect("Failed to get current dir");
    
    // Change to temp directory
    std::env::set_current_dir(temp_dir.path()).expect("Failed to change dir");
    
    // Set VX_HOME to temp directory
    std::env::set_var("VX_HOME", temp_dir.path());
    
    let manager = VenvManager::new().expect("Failed to create VenvManager");
    
    // Create invalid TOML
    let invalid_config = r#"
[tools
invalid toml syntax
"#;
    
    fs::write(".vx.toml", invalid_config).expect("Failed to write config");
    
    // Test that invalid config is handled gracefully
    let result = manager.load_project_config();
    assert!(result.is_err(), "Should fail with invalid TOML");
    
    // Restore original directory and clean up
    std::env::set_current_dir(original_dir).expect("Failed to restore dir");
    std::env::remove_var("VX_HOME");
}

/// Test environment variable handling
#[test]
fn test_venv_environment_variables() {
    // Test VenvManager static methods
    assert!(!VenvManager::is_active(), "Should not be active initially");
    assert!(VenvManager::current().is_none(), "Should have no current venv");
    
    // Set environment variable
    std::env::set_var("VX_VENV", "test-env");
    
    assert!(VenvManager::is_active(), "Should be active with VX_VENV set");
    assert_eq!(VenvManager::current(), Some("test-env".to_string()));
    
    // Clean up
    std::env::remove_var("VX_VENV");
    
    assert!(!VenvManager::is_active(), "Should not be active after cleanup");
    assert!(VenvManager::current().is_none(), "Should have no current venv after cleanup");
}

/// Test shell activation command generation
#[test]
fn test_shell_activation() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    
    // Set VX_HOME to temp directory
    std::env::set_var("VX_HOME", temp_dir.path());
    
    let manager = VenvManager::new().expect("Failed to create VenvManager");
    
    // Test activation command generation
    let commands = manager.generate_activation_commands("test-env");
    assert!(commands.is_ok(), "Should generate activation commands");
    
    let commands = commands.unwrap();
    assert!(commands.contains("export VX_VENV=test-env"), "Should set VX_VENV");
    assert!(commands.contains("export PS1="), "Should set prompt");
    
    // Clean up
    std::env::remove_var("VX_HOME");
}
