//! Plugin and tool registry for managing loaded plugins

use crate::{Result, VxError, VxPackageManager, VxPlugin, VxTool};
use std::collections::HashMap;

/// Registry for managing all loaded plugins
pub struct PluginRegistry {
    plugins: Vec<Box<dyn VxPlugin>>,
    tool_cache: HashMap<String, usize>, // tool_name -> plugin_index
    pm_cache: HashMap<String, usize>,   // pm_name -> plugin_index
}

impl PluginRegistry {
    /// Create a new plugin registry
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
            tool_cache: HashMap::new(),
            pm_cache: HashMap::new(),
        }
    }

    /// Register a new plugin
    pub fn register(&mut self, plugin: Box<dyn VxPlugin>) -> Result<()> {
        let plugin_index = self.plugins.len();

        // Cache tool mappings
        for tool in plugin.tools() {
            let tool_name = tool.name().to_string();
            if self.tool_cache.contains_key(&tool_name) {
                return Err(VxError::Other {
                    message: format!("Tool '{}' is already registered", tool_name),
                });
            }
            self.tool_cache.insert(tool_name.clone(), plugin_index);

            // Also register aliases
            for alias in tool.aliases() {
                self.tool_cache.insert(alias.to_string(), plugin_index);
            }
        }

        // Cache package manager mappings
        for pm in plugin.package_managers() {
            let pm_name = pm.name().to_string();
            if self.pm_cache.contains_key(&pm_name) {
                return Err(VxError::Other {
                    message: format!("Package manager '{}' is already registered", pm_name),
                });
            }
            self.pm_cache.insert(pm_name, plugin_index);
        }

        self.plugins.push(plugin);
        Ok(())
    }

    /// Find plugin that supports the given tool
    pub fn find_plugin_for_tool(&self, tool_name: &str) -> Option<&dyn VxPlugin> {
        self.tool_cache
            .get(tool_name)
            .and_then(|&index| self.plugins.get(index))
            .map(|plugin| plugin.as_ref())
    }

    /// Find plugin that supports the given package manager
    pub fn find_plugin_for_package_manager(&self, pm_name: &str) -> Option<&dyn VxPlugin> {
        self.pm_cache
            .get(pm_name)
            .and_then(|&index| self.plugins.get(index))
            .map(|plugin| plugin.as_ref())
    }

    /// Get tool by name
    pub fn get_tool(&self, tool_name: &str) -> Option<Box<dyn VxTool>> {
        if let Some(plugin) = self.find_plugin_for_tool(tool_name) {
            plugin
                .tools()
                .into_iter()
                .find(|tool| tool.name() == tool_name || tool.aliases().contains(&tool_name))
        } else {
            None
        }
    }

    /// Get package manager by name
    pub fn get_package_manager(&self, pm_name: &str) -> Option<Box<dyn VxPackageManager>> {
        if let Some(plugin) = self.find_plugin_for_package_manager(pm_name) {
            plugin
                .package_managers()
                .into_iter()
                .find(|pm| pm.name() == pm_name)
        } else {
            None
        }
    }

    /// Get all available tools from all plugins
    pub fn get_all_tools(&self) -> Vec<Box<dyn VxTool>> {
        self.plugins
            .iter()
            .flat_map(|plugin| plugin.tools())
            .collect()
    }

    /// Get all available package managers from all plugins
    pub fn get_all_package_managers(&self) -> Vec<Box<dyn VxPackageManager>> {
        self.plugins
            .iter()
            .flat_map(|plugin| plugin.package_managers())
            .collect()
    }

    /// Get all registered tool names
    pub fn get_tool_names(&self) -> Vec<String> {
        self.tool_cache.keys().cloned().collect()
    }

    /// Get all registered package manager names
    pub fn get_package_manager_names(&self) -> Vec<String> {
        self.pm_cache.keys().cloned().collect()
    }

    /// Check if a tool is supported
    pub fn supports_tool(&self, tool_name: &str) -> bool {
        self.tool_cache.contains_key(tool_name)
    }

    /// Check if a package manager is supported
    pub fn supports_package_manager(&self, pm_name: &str) -> bool {
        self.pm_cache.contains_key(pm_name)
    }

    /// Get all plugins
    pub fn get_plugins(&self) -> &[Box<dyn VxPlugin>] {
        &self.plugins
    }

    /// Initialize all plugins
    pub async fn initialize_all(&mut self) -> Result<()> {
        for plugin in &mut self.plugins {
            plugin.initialize().await?;
        }
        Ok(())
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Simplified tool registry for backward compatibility
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

    /// Register a tool plugin
    pub fn register_plugin(&mut self, plugin: Box<dyn VxPlugin>) -> Result<()> {
        self.plugin_registry.register(plugin)
    }

    /// Get a tool by name
    pub fn get_tool(&self, tool_name: &str) -> Option<Box<dyn VxTool>> {
        self.plugin_registry.get_tool(tool_name)
    }

    /// Check if a tool is supported
    pub fn supports_tool(&self, tool_name: &str) -> bool {
        self.plugin_registry.supports_tool(tool_name)
    }

    /// Get all tool names
    pub fn get_tool_names(&self) -> Vec<String> {
        self.plugin_registry.get_tool_names()
    }

    /// Get all tools
    pub fn get_all_tools(&self) -> Vec<Box<dyn VxTool>> {
        self.plugin_registry.get_all_tools()
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Ecosystem, PackageSpec, VersionInfo};
    use std::path::Path;

    // Mock implementations for testing
    struct MockTool {
        name: String,
    }

    #[async_trait::async_trait]
    impl VxTool for MockTool {
        fn name(&self) -> &str {
            &self.name
        }

        async fn fetch_versions(&self, _include_prerelease: bool) -> Result<Vec<VersionInfo>> {
            Ok(vec![VersionInfo::new("1.0.0".to_string())])
        }
    }

    struct MockPackageManager {
        name: String,
    }

    #[async_trait::async_trait]
    impl VxPackageManager for MockPackageManager {
        fn name(&self) -> &str {
            &self.name
        }

        fn ecosystem(&self) -> Ecosystem {
            Ecosystem::Other("test".to_string())
        }

        async fn install_packages(
            &self,
            _packages: &[PackageSpec],
            _project_path: &Path,
        ) -> Result<()> {
            Ok(())
        }
    }

    struct MockPlugin {
        name: String,
    }

    #[async_trait::async_trait]
    impl VxPlugin for MockPlugin {
        fn name(&self) -> &str {
            &self.name
        }

        fn tools(&self) -> Vec<Box<dyn VxTool>> {
            vec![Box::new(MockTool {
                name: "test-tool".to_string(),
            })]
        }

        fn package_managers(&self) -> Vec<Box<dyn VxPackageManager>> {
            vec![Box::new(MockPackageManager {
                name: "test-pm".to_string(),
            })]
        }
    }

    #[tokio::test]
    async fn test_plugin_registry() {
        let mut registry = PluginRegistry::new();

        let plugin = Box::new(MockPlugin {
            name: "test-plugin".to_string(),
        });

        assert!(registry.register(plugin).is_ok());
        assert!(registry.supports_tool("test-tool"));
        assert!(registry.supports_package_manager("test-pm"));
    }
}
