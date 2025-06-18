//! Plugin registry for managing and discovering plugins
//!
//! This module provides functionality for registering, discovering, and managing
//! plugins in the vx ecosystem.

use crate::{Result, VxPackageManager, VxPlugin, VxTool};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Plugin registry that manages all loaded plugins
///
/// The registry is responsible for:
/// - Loading and registering plugins
/// - Discovering tools and package managers
/// - Managing plugin lifecycle
/// - Resolving plugin dependencies
#[derive(Default)]
pub struct PluginRegistry {
    /// Map of plugin name to plugin instance
    plugins: Arc<RwLock<HashMap<String, Box<dyn VxPlugin>>>>,
    /// Map of tool name to plugin name (for quick lookup)
    tool_index: Arc<RwLock<HashMap<String, String>>>,
    /// Map of package manager name to plugin name (for quick lookup)
    pm_index: Arc<RwLock<HashMap<String, String>>>,
}

impl PluginRegistry {
    /// Create a new empty plugin registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a plugin with the registry
    ///
    /// This method adds a plugin to the registry and updates the internal
    /// indexes for quick tool and package manager lookup.
    pub async fn register_plugin(&self, mut plugin: Box<dyn VxPlugin>) -> Result<()> {
        let plugin_name = plugin.name().to_string();

        // Initialize the plugin
        plugin.initialize().await?;

        // Update tool index
        {
            let mut tool_index = self.tool_index.write().unwrap();
            for tool in plugin.tools() {
                tool_index.insert(tool.name().to_string(), plugin_name.clone());
                // Also register aliases
                for alias in tool.aliases() {
                    tool_index.insert(alias.to_string(), plugin_name.clone());
                }
            }
        }

        // Update package manager index
        {
            let mut pm_index = self.pm_index.write().unwrap();
            for pm in plugin.package_managers() {
                pm_index.insert(pm.name().to_string(), plugin_name.clone());
            }
        }

        // Register the plugin
        {
            let mut plugins = self.plugins.write().unwrap();
            plugins.insert(plugin_name, plugin);
        }

        Ok(())
    }
    /// Unregister a plugin from the registry
    ///
    /// This method removes a plugin and cleans up all associated indexes.
    pub async fn unregister_plugin(&self, plugin_name: &str) -> Result<()> {
        // Get the plugin first to call shutdown
        let mut plugin = {
            let mut plugins = self.plugins.write().unwrap();
            plugins.remove(plugin_name)
        };

        if let Some(ref mut plugin) = plugin {
            // Shutdown the plugin
            plugin.shutdown().await?;

            // Clean up tool index
            {
                let mut tool_index = self.tool_index.write().unwrap();
                tool_index.retain(|_, p_name| p_name != plugin_name);
            }

            // Clean up package manager index
            {
                let mut pm_index = self.pm_index.write().unwrap();
                pm_index.retain(|_, p_name| p_name != plugin_name);
            }
        }

        Ok(())
    }

    /// Get a tool by name
    ///
    /// Returns the tool implementation if found in any registered plugin.
    pub fn get_tool(&self, tool_name: &str) -> Option<Box<dyn VxTool>> {
        let tool_index = self.tool_index.read().unwrap();
        let plugins = self.plugins.read().unwrap();

        if let Some(plugin_name) = tool_index.get(tool_name) {
            if let Some(plugin) = plugins.get(plugin_name) {
                return plugin
                    .tools()
                    .into_iter()
                    .find(|tool| tool.name() == tool_name || tool.aliases().contains(&tool_name));
            }
        }

        None
    }

    /// Get a package manager by name
    ///
    /// Returns the package manager implementation if found in any registered plugin.
    pub fn get_package_manager(&self, pm_name: &str) -> Option<Box<dyn VxPackageManager>> {
        let pm_index = self.pm_index.read().unwrap();
        let plugins = self.plugins.read().unwrap();

        if let Some(plugin_name) = pm_index.get(pm_name) {
            if let Some(plugin) = plugins.get(plugin_name) {
                return plugin
                    .package_managers()
                    .into_iter()
                    .find(|pm| pm.name() == pm_name);
            }
        }

        None
    }
    /// List all registered plugins
    ///
    /// Returns a vector of plugin names currently registered.
    pub fn list_plugins(&self) -> Vec<String> {
        let plugins = self.plugins.read().unwrap();
        plugins.keys().cloned().collect()
    }

    /// List all available tools
    ///
    /// Returns a vector of tool names from all registered plugins.
    pub fn list_tools(&self) -> Vec<String> {
        let tool_index = self.tool_index.read().unwrap();
        tool_index.keys().cloned().collect()
    }

    /// List all available package managers
    ///
    /// Returns a vector of package manager names from all registered plugins.
    pub fn list_package_managers(&self) -> Vec<String> {
        let pm_index = self.pm_index.read().unwrap();
        pm_index.keys().cloned().collect()
    }

    /// Check if a tool is available
    ///
    /// Returns true if any registered plugin provides the specified tool.
    pub fn has_tool(&self, tool_name: &str) -> bool {
        let tool_index = self.tool_index.read().unwrap();
        tool_index.contains_key(tool_name)
    }

    /// Check if a package manager is available
    ///
    /// Returns true if any registered plugin provides the specified package manager.
    pub fn has_package_manager(&self, pm_name: &str) -> bool {
        let pm_index = self.pm_index.read().unwrap();
        pm_index.contains_key(pm_name)
    }

    /// Get plugin information
    ///
    /// Returns metadata about a specific plugin if it's registered.
    pub fn get_plugin_info(&self, plugin_name: &str) -> Option<HashMap<String, String>> {
        let plugins = self.plugins.read().unwrap();
        plugins.get(plugin_name).map(|plugin| plugin.metadata())
    }

    /// Get all plugin information
    ///
    /// Returns metadata for all registered plugins.
    pub fn get_all_plugin_info(&self) -> HashMap<String, HashMap<String, String>> {
        let plugins = self.plugins.read().unwrap();
        plugins
            .iter()
            .map(|(name, plugin)| (name.clone(), plugin.metadata()))
            .collect()
    }
    /// Shutdown all plugins
    ///
    /// This method shuts down all registered plugins and clears the registry.
    /// It should be called when the application is shutting down.
    pub async fn shutdown_all(&self) -> Result<()> {
        let plugins = {
            let mut plugins_guard = self.plugins.write().unwrap();
            std::mem::take(&mut *plugins_guard)
        };

        // Shutdown all plugins
        for (_, mut plugin) in plugins {
            if let Err(e) = plugin.shutdown().await {
                eprintln!(
                    "Warning: Failed to shutdown plugin {}: {}",
                    plugin.name(),
                    e
                );
            }
        }

        // Clear indexes
        {
            let mut tool_index = self.tool_index.write().unwrap();
            tool_index.clear();
        }
        {
            let mut pm_index = self.pm_index.write().unwrap();
            pm_index.clear();
        }

        Ok(())
    }
}

/// Builder for creating and configuring a plugin registry
///
/// This builder provides a fluent interface for setting up a plugin registry
/// with various configuration options.
#[derive(Default)]
pub struct PluginRegistryBuilder {
    plugins: Vec<Box<dyn VxPlugin>>,
}

impl PluginRegistryBuilder {
    /// Create a new plugin registry builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a plugin to be registered
    ///
    /// The plugin will be registered when `build()` is called.
    pub fn with_plugin(mut self, plugin: Box<dyn VxPlugin>) -> Self {
        self.plugins.push(plugin);
        self
    }

    /// Build the plugin registry
    ///
    /// This method creates the registry and registers all added plugins.
    pub async fn build(self) -> Result<PluginRegistry> {
        let registry = PluginRegistry::new();

        for plugin in self.plugins {
            registry.register_plugin(plugin).await?;
        }

        Ok(registry)
    }
}

/// Simplified tool registry for backward compatibility
///
/// This registry provides a simplified interface focused on tools only,
/// maintaining compatibility with existing code that expects a ToolRegistry.
pub struct ToolRegistry {
    plugin_registry: PluginRegistry,
}

impl ToolRegistry {
    /// Create a new tool registry
    pub fn new() -> Self {
        Self {
            plugin_registry: PluginRegistry::new(),
        }
    }

    /// Register a plugin that provides tools
    pub async fn register_plugin(&self, plugin: Box<dyn VxPlugin>) -> Result<()> {
        self.plugin_registry.register_plugin(plugin).await
    }

    /// Get a tool by name
    pub fn get_tool(&self, tool_name: &str) -> Option<Box<dyn VxTool>> {
        self.plugin_registry.get_tool(tool_name)
    }

    /// Check if a tool is supported
    pub fn supports_tool(&self, tool_name: &str) -> bool {
        self.plugin_registry.has_tool(tool_name)
    }

    /// Get all tool names
    pub fn get_tool_names(&self) -> Vec<String> {
        self.plugin_registry.list_tools()
    }

    /// Get all tools from all plugins
    pub fn get_all_tools(&self) -> Vec<Box<dyn VxTool>> {
        let plugins = self.plugin_registry.plugins.read().unwrap();
        plugins.values().flat_map(|plugin| plugin.tools()).collect()
    }

    /// Initialize all plugins
    pub async fn initialize_all(&self) -> Result<()> {
        // Get all plugins and initialize them
        let plugins = {
            let plugins_guard = self.plugin_registry.plugins.read().unwrap();
            plugins_guard.keys().cloned().collect::<Vec<_>>()
        };

        for _plugin_name in plugins {
            // Note: This is a simplified implementation
            // In a real scenario, we'd need mutable access to plugins
            // For now, we assume plugins are already initialized during registration
        }

        Ok(())
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}
