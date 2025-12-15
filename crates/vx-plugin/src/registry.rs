//! Bundle registry for managing and discovering tool bundles
//!
//! This module provides functionality for registering, discovering, and managing
//! tool bundles (formerly plugins) in the vx ecosystem.

use crate::{PackageManager, Result, ToolBundle, VxTool};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Bundle registry that manages all loaded tool bundles
///
/// The registry is responsible for:
/// - Loading and registering bundles
/// - Discovering tools and package managers
/// - Managing bundle lifecycle
/// - Resolving bundle dependencies
#[derive(Default, Clone)]
pub struct BundleRegistry {
    /// Map of bundle name to bundle instance
    bundles: Arc<RwLock<HashMap<String, Box<dyn ToolBundle>>>>,
    /// Map of tool name to bundle name (for quick lookup)
    tool_index: Arc<RwLock<HashMap<String, String>>>,
    /// Map of package manager name to bundle name (for quick lookup)
    pm_index: Arc<RwLock<HashMap<String, String>>>,
}

impl BundleRegistry {
    /// Create a new empty bundle registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a bundle with the registry
    ///
    /// This method adds a bundle to the registry and updates the internal
    /// indexes for quick tool and package manager lookup.
    pub async fn register_bundle(&self, mut bundle: Box<dyn ToolBundle>) -> Result<()> {
        let bundle_name = bundle.name().to_string();

        // Initialize the bundle
        bundle.initialize().await?;

        // Update tool index
        {
            let mut tool_index = self.tool_index.write().unwrap();
            for tool in bundle.tools() {
                tool_index.insert(tool.name().to_string(), bundle_name.clone());
                // Also register aliases
                for alias in tool.aliases() {
                    tool_index.insert(alias.to_string(), bundle_name.clone());
                }
            }
        }

        // Update package manager index
        {
            let mut pm_index = self.pm_index.write().unwrap();
            for pm in bundle.package_managers() {
                pm_index.insert(pm.name().to_string(), bundle_name.clone());
            }
        }

        // Register the bundle
        {
            let mut bundles = self.bundles.write().unwrap();
            bundles.insert(bundle_name, bundle);
        }

        Ok(())
    }

    /// Unregister a bundle from the registry
    ///
    /// This method removes a bundle and cleans up all associated indexes.
    pub async fn unregister_bundle(&self, bundle_name: &str) -> Result<()> {
        // Get the bundle first to call shutdown
        let mut bundle = {
            let mut bundles = self.bundles.write().unwrap();
            bundles.remove(bundle_name)
        };

        if let Some(ref mut bundle) = bundle {
            // Shutdown the bundle
            bundle.shutdown().await?;

            // Clean up tool index
            {
                let mut tool_index = self.tool_index.write().unwrap();
                tool_index.retain(|_, b_name| b_name != bundle_name);
            }

            // Clean up package manager index
            {
                let mut pm_index = self.pm_index.write().unwrap();
                pm_index.retain(|_, b_name| b_name != bundle_name);
            }
        }

        Ok(())
    }

    /// Get a tool by name
    ///
    /// Returns the tool implementation if found in any registered bundle.
    pub fn get_tool(&self, tool_name: &str) -> Option<Box<dyn VxTool>> {
        let tool_index = self.tool_index.read().unwrap();
        let bundles = self.bundles.read().unwrap();

        if let Some(bundle_name) = tool_index.get(tool_name) {
            if let Some(bundle) = bundles.get(bundle_name) {
                return bundle
                    .tools()
                    .into_iter()
                    .find(|tool| tool.name() == tool_name || tool.aliases().contains(&tool_name));
            }
        }

        None
    }

    /// Get a package manager by name
    ///
    /// Returns the package manager implementation if found in any registered bundle.
    pub fn get_package_manager(&self, pm_name: &str) -> Option<Box<dyn PackageManager>> {
        let pm_index = self.pm_index.read().unwrap();
        let bundles = self.bundles.read().unwrap();

        if let Some(bundle_name) = pm_index.get(pm_name) {
            if let Some(bundle) = bundles.get(bundle_name) {
                return bundle
                    .package_managers()
                    .into_iter()
                    .find(|pm| pm.name() == pm_name);
            }
        }

        None
    }

    /// List all registered bundles
    ///
    /// Returns a vector of bundle names currently registered.
    pub fn list_bundles(&self) -> Vec<String> {
        let bundles = self.bundles.read().unwrap();
        bundles.keys().cloned().collect()
    }

    /// List all available tools
    ///
    /// Returns a vector of tool names from all registered bundles.
    pub fn list_tools(&self) -> Vec<String> {
        let tool_index = self.tool_index.read().unwrap();
        tool_index.keys().cloned().collect()
    }

    /// List all available package managers
    ///
    /// Returns a vector of package manager names from all registered bundles.
    pub fn list_package_managers(&self) -> Vec<String> {
        let pm_index = self.pm_index.read().unwrap();
        pm_index.keys().cloned().collect()
    }

    /// Check if a tool is available
    ///
    /// Returns true if any registered bundle provides the specified tool.
    pub fn has_tool(&self, tool_name: &str) -> bool {
        let tool_index = self.tool_index.read().unwrap();
        tool_index.contains_key(tool_name)
    }

    /// Check if a package manager is available
    ///
    /// Returns true if any registered bundle provides the specified package manager.
    pub fn has_package_manager(&self, pm_name: &str) -> bool {
        let pm_index = self.pm_index.read().unwrap();
        pm_index.contains_key(pm_name)
    }

    /// Get bundle information
    ///
    /// Returns metadata about a specific bundle if it's registered.
    pub fn get_bundle_info(&self, bundle_name: &str) -> Option<HashMap<String, String>> {
        let bundles = self.bundles.read().unwrap();
        bundles.get(bundle_name).map(|bundle| bundle.metadata())
    }

    /// Get all bundle information
    ///
    /// Returns metadata for all registered bundles.
    pub fn get_all_bundle_info(&self) -> HashMap<String, HashMap<String, String>> {
        let bundles = self.bundles.read().unwrap();
        bundles
            .iter()
            .map(|(name, bundle)| (name.clone(), bundle.metadata()))
            .collect()
    }

    /// Shutdown all bundles
    ///
    /// This method shuts down all registered bundles and clears the registry.
    /// It should be called when the application is shutting down.
    pub async fn shutdown_all(&self) -> Result<()> {
        let bundles = {
            let mut bundles_guard = self.bundles.write().unwrap();
            std::mem::take(&mut *bundles_guard)
        };

        // Shutdown all bundles
        for (_, mut bundle) in bundles {
            if let Err(e) = bundle.shutdown().await {
                eprintln!(
                    "Warning: Failed to shutdown bundle {}: {}",
                    bundle.name(),
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

    // Backward compatibility methods

    /// Register a plugin (deprecated, use register_bundle)
    #[deprecated(since = "0.5.0", note = "Use `register_bundle` instead")]
    pub async fn register_plugin(&self, plugin: Box<dyn ToolBundle>) -> Result<()> {
        self.register_bundle(plugin).await
    }

    /// List all plugins (deprecated, use list_bundles)
    #[deprecated(since = "0.5.0", note = "Use `list_bundles` instead")]
    pub fn list_plugins(&self) -> Vec<String> {
        self.list_bundles()
    }

    /// Get plugin info (deprecated, use get_bundle_info)
    #[deprecated(since = "0.5.0", note = "Use `get_bundle_info` instead")]
    pub fn get_plugin_info(&self, plugin_name: &str) -> Option<HashMap<String, String>> {
        self.get_bundle_info(plugin_name)
    }

    /// Get all plugin info (deprecated, use get_all_bundle_info)
    #[deprecated(since = "0.5.0", note = "Use `get_all_bundle_info` instead")]
    pub fn get_all_plugin_info(&self) -> HashMap<String, HashMap<String, String>> {
        self.get_all_bundle_info()
    }
}

/// Builder for creating and configuring a bundle registry
///
/// This builder provides a fluent interface for setting up a bundle registry
/// with various configuration options.
#[derive(Default)]
pub struct BundleRegistryBuilder {
    bundles: Vec<Box<dyn ToolBundle>>,
}

impl BundleRegistryBuilder {
    /// Create a new bundle registry builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a bundle to be registered
    ///
    /// The bundle will be registered when `build()` is called.
    pub fn with_bundle(mut self, bundle: Box<dyn ToolBundle>) -> Self {
        self.bundles.push(bundle);
        self
    }

    /// Add a plugin (deprecated, use with_bundle)
    #[deprecated(since = "0.5.0", note = "Use `with_bundle` instead")]
    pub fn with_plugin(self, plugin: Box<dyn ToolBundle>) -> Self {
        self.with_bundle(plugin)
    }

    /// Build the bundle registry
    ///
    /// This method creates the registry and registers all added bundles.
    pub async fn build(self) -> Result<BundleRegistry> {
        let registry = BundleRegistry::new();

        for bundle in self.bundles {
            registry.register_bundle(bundle).await?;
        }

        Ok(registry)
    }
}

/// Simplified tool registry for backward compatibility
///
/// This registry provides a simplified interface focused on tools only,
/// maintaining compatibility with existing code that expects a ToolRegistry.
pub struct ToolRegistry {
    bundle_registry: BundleRegistry,
}

impl ToolRegistry {
    /// Create a new tool registry
    pub fn new() -> Self {
        Self {
            bundle_registry: BundleRegistry::new(),
        }
    }

    /// Register a bundle that provides tools
    pub async fn register_bundle(&self, bundle: Box<dyn ToolBundle>) -> Result<()> {
        self.bundle_registry.register_bundle(bundle).await
    }

    /// Register a plugin (deprecated, use register_bundle)
    #[deprecated(since = "0.5.0", note = "Use `register_bundle` instead")]
    pub async fn register_plugin(&self, plugin: Box<dyn ToolBundle>) -> Result<()> {
        self.register_bundle(plugin).await
    }

    /// Get a tool by name
    pub fn get_tool(&self, tool_name: &str) -> Option<Box<dyn VxTool>> {
        self.bundle_registry.get_tool(tool_name)
    }

    /// Check if a tool is supported
    pub fn supports_tool(&self, tool_name: &str) -> bool {
        self.bundle_registry.has_tool(tool_name)
    }

    /// Get all tool names
    pub fn get_tool_names(&self) -> Vec<String> {
        self.bundle_registry.list_tools()
    }

    /// Get all tools from all bundles
    pub fn get_all_tools(&self) -> Vec<Box<dyn VxTool>> {
        let bundles = self.bundle_registry.bundles.read().unwrap();
        bundles.values().flat_map(|bundle| bundle.tools()).collect()
    }

    /// Initialize all bundles
    pub async fn initialize_all(&self) -> Result<()> {
        // Get all bundles and initialize them
        let bundles = {
            let bundles_guard = self.bundle_registry.bundles.read().unwrap();
            bundles_guard.keys().cloned().collect::<Vec<_>>()
        };

        for _bundle_name in bundles {
            // Note: This is a simplified implementation
            // In a real scenario, we'd need mutable access to bundles
            // For now, we assume bundles are already initialized during registration
        }

        Ok(())
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}
