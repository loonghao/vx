//! Plugin system integration tests
//!
//! These tests verify that the plugin system works correctly
//! and that all plugins are properly registered and functional.

use vx_core::{PluginRegistry, VxPlugin};

/// Test that all plugins can be registered without conflicts
#[tokio::test]
async fn test_plugin_registration() {
    let mut registry = PluginRegistry::new();

    // Register all available plugins
    let node_plugin = Box::new(vx_tool_node::NodePlugin::new());
    let go_plugin = Box::new(vx_tool_go::GoPlugin::new());
    let rust_plugin = Box::new(vx_tool_rust::RustPlugin::new());
    let uv_plugin = Box::new(vx_tool_uv::UvPlugin::new());

    assert!(registry.register(node_plugin).is_ok());
    assert!(registry.register(go_plugin).is_ok());
    assert!(registry.register(rust_plugin).is_ok());
    assert!(registry.register(uv_plugin).is_ok());

    // Verify tools are available
    assert!(registry.supports_tool("node"));
    assert!(registry.supports_tool("npm"));
    assert!(registry.supports_tool("npx"));
    assert!(registry.supports_tool("go"));
    assert!(registry.supports_tool("cargo"));
    assert!(registry.supports_tool("rustc"));
    assert!(registry.supports_tool("uv"));
    assert!(registry.supports_tool("uvx"));

    // Verify package managers are available
    assert!(registry.supports_package_manager("npm"));
}

/// Test that each plugin provides the expected tools
#[test]
fn test_node_plugin_tools() {
    let plugin = vx_tool_node::NodePlugin::new();
    let tools = plugin.tools();

    assert!(!tools.is_empty());

    let tool_names: Vec<&str> = tools.iter().map(|t| t.name()).collect();
    assert!(tool_names.contains(&"node"));
    assert!(tool_names.contains(&"npm"));
    assert!(tool_names.contains(&"npx"));
}

#[test]
fn test_go_plugin_tools() {
    let plugin = vx_tool_go::GoPlugin::new();
    let tools = plugin.tools();

    assert!(!tools.is_empty());

    let tool_names: Vec<&str> = tools.iter().map(|t| t.name()).collect();
    assert!(tool_names.contains(&"go"));
}

#[test]
fn test_rust_plugin_tools() {
    let plugin = vx_tool_rust::RustPlugin::new();
    let tools = plugin.tools();

    assert!(!tools.is_empty());

    let tool_names: Vec<&str> = tools.iter().map(|t| t.name()).collect();
    assert!(tool_names.contains(&"cargo"));
    assert!(tool_names.contains(&"rustc"));
    assert!(tool_names.contains(&"rustup"));
    assert!(tool_names.contains(&"rustdoc"));
    assert!(tool_names.contains(&"rustfmt"));
    assert!(tool_names.contains(&"clippy"));
}

#[test]
fn test_uv_plugin_tools() {
    let plugin = vx_tool_uv::UvPlugin::new();
    let tools = plugin.tools();

    assert!(!tools.is_empty());

    let tool_names: Vec<&str> = tools.iter().map(|t| t.name()).collect();
    assert!(tool_names.contains(&"uv"));
    assert!(tool_names.contains(&"uvx"));
}

/// Test that package managers are properly provided
#[test]
fn test_npm_package_manager() {
    let plugin = vx_tool_node::NodePlugin::new();
    let package_managers = plugin.package_managers();

    assert!(!package_managers.is_empty());

    let pm_names: Vec<&str> = package_managers.iter().map(|pm| pm.name()).collect();
    assert!(pm_names.contains(&"npm"));
}

/// Test plugin metadata
#[test]
fn test_plugin_metadata() {
    let node_plugin = vx_tool_node::NodePlugin::new();
    assert_eq!(node_plugin.name(), "node");
    assert!(!node_plugin.description().is_empty());

    let go_plugin = vx_tool_go::GoPlugin::new();
    assert_eq!(go_plugin.name(), "go");
    assert!(!go_plugin.description().is_empty());

    let rust_plugin = vx_tool_rust::RustPlugin::new();
    assert_eq!(rust_plugin.name(), "rust");
    assert!(!rust_plugin.description().is_empty());

    let uv_plugin = vx_tool_uv::UvPlugin::new();
    assert_eq!(uv_plugin.name(), "uv");
    assert!(!uv_plugin.description().is_empty());
}

/// Test that tools can fetch version information
#[tokio::test]
async fn test_tool_version_fetching() {
    let node_plugin = vx_tool_node::NodePlugin::new();
    let tools = node_plugin.tools();

    for tool in tools {
        if tool.name() == "node" {
            // Test version fetching (this might fail if network is unavailable)
            let result = tool.fetch_versions(false).await;
            // We don't assert success here because it depends on network connectivity
            // But we ensure the method doesn't panic
            match result {
                Ok(versions) => {
                    // If successful, should have at least one version
                    if !versions.is_empty() {
                        assert!(!versions[0].version.is_empty());
                    }
                }
                Err(_) => {
                    // Network errors are acceptable in tests
                }
            }
            break;
        }
    }
}

/// Test plugin ecosystem identification
#[test]
fn test_plugin_ecosystems() {
    use vx_core::Ecosystem;

    let node_plugin = vx_tool_node::NodePlugin::new();
    let package_managers = node_plugin.package_managers();

    for pm in package_managers {
        if pm.name() == "npm" {
            assert_eq!(pm.ecosystem(), Ecosystem::JavaScript);
            break;
        }
    }
}

/// Test that plugins don't have naming conflicts
#[test]
fn test_no_plugin_naming_conflicts() {
    let mut all_tool_names = Vec::new();
    let mut all_pm_names = Vec::new();

    // Collect all tool names from all plugins
    let plugins: Vec<Box<dyn VxPlugin>> = vec![
        Box::new(vx_tool_node::NodePlugin::new()),
        Box::new(vx_tool_go::GoPlugin::new()),
        Box::new(vx_tool_rust::RustPlugin::new()),
        Box::new(vx_tool_uv::UvPlugin::new()),
    ];

    for plugin in plugins {
        for tool in plugin.tools() {
            all_tool_names.push(tool.name().to_string());
        }
        for pm in plugin.package_managers() {
            all_pm_names.push(pm.name().to_string());
        }
    }

    // Check for duplicates in tool names
    let mut unique_tool_names = all_tool_names.clone();
    unique_tool_names.sort();
    unique_tool_names.dedup();
    assert_eq!(
        all_tool_names.len(),
        unique_tool_names.len(),
        "Found duplicate tool names: {:?}",
        all_tool_names
    );

    // Check for duplicates in package manager names
    let mut unique_pm_names = all_pm_names.clone();
    unique_pm_names.sort();
    unique_pm_names.dedup();
    assert_eq!(
        all_pm_names.len(),
        unique_pm_names.len(),
        "Found duplicate package manager names: {:?}",
        all_pm_names
    );
}
