//! Combined plugin trait and plugin management functionality
//!
//! This module defines the `VxPlugin` trait, which is the main interface for
//! creating plugins that can provide both tools and package managers.

use crate::{Result, VxPackageManager, VxTool};
use async_trait::async_trait;
use std::collections::HashMap;

/// Combined plugin trait that can provide both tools and package managers
///
/// This is the main trait that plugin developers implement to register their functionality.
/// A plugin can provide tools, package managers, or both.
///
/// # Example
///
/// ```rust,no_run
/// use vx_plugin::{VxPlugin, VxTool, VxPackageManager, Result};
/// use async_trait::async_trait;
///
/// struct MyPlugin;
///
/// #[async_trait]
/// impl VxPlugin for MyPlugin {
///     fn name(&self) -> &str {
///         "my-plugin"
///     }
///
///     fn description(&self) -> &str {
///         "A plugin that provides custom tools and package managers"
///     }
///
///     fn tools(&self) -> Vec<Box<dyn VxTool>> {
///         // Return your tool implementations
///         vec![]
///     }
///
///     fn package_managers(&self) -> Vec<Box<dyn VxPackageManager>> {
///         // Return your package manager implementations
///         vec![]
///     }
/// }
/// ```
#[async_trait]
pub trait VxPlugin: Send + Sync {
    /// Plugin name (required)
    ///
    /// This should be a unique identifier for the plugin.
    fn name(&self) -> &str;

    /// Plugin description (optional)
    ///
    /// A human-readable description of what this plugin provides.
    fn description(&self) -> &str {
        "A vx plugin"
    }

    /// Plugin version (optional)
    ///
    /// The version of this plugin implementation.
    fn version(&self) -> &str {
        "0.1.0"
    }

    /// Plugin author (optional)
    ///
    /// Information about who created this plugin.
    fn author(&self) -> Option<&str> {
        None
    }

    /// Plugin homepage or repository URL (optional)
    ///
    /// URL where users can find more information about this plugin.
    fn homepage(&self) -> Option<&str> {
        None
    }

    /// Get all tools provided by this plugin
    ///
    /// Return a vector of tool implementations that this plugin provides.
    /// Return an empty vector if this plugin doesn't provide any tools.
    fn tools(&self) -> Vec<Box<dyn VxTool>> {
        vec![]
    }

    /// Get all package managers provided by this plugin
    ///
    /// Return a vector of package manager implementations that this plugin provides.
    /// Return an empty vector if this plugin doesn't provide any package managers.
    fn package_managers(&self) -> Vec<Box<dyn VxPackageManager>> {
        vec![]
    }
    /// Initialize the plugin (optional)
    ///
    /// This method is called when the plugin is loaded. Use it to perform
    /// any necessary setup, such as checking dependencies or initializing
    /// internal state.
    async fn initialize(&mut self) -> Result<()> {
        Ok(())
    }

    /// Shutdown the plugin (optional)
    ///
    /// This method is called when the plugin is being unloaded. Use it to
    /// perform cleanup operations.
    async fn shutdown(&mut self) -> Result<()> {
        Ok(())
    }

    /// Check if this plugin supports a specific tool
    ///
    /// Default implementation checks all tools provided by this plugin,
    /// including their aliases.
    fn supports_tool(&self, tool_name: &str) -> bool {
        self.tools()
            .iter()
            .any(|tool| tool.name() == tool_name || tool.aliases().contains(&tool_name))
    }

    /// Check if this plugin supports a specific package manager
    ///
    /// Default implementation checks all package managers provided by this plugin.
    fn supports_package_manager(&self, pm_name: &str) -> bool {
        self.package_managers()
            .iter()
            .any(|pm| pm.name() == pm_name)
    }

    /// Get a specific tool by name
    ///
    /// Returns the first tool that matches the given name or alias.
    /// Note: This method returns an owned Box since the tools are owned by the plugin.
    fn get_tool(&self, tool_name: &str) -> Option<Box<dyn VxTool>> {
        self.tools()
            .into_iter()
            .find(|tool| tool.name() == tool_name || tool.aliases().contains(&tool_name))
    }

    /// Get a specific package manager by name
    ///
    /// Returns the first package manager that matches the given name.
    /// Note: This method returns an owned Box since the package managers are owned by the plugin.
    fn get_package_manager(&self, pm_name: &str) -> Option<Box<dyn VxPackageManager>> {
        self.package_managers()
            .into_iter()
            .find(|pm| pm.name() == pm_name)
    }
    /// Check plugin compatibility with the current vx version
    ///
    /// Override this to implement version compatibility checks.
    /// The default implementation accepts all versions.
    fn is_compatible_with(&self, vx_version: &str) -> bool {
        let _ = vx_version;
        true
    }

    /// Get plugin dependencies
    ///
    /// Return a list of other plugins that this plugin depends on.
    /// The default implementation has no dependencies.
    fn dependencies(&self) -> Vec<&str> {
        vec![]
    }

    /// Get plugin configuration schema
    ///
    /// Return a JSON schema describing the configuration options
    /// that this plugin accepts. The default implementation has no configuration.
    fn config_schema(&self) -> Option<serde_json::Value> {
        None
    }

    /// Validate plugin configuration
    ///
    /// Check if the provided configuration is valid for this plugin.
    /// The default implementation accepts any configuration.
    fn validate_config(&self, _config: &serde_json::Value) -> Result<()> {
        Ok(())
    }

    /// Additional metadata for the plugin (optional)
    ///
    /// Override this to provide plugin-specific metadata such as
    /// supported platforms, feature flags, etc.
    fn metadata(&self) -> HashMap<String, String> {
        let mut metadata = HashMap::new();
        metadata.insert("name".to_string(), self.name().to_string());
        metadata.insert("version".to_string(), self.version().to_string());
        metadata.insert("description".to_string(), self.description().to_string());

        if let Some(author) = self.author() {
            metadata.insert("author".to_string(), author.to_string());
        }

        if let Some(homepage) = self.homepage() {
            metadata.insert("homepage".to_string(), homepage.to_string());
        }

        metadata
    }
}

/// Standard plugin implementation for single-tool plugins
///
/// This is a convenience implementation for plugins that provide a single tool.
/// It handles the boilerplate of implementing VxPlugin for simple cases.
pub struct StandardPlugin {
    name: String,
    description: String,
    version: String,
    author: Option<String>,
    homepage: Option<String>,
    tool_factory: Box<dyn Fn() -> Box<dyn VxTool> + Send + Sync>,
}

impl StandardPlugin {
    /// Create a new standard plugin
    ///
    /// # Arguments
    ///
    /// * `name` - Plugin name
    /// * `description` - Plugin description
    /// * `version` - Plugin version
    /// * `tool_factory` - Factory function that creates the tool instance
    pub fn new<F>(
        name: impl Into<String>,
        description: impl Into<String>,
        version: impl Into<String>,
        tool_factory: F,
    ) -> Self
    where
        F: Fn() -> Box<dyn VxTool> + Send + Sync + 'static,
    {
        Self {
            name: name.into(),
            description: description.into(),
            version: version.into(),
            author: None,
            homepage: None,
            tool_factory: Box::new(tool_factory),
        }
    }

    /// Set the plugin author
    pub fn with_author(mut self, author: impl Into<String>) -> Self {
        self.author = Some(author.into());
        self
    }

    /// Set the plugin homepage
    pub fn with_homepage(mut self, homepage: impl Into<String>) -> Self {
        self.homepage = Some(homepage.into());
        self
    }
}

#[async_trait]
impl VxPlugin for StandardPlugin {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn version(&self) -> &str {
        &self.version
    }

    fn author(&self) -> Option<&str> {
        self.author.as_deref()
    }

    fn homepage(&self) -> Option<&str> {
        self.homepage.as_deref()
    }

    fn tools(&self) -> Vec<Box<dyn VxTool>> {
        vec![(self.tool_factory)()]
    }
}
