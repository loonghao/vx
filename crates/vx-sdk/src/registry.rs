//! Bundle registry for managing and discovering tool bundles

use crate::{PackageManager, Result, Tool, ToolBundle};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Bundle registry that manages all loaded tool bundles
#[derive(Default)]
pub struct BundleRegistry {
    bundles: Arc<RwLock<HashMap<String, Box<dyn ToolBundle>>>>,
    tool_index: Arc<RwLock<HashMap<String, String>>>,
    pm_index: Arc<RwLock<HashMap<String, String>>>,
}

impl BundleRegistry {
    /// Create a new empty bundle registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a bundle with the registry
    pub async fn register_bundle(&self, mut bundle: Box<dyn ToolBundle>) -> Result<()> {
        let bundle_name = bundle.name().to_string();

        bundle.initialize().await?;

        {
            let mut tool_index = self.tool_index.write().unwrap();
            for tool in bundle.tools() {
                tool_index.insert(tool.name().to_string(), bundle_name.clone());
                for alias in tool.aliases() {
                    tool_index.insert(alias.to_string(), bundle_name.clone());
                }
            }
        }

        {
            let mut pm_index = self.pm_index.write().unwrap();
            for pm in bundle.package_managers() {
                pm_index.insert(pm.name().to_string(), bundle_name.clone());
            }
        }

        {
            let mut bundles = self.bundles.write().unwrap();
            bundles.insert(bundle_name, bundle);
        }

        Ok(())
    }

    /// Unregister a bundle from the registry
    pub async fn unregister_bundle(&self, bundle_name: &str) -> Result<()> {
        let mut bundle = {
            let mut bundles = self.bundles.write().unwrap();
            bundles.remove(bundle_name)
        };

        if let Some(ref mut bundle) = bundle {
            bundle.shutdown().await?;

            {
                let mut tool_index = self.tool_index.write().unwrap();
                tool_index.retain(|_, b_name| b_name != bundle_name);
            }

            {
                let mut pm_index = self.pm_index.write().unwrap();
                pm_index.retain(|_, b_name| b_name != bundle_name);
            }
        }

        Ok(())
    }

    /// Get a tool by name
    pub fn get_tool(&self, tool_name: &str) -> Option<Box<dyn Tool>> {
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
    pub fn list_bundles(&self) -> Vec<String> {
        let bundles = self.bundles.read().unwrap();
        bundles.keys().cloned().collect()
    }

    /// List all available tools
    pub fn list_tools(&self) -> Vec<String> {
        let tool_index = self.tool_index.read().unwrap();
        tool_index.keys().cloned().collect()
    }

    /// List all available package managers
    pub fn list_package_managers(&self) -> Vec<String> {
        let pm_index = self.pm_index.read().unwrap();
        pm_index.keys().cloned().collect()
    }

    /// Check if a tool is available
    pub fn has_tool(&self, tool_name: &str) -> bool {
        let tool_index = self.tool_index.read().unwrap();
        tool_index.contains_key(tool_name)
    }

    /// Check if a package manager is available
    pub fn has_package_manager(&self, pm_name: &str) -> bool {
        let pm_index = self.pm_index.read().unwrap();
        pm_index.contains_key(pm_name)
    }

    /// Get bundle information
    pub fn get_bundle_info(&self, bundle_name: &str) -> Option<HashMap<String, String>> {
        let bundles = self.bundles.read().unwrap();
        bundles.get(bundle_name).map(|bundle| bundle.metadata())
    }

    /// Get all bundle information
    pub fn get_all_bundle_info(&self) -> HashMap<String, HashMap<String, String>> {
        let bundles = self.bundles.read().unwrap();
        bundles
            .iter()
            .map(|(name, bundle)| (name.clone(), bundle.metadata()))
            .collect()
    }

    /// Shutdown all bundles
    pub async fn shutdown_all(&self) -> Result<()> {
        let bundles = {
            let mut bundles_guard = self.bundles.write().unwrap();
            std::mem::take(&mut *bundles_guard)
        };

        for (_, mut bundle) in bundles {
            if let Err(e) = bundle.shutdown().await {
                eprintln!(
                    "Warning: Failed to shutdown bundle {}: {}",
                    bundle.name(),
                    e
                );
            }
        }

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

/// Builder for creating and configuring a bundle registry
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
    pub fn with_bundle(mut self, bundle: Box<dyn ToolBundle>) -> Self {
        self.bundles.push(bundle);
        self
    }

    /// Build the bundle registry
    pub async fn build(self) -> Result<BundleRegistry> {
        let registry = BundleRegistry::new();

        for bundle in self.bundles {
            registry.register_bundle(bundle).await?;
        }

        Ok(registry)
    }
}

/// Simplified tool registry for backward compatibility
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

    /// Get a tool by name
    pub fn get_tool(&self, tool_name: &str) -> Option<Box<dyn Tool>> {
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
    pub fn get_all_tools(&self) -> Vec<Box<dyn Tool>> {
        let bundles = self.bundle_registry.bundles.read().unwrap();
        bundles.values().flat_map(|bundle| bundle.tools()).collect()
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}
