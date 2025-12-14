//! Tool bundle trait and management functionality
//!
//! This module defines the `ToolBundle` trait (formerly `VxPlugin`), which is the main
//! interface for creating bundles that can provide both tools and package managers.

use crate::{PackageManager, Result, VxTool};
use async_trait::async_trait;
use std::collections::HashMap;

/// Tool bundle trait that can provide both tools and package managers
///
/// A bundle groups related tools and package managers together. For example,
/// a Node.js bundle might include the Node runtime, npm, and npx tools.
///
/// # Example
///
/// ```rust,no_run
/// use vx_plugin::{ToolBundle, VxTool, PackageManager, Result};
/// use async_trait::async_trait;
///
/// struct MyBundle;
///
/// #[async_trait]
/// impl ToolBundle for MyBundle {
///     fn name(&self) -> &str {
///         "my-bundle"
///     }
///
///     fn description(&self) -> &str {
///         "A bundle that provides custom tools and package managers"
///     }
///
///     fn tools(&self) -> Vec<Box<dyn VxTool>> {
///         vec![]
///     }
///
///     fn package_managers(&self) -> Vec<Box<dyn PackageManager>> {
///         vec![]
///     }
/// }
/// ```
#[async_trait]
pub trait ToolBundle: Send + Sync {
    /// Bundle name (required)
    ///
    /// This should be a unique identifier for the bundle.
    fn name(&self) -> &str;

    /// Bundle description (optional)
    ///
    /// A human-readable description of what this bundle provides.
    fn description(&self) -> &str {
        "A vx tool bundle"
    }

    /// Bundle version (optional)
    ///
    /// The version of this bundle implementation.
    fn version(&self) -> &str {
        "0.1.0"
    }

    /// Bundle author (optional)
    ///
    /// Information about who created this bundle.
    fn author(&self) -> Option<&str> {
        None
    }

    /// Bundle homepage or repository URL (optional)
    ///
    /// URL where users can find more information about this bundle.
    fn homepage(&self) -> Option<&str> {
        None
    }

    /// Get all tools provided by this bundle
    ///
    /// Return a vector of tool implementations that this bundle provides.
    /// Return an empty vector if this bundle doesn't provide any tools.
    fn tools(&self) -> Vec<Box<dyn VxTool>> {
        vec![]
    }

    /// Get all package managers provided by this bundle
    ///
    /// Return a vector of package manager implementations that this bundle provides.
    /// Return an empty vector if this bundle doesn't provide any package managers.
    fn package_managers(&self) -> Vec<Box<dyn PackageManager>> {
        vec![]
    }

    /// Initialize the bundle (optional)
    ///
    /// This method is called when the bundle is loaded. Use it to perform
    /// any necessary setup, such as checking dependencies or initializing
    /// internal state.
    async fn initialize(&mut self) -> Result<()> {
        Ok(())
    }

    /// Shutdown the bundle (optional)
    ///
    /// This method is called when the bundle is being unloaded. Use it to
    /// perform cleanup operations.
    async fn shutdown(&mut self) -> Result<()> {
        Ok(())
    }

    /// Check if this bundle supports a specific tool
    ///
    /// Default implementation checks all tools provided by this bundle,
    /// including their aliases.
    fn supports_tool(&self, tool_name: &str) -> bool {
        self.tools()
            .iter()
            .any(|tool| tool.name() == tool_name || tool.aliases().contains(&tool_name))
    }

    /// Check if this bundle supports a specific package manager
    ///
    /// Default implementation checks all package managers provided by this bundle.
    fn supports_package_manager(&self, pm_name: &str) -> bool {
        self.package_managers()
            .iter()
            .any(|pm| pm.name() == pm_name)
    }

    /// Get a specific tool by name
    ///
    /// Returns the first tool that matches the given name or alias.
    fn get_tool(&self, tool_name: &str) -> Option<Box<dyn VxTool>> {
        self.tools()
            .into_iter()
            .find(|tool| tool.name() == tool_name || tool.aliases().contains(&tool_name))
    }

    /// Get a specific package manager by name
    ///
    /// Returns the first package manager that matches the given name.
    fn get_package_manager(&self, pm_name: &str) -> Option<Box<dyn PackageManager>> {
        self.package_managers()
            .into_iter()
            .find(|pm| pm.name() == pm_name)
    }

    /// Check bundle compatibility with the current vx version
    ///
    /// Override this to implement version compatibility checks.
    /// The default implementation accepts all versions.
    fn is_compatible_with(&self, vx_version: &str) -> bool {
        let _ = vx_version;
        true
    }

    /// Get bundle dependencies
    ///
    /// Return a list of other bundles that this bundle depends on.
    /// The default implementation has no dependencies.
    fn dependencies(&self) -> Vec<&str> {
        vec![]
    }

    /// Get bundle configuration schema
    ///
    /// Return a JSON schema describing the configuration options
    /// that this bundle accepts. The default implementation has no configuration.
    fn config_schema(&self) -> Option<serde_json::Value> {
        None
    }

    /// Validate bundle configuration
    ///
    /// Check if the provided configuration is valid for this bundle.
    /// The default implementation accepts any configuration.
    fn validate_config(&self, _config: &serde_json::Value) -> Result<()> {
        Ok(())
    }

    /// Additional metadata for the bundle (optional)
    ///
    /// Override this to provide bundle-specific metadata such as
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

/// Standard bundle implementation for single-tool bundles
///
/// This is a convenience implementation for bundles that provide a single tool.
/// It handles the boilerplate of implementing ToolBundle for simple cases.
pub struct StandardBundle {
    name: String,
    description: String,
    version: String,
    author: Option<String>,
    homepage: Option<String>,
    tool_factory: Box<dyn Fn() -> Box<dyn VxTool> + Send + Sync>,
}

impl StandardBundle {
    /// Create a new standard bundle
    ///
    /// # Arguments
    ///
    /// * `name` - Bundle name
    /// * `description` - Bundle description
    /// * `version` - Bundle version
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

    /// Set the bundle author
    pub fn with_author(mut self, author: impl Into<String>) -> Self {
        self.author = Some(author.into());
        self
    }

    /// Set the bundle homepage
    pub fn with_homepage(mut self, homepage: impl Into<String>) -> Self {
        self.homepage = Some(homepage.into());
        self
    }
}

#[async_trait]
impl ToolBundle for StandardBundle {
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
