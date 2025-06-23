use std::collections::HashMap;
use vx_plugin::{Ecosystem, VxPackageManager, VxPlugin, VxTool};
use vx_tool_bun::{BunPackageManager, BunPlugin, BunTool};

#[test]
fn test_bun_tool_basic_properties() {
    let tool = BunTool::new();

    assert_eq!(tool.name(), "bun");
    assert_eq!(
        tool.description(),
        "Incredibly fast JavaScript runtime, bundler, test runner, and package manager"
    );
    assert!(tool.aliases().is_empty());
}

#[test]
fn test_bun_tool_metadata() {
    let tool = BunTool::new();
    let metadata = tool.metadata();

    assert_eq!(
        metadata.get("homepage"),
        Some(&"https://bun.sh/".to_string())
    );
    assert_eq!(metadata.get("ecosystem"), Some(&"javascript".to_string()));
    assert_eq!(
        metadata.get("repository"),
        Some(&"https://github.com/oven-sh/bun".to_string())
    );
}

#[test]
fn test_bun_tool_dependencies() {
    let tool = BunTool::new();
    let dependencies = tool.get_dependencies();

    // Bun is a standalone runtime that doesn't require Node.js
    assert!(dependencies.is_empty());
}

#[test]
fn test_bun_tool_default() {
    let tool = BunTool::default();
    assert_eq!(tool.name(), "bun");
}

#[tokio::test]
async fn test_bun_tool_download_url() {
    let tool = BunTool::new();
    let url = tool.get_download_url("1.0.0").await.unwrap();

    assert!(url.is_some());
    let url = url.unwrap();
    assert!(url.contains("github.com/oven-sh/bun"));
    assert!(url.contains("1.0.0"));
}

#[tokio::test]
async fn test_bun_tool_active_version() {
    let tool = BunTool::new();
    let result = tool.get_active_version().await;

    // Should return an error when no versions are installed
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("No Bun versions installed"));
}

#[tokio::test]
async fn test_bun_tool_installed_versions() {
    let tool = BunTool::new();
    let versions = tool.get_installed_versions().await.unwrap();

    // Default implementation returns empty list
    assert!(versions.is_empty());
}

#[test]
fn test_bun_package_manager_basic_properties() {
    let pm = BunPackageManager;

    assert_eq!(pm.name(), "bun");
    assert_eq!(pm.ecosystem(), Ecosystem::Node);
    assert_eq!(
        pm.description(),
        "Incredibly fast JavaScript runtime, bundler, test runner, and package manager"
    );
}

#[test]
fn test_bun_package_manager_config_files() {
    let pm = BunPackageManager;
    let config_files = pm.get_config_files();

    assert_eq!(
        config_files,
        vec!["package.json", "bun.lockb", "bunfig.toml"]
    );
}

#[test]
fn test_bun_package_manager_project_detection() {
    use std::fs;
    use tempfile::TempDir;

    let temp_dir = TempDir::new().unwrap();
    let pm = BunPackageManager;

    // Initially not a bun project
    assert!(!pm.is_preferred_for_project(temp_dir.path()));

    // Create bun.lockb file
    let lockb_path = temp_dir.path().join("bun.lockb");
    fs::write(&lockb_path, "").unwrap();

    // Now it should be detected as a bun project
    assert!(pm.is_preferred_for_project(temp_dir.path()));
}

#[test]
fn test_bun_plugin_basic_properties() {
    let plugin = BunPlugin::new();

    assert_eq!(plugin.name(), "bun");
    assert_eq!(plugin.description(), "Bun package manager support for vx");
    assert_eq!(plugin.version(), "1.0.0");
}

#[test]
fn test_bun_plugin_tool_support() {
    let plugin = BunPlugin::new();

    assert!(plugin.supports_tool("bun"));
    assert!(!plugin.supports_tool("node"));
    assert!(!plugin.supports_tool("npm"));
    assert!(!plugin.supports_tool("yarn"));
}

#[test]
fn test_bun_plugin_tools() {
    let plugin = BunPlugin::new();
    let tools = plugin.tools();

    assert_eq!(tools.len(), 1);
    assert_eq!(tools[0].name(), "bun");
}

#[test]
fn test_bun_plugin_package_managers() {
    let plugin = BunPlugin::new();
    let package_managers = plugin.package_managers();

    assert_eq!(package_managers.len(), 1);
    assert_eq!(package_managers[0].name(), "bun");
    assert_eq!(package_managers[0].ecosystem(), Ecosystem::Node);
}

#[test]
fn test_bun_plugin_default() {
    let plugin = BunPlugin::default();
    assert_eq!(plugin.name(), "bun");
}

#[test]
fn test_create_bun_plugin() {
    let plugin = vx_tool_bun::create_bun_plugin();
    assert_eq!(plugin.name(), "bun");
    assert_eq!(plugin.version(), "1.0.0");
}

#[test]
fn test_bun_tool_independence() {
    let tool = BunTool::new();
    let dependencies = tool.get_dependencies();

    // Verify that Bun doesn't depend on Node.js
    assert!(dependencies.is_empty());

    // Verify metadata indicates it's a JavaScript ecosystem tool
    let metadata = tool.metadata();
    assert_eq!(metadata.get("ecosystem"), Some(&"javascript".to_string()));
}

#[test]
fn test_bun_vs_node_ecosystem() {
    let bun_tool = BunTool::new();
    let bun_pm = BunPackageManager;

    // Both should be in the Node ecosystem (JavaScript)
    assert_eq!(bun_pm.ecosystem(), Ecosystem::Node);
    assert_eq!(
        bun_tool.metadata().get("ecosystem"),
        Some(&"javascript".to_string())
    );

    // But Bun should not depend on Node.js
    assert!(bun_tool.get_dependencies().is_empty());
}

#[test]
fn test_bun_tool_serialization_compatibility() {
    let tool = BunTool::new();
    let metadata = tool.metadata();

    // Test that metadata can be serialized (important for configuration)
    let serialized = serde_json::to_string(&metadata).unwrap();
    let deserialized: HashMap<String, String> = serde_json::from_str(&serialized).unwrap();

    assert_eq!(metadata, deserialized);
}

#[tokio::test]
async fn test_bun_tool_version_fetching() {
    let tool = BunTool::new();

    // Test that version fetching doesn't panic
    // Note: This might fail in CI without internet, so we just test it doesn't panic
    let result = tool.fetch_versions(false).await;

    // We don't assert success because it depends on network connectivity
    // But we can assert that it returns a Result
    match result {
        Ok(_versions) => {
            // If successful, versions should be a valid Vec
            // No need to check len() >= 0 as it's always true for Vec
        }
        Err(_) => {
            // If it fails, that's also acceptable (network issues, etc.)
        }
    }
}
