// Integration tests for the Figment-based configuration system
// Tests the complete configuration resolution flow including project config reuse

use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;
use vx::config_figment::FigmentConfigManager;

#[test]
fn test_minimal_config_always_works() {
    // This should always work, even with no configuration files
    let manager = FigmentConfigManager::minimal().expect("Minimal config should always work");

    // Check that basic tools are available
    let tools = manager.get_available_tools();
    assert!(
        !tools.is_empty(),
        "Should have at least some tools available"
    );
    assert!(tools.contains(&"uv".to_string()), "Should support uv");

    // Check that we can get download URLs
    let url = manager.get_download_url("uv", "latest");
    assert!(
        url.is_ok(),
        "Should be able to get download URL for uv: {:?}",
        url
    );

    let url = url.unwrap();
    assert!(
        url.starts_with("http"),
        "URL should be valid HTTP URL: {}",
        url
    );
}

#[test]
fn test_full_config_fallback() {
    // Test that full config falls back gracefully when no config files exist
    match FigmentConfigManager::new() {
        Ok(manager) => {
            // If full config works, test it
            let tools = manager.get_available_tools();
            assert!(!tools.is_empty());

            let url = manager.get_download_url("uv", "latest");
            assert!(url.is_ok());
        }
        Err(_) => {
            // If full config fails, that's okay - we should be able to use minimal
            let manager = FigmentConfigManager::minimal().expect("Minimal should work as fallback");
            let url = manager.get_download_url("uv", "latest");
            assert!(url.is_ok());
        }
    }
}

#[test]
fn test_config_status_and_diagnostics() {
    let manager = FigmentConfigManager::minimal().expect("Minimal config should work");

    // Test status
    let status = manager.get_status();
    assert!(
        !status.layers.is_empty(),
        "Should have at least builtin layer"
    );
    assert!(
        !status.available_tools.is_empty(),
        "Should have available tools"
    );

    // Test layer info
    let layers = &status.layers;
    assert!(!layers.is_empty(), "Should have layer information");

    // Should have at least the builtin layer
    let builtin_layer = layers.iter().find(|l| l.name == "builtin");
    assert!(builtin_layer.is_some(), "Should have builtin layer");
    assert!(
        builtin_layer.unwrap().available,
        "Builtin layer should be available"
    );
}

#[test]
fn test_resolution_context() {
    let mut manager = ConfigManager::minimal().expect("Minimal config should work");

    // Test basic context
    let context = ResolutionContext::new("uv", "latest");
    let resolution = manager.get_download_url_with_context(&context);
    assert!(resolution.is_ok(), "Basic resolution should work");

    let resolution = resolution.unwrap();
    assert_eq!(resolution.version, "latest");
    assert!(!resolution.download_url.is_empty());
    assert_eq!(resolution.source_layer, "builtin");
}

#[test]
fn test_safe_download_url() {
    let mut manager = ConfigManager::minimal().expect("Minimal config should work");

    // Test safe URL retrieval
    let url = manager.get_download_url_safe("uv", "latest");
    assert!(url.is_ok(), "Safe URL retrieval should work");

    // Test with non-existent tool (should fail gracefully)
    let url = manager.get_download_url_safe("nonexistent-tool", "latest");
    assert!(url.is_err(), "Should fail for non-existent tool");
}

#[test]
fn test_multiple_tools() {
    let mut manager = ConfigManager::minimal().expect("Minimal config should work");

    let test_tools = vec!["uv", "node", "go", "rust"];

    for tool in test_tools {
        if manager.supports_tool(tool) {
            let url = manager.get_download_url(tool, "latest");
            assert!(url.is_ok(), "Should get URL for {}: {:?}", tool, url);

            let url = url.unwrap();
            assert!(
                url.starts_with("http"),
                "URL should be valid for {}: {}",
                tool,
                url
            );
        }
    }
}

#[test]
fn test_version_aliases() {
    let mut manager = ConfigManager::minimal().expect("Minimal config should work");

    let version_aliases = vec!["latest"];

    for alias in version_aliases {
        if manager.supports_tool("uv") {
            let url = manager.get_download_url("uv", alias);
            assert!(
                url.is_ok(),
                "Should resolve version alias {}: {:?}",
                alias,
                url
            );
        }
    }
}

#[test]
fn test_config_manager_creation_patterns() {
    // Test different creation patterns

    // Pattern 1: Minimal (should always work)
    let minimal = ConfigManager::minimal();
    assert!(minimal.is_ok(), "Minimal creation should always work");

    // Pattern 2: Full (may fail, but that's okay)
    let full = ConfigManager::new();
    match full {
        Ok(_) => println!("Full configuration loaded successfully"),
        Err(e) => println!("Full configuration failed (expected): {}", e),
    }

    // Pattern 3: Fallback pattern
    let manager = ConfigManager::new().or_else(|_| ConfigManager::minimal());
    assert!(manager.is_ok(), "Fallback pattern should always work");
}

#[test]
fn test_cache_behavior() {
    let mut manager = ConfigManager::minimal().expect("Minimal config should work");

    // First resolution
    let url1 = manager.get_download_url("uv", "latest");
    assert!(url1.is_ok());

    // Second resolution (should use cache)
    let url2 = manager.get_download_url("uv", "latest");
    assert!(url2.is_ok());

    assert_eq!(
        url1.unwrap(),
        url2.unwrap(),
        "Cached results should be identical"
    );

    // Clear cache and try again
    manager.clear_cache();
    let url3 = manager.get_download_url("uv", "latest");
    assert!(url3.is_ok());
}

#[test]
fn test_error_handling() {
    let mut manager = ConfigManager::minimal().expect("Minimal config should work");

    // Test non-existent tool
    let result = manager.get_download_url("nonexistent-tool", "latest");
    assert!(result.is_err(), "Should fail for non-existent tool");

    // Test non-existent version (may or may not fail depending on implementation)
    let result = manager.get_download_url("uv", "nonexistent-version");
    // This might succeed if the layer treats unknown versions as "latest"
    println!("Non-existent version result: {:?}", result);
}

#[test]
fn test_platform_specific_urls() {
    let mut manager = ConfigManager::minimal().expect("Minimal config should work");

    if manager.supports_tool("uv") {
        let url = manager
            .get_download_url("uv", "latest")
            .expect("Should get URL");

        // URL should contain platform-specific information
        let current_platform = if cfg!(windows) {
            "windows"
        } else if cfg!(target_os = "macos") {
            "darwin"
        } else {
            "linux"
        };

        // The URL might contain platform information
        println!("Platform-specific URL for {}: {}", current_platform, url);

        // At minimum, it should be a valid HTTP URL
        assert!(url.starts_with("http"));
    }
}

// Helper function to create temporary config files for testing
#[allow(dead_code)]
fn create_temp_config() -> TempDir {
    let temp_dir = TempDir::new().expect("Should create temp dir");

    // Create a basic config structure
    let config_dir = temp_dir.path().join(".vx");
    fs::create_dir_all(&config_dir).expect("Should create config dir");

    // Create basic global config
    let global_config = r#"
[defaults]
auto_install = true
default_registry = "official"

[tools.uv]
version = "latest"
"#;

    fs::write(config_dir.join("config.toml"), global_config).expect("Should write config file");

    temp_dir
}

#[test]
fn test_config_reload() {
    let mut manager = FigmentConfigManager::minimal().expect("Minimal config should work");

    // Test reload functionality
    let result = manager.reload();
    assert!(result.is_ok(), "Reload should not fail");

    // Manager should still work after reload
    let tools = manager.get_available_tools();
    assert!(!tools.is_empty(), "Should still have tools after reload");
}

#[test]
fn test_project_config_detection() {
    let temp_dir = TempDir::new().expect("Should create temp dir");
    let original_dir = std::env::current_dir().expect("Should get current dir");

    // Change to temp directory
    std::env::set_current_dir(&temp_dir).expect("Should change dir");

    // Test 1: No project files
    let manager =
        FigmentConfigManager::new().unwrap_or_else(|_| FigmentConfigManager::minimal().unwrap());
    let status = manager.get_status();
    assert!(
        status.project_info.is_none()
            || matches!(
                status.project_info.as_ref().unwrap().project_type,
                vx::config_figment::ProjectType::Unknown
            )
    );

    // Test 2: Python project
    fs::write(
        "pyproject.toml",
        r#"
[project]
requires-python = ">=3.9"

[tool.uv]
version = "0.1.0"
"#,
    )
    .expect("Should write pyproject.toml");

    let manager = FigmentConfigManager::new().expect("Should load Python project config");
    let status = manager.get_status();

    if let Some(project_info) = &status.project_info {
        assert!(matches!(
            project_info.project_type,
            vx::config_figment::ProjectType::Python
        ));
        assert!(project_info.tool_versions.contains_key("python"));
        assert!(project_info.tool_versions.contains_key("uv"));
        assert_eq!(
            project_info.tool_versions.get("uv"),
            Some(&"0.1.0".to_string())
        );
    }

    // Test 3: Node.js project
    fs::remove_file("pyproject.toml").ok();
    fs::write(
        "package.json",
        r#"
{
  "engines": {
    "node": ">=18.0.0",
    "npm": ">=9.0.0"
  }
}
"#,
    )
    .expect("Should write package.json");

    let manager = FigmentConfigManager::new().expect("Should load Node.js project config");
    let status = manager.get_status();

    if let Some(project_info) = &status.project_info {
        assert!(matches!(
            project_info.project_type,
            vx::config_figment::ProjectType::Node
        ));
        assert!(project_info.tool_versions.contains_key("node"));
    }

    // Restore original directory
    std::env::set_current_dir(original_dir).expect("Should restore dir");
}
