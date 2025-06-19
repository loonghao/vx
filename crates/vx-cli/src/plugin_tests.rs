//! Tests for plugin system functionality

use crate::test_utils::*;
use vx_plugin::{PluginRegistry, VxPlugin, VxTool};

#[tokio::test]
async fn test_plugin_registry_creation() {
    let registry = PluginRegistry::new();

    // Registry should be created successfully
    let plugins = registry.list_plugins();
    assert!(plugins.is_empty()); // Should start empty
}

#[tokio::test]
async fn test_plugin_registration() {
    let registry = PluginRegistry::new();

    // Create a mock plugin
    let plugin = MockPlugin::new("test-plugin").with_tool(MockTool::new("test-tool", "1.0.0"));

    // Register the plugin
    let result = registry.register_plugin(Box::new(plugin)).await;
    assert!(result.is_ok());

    // Check that plugin is registered
    let plugins = registry.list_plugins();
    assert_eq!(plugins.len(), 1);
    assert!(plugins.contains(&"test-plugin".to_string()));
}

#[tokio::test]
async fn test_multiple_plugin_registration() {
    let registry = PluginRegistry::new();

    // Register multiple plugins
    let plugin1 = MockPlugin::new("plugin1").with_tool(MockTool::new("tool1", "1.0.0"));
    let plugin2 = MockPlugin::new("plugin2").with_tool(MockTool::new("tool2", "2.0.0"));

    let _ = registry.register_plugin(Box::new(plugin1)).await;
    let _ = registry.register_plugin(Box::new(plugin2)).await;

    let plugins = registry.list_plugins();
    assert_eq!(plugins.len(), 2);
    assert!(plugins.contains(&"plugin1".to_string()));
    assert!(plugins.contains(&"plugin2".to_string()));
}

#[tokio::test]
async fn test_tool_discovery() {
    let registry = PluginRegistry::new();

    // Create plugin with multiple tools
    let plugin = MockPlugin::new("multi-tool-plugin")
        .with_tool(MockTool::new("node", "18.0.0"))
        .with_tool(MockTool::new("npm", "8.0.0"))
        .with_tool(MockTool::new("yarn", "1.22.0"));

    let _ = registry.register_plugin(Box::new(plugin)).await;

    // Test tool discovery
    let tools = registry.list_tools();
    assert_eq!(tools.len(), 3);
    assert!(tools.contains(&"node".to_string()));
    assert!(tools.contains(&"npm".to_string()));
    assert!(tools.contains(&"yarn".to_string()));
}

#[tokio::test]
async fn test_get_tool() {
    let mut registry = PluginRegistry::new();

    let plugin = MockPlugin::new("test-plugin").with_tool(MockTool::new("test-tool", "1.0.0"));

    let _ = registry.register_plugin(Box::new(plugin)).await;

    // Test getting existing tool
    let tool = registry.get_tool("test-tool");
    assert!(tool.is_some());

    let tool = tool.unwrap();
    assert_eq!(tool.name(), "test-tool");

    // Test getting non-existent tool
    let missing_tool = registry.get_tool("missing-tool");
    assert!(missing_tool.is_none());
}

#[tokio::test]
async fn test_tool_installation_status() {
    let registry = PluginRegistry::new();

    // Create tools with different installation states
    let installed_tool = MockTool::new("installed-tool", "1.0.0")
        .with_executable(std::path::PathBuf::from("/usr/bin/installed-tool"));
    let not_installed_tool = MockTool::new("not-installed-tool", "1.0.0");

    let plugin = MockPlugin::new("test-plugin")
        .with_tool(installed_tool)
        .with_tool(not_installed_tool);

    let _ = registry.register_plugin(Box::new(plugin)).await;

    // Test that tools can be retrieved
    let tool = registry.get_tool("installed-tool");
    assert!(tool.is_some());

    let tool = registry.get_tool("not-installed-tool");
    assert!(tool.is_some());
}

#[test]
fn test_mock_plugin_creation() {
    let plugin = MockPlugin::new("test-plugin");
    assert_eq!(plugin.name(), "test-plugin");
    assert_eq!(plugin.description(), "Mock plugin for testing");
    assert!(plugin.tools().is_empty());
}

#[test]
fn test_mock_plugin_with_tools() {
    let tool1 = MockTool::new("tool1", "1.0.0");
    let tool2 = MockTool::new("tool2", "2.0.0");

    let plugin = MockPlugin::new("test-plugin")
        .with_tool(tool1)
        .with_tool(tool2);

    let tools = plugin.tools();
    assert_eq!(tools.len(), 2);
    assert_eq!(tools[0].name(), "tool1");
    assert_eq!(tools[1].name(), "tool2");
}

#[test]
fn test_mock_tool_properties() {
    let tool = MockTool::new("test-tool", "1.0.0");

    assert_eq!(tool.name(), "test-tool");
    assert_eq!(tool.description(), "Mock tool for testing");
    assert_eq!(tool.version, "1.0.0");
    assert!(tool.executable_path.is_none());
}

#[test]
fn test_mock_tool_with_executable() {
    let path = std::path::PathBuf::from("/usr/bin/test-tool");
    let tool = MockTool::new("test-tool", "1.0.0").with_executable(path.clone());

    assert_eq!(tool.name(), "test-tool");
    assert_eq!(tool.executable_path, Some(path));
}

#[test]
fn test_mock_tool_failure_mode() {
    let tool = MockTool::new("failing-tool", "1.0.0").with_failure();

    assert!(tool.should_fail);
}

#[tokio::test]
async fn test_plugin_registry_tool_lookup_performance() {
    let mut registry = PluginRegistry::new();

    // Register many tools to test lookup performance
    for i in 0..100 {
        let plugin = MockPlugin::new(&format!("plugin-{}", i))
            .with_tool(MockTool::new(&format!("tool-{}", i), "1.0.0"));
        let _ = registry.register_plugin(Box::new(plugin)).await;
    }

    // Test that lookup is still fast
    let start = std::time::Instant::now();
    let tool = registry.get_tool("tool-50");
    let duration = start.elapsed();

    assert!(tool.is_some());
    assert!(duration.as_millis() < 10); // Should be very fast
}

#[tokio::test]
async fn test_plugin_registry_duplicate_tools() {
    let mut registry = PluginRegistry::new();

    // Register two plugins with the same tool name
    let plugin1 = MockPlugin::new("plugin1").with_tool(MockTool::new("common-tool", "1.0.0"));
    let plugin2 = MockPlugin::new("plugin2").with_tool(MockTool::new("common-tool", "2.0.0"));

    let _ = registry.register_plugin(Box::new(plugin1)).await;
    let _ = registry.register_plugin(Box::new(plugin2)).await;

    // Should handle duplicate tool names gracefully
    let tool = registry.get_tool("common-tool");
    assert!(tool.is_some());

    // The behavior for duplicate tools depends on implementation
    // This test ensures it doesn't panic
}

#[tokio::test]
async fn test_empty_plugin_registration() {
    let mut registry = PluginRegistry::new();

    // Register a plugin with no tools
    let empty_plugin = MockPlugin::new("empty-plugin");
    let result = registry.register_plugin(Box::new(empty_plugin)).await;

    assert!(result.is_ok());

    let plugins = registry.list_plugins();
    assert_eq!(plugins.len(), 1);
    assert!(plugins.contains(&"empty-plugin".to_string()));

    let tools = registry.list_tools();
    assert!(tools.is_empty());
}
