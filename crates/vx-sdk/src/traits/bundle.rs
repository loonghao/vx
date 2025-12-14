//! Tool bundle trait definition
//!
//! The `ToolBundle` trait groups related tools and package managers together.

use crate::{PackageManager, Result, Tool};
use async_trait::async_trait;
use std::collections::HashMap;

/// Tool bundle trait that provides both tools and package managers
///
/// A bundle groups related tools and package managers together. For example,
/// a Node.js bundle might include the Node runtime, npm, and npx tools.
///
/// # Example
///
/// ```rust,no_run
/// use vx_sdk::{ToolBundle, Tool, PackageManager, Result};
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
///         "A bundle providing custom tools"
///     }
///
///     fn tools(&self) -> Vec<Box<dyn Tool>> {
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
    fn name(&self) -> &str;

    /// Bundle description
    fn description(&self) -> &str {
        "A vx tool bundle"
    }

    /// Bundle version
    fn version(&self) -> &str {
        "0.1.0"
    }

    /// Bundle author
    fn author(&self) -> Option<&str> {
        None
    }

    /// Bundle homepage or repository URL
    fn homepage(&self) -> Option<&str> {
        None
    }

    /// Get all tools provided by this bundle
    fn tools(&self) -> Vec<Box<dyn Tool>> {
        vec![]
    }

    /// Get all package managers provided by this bundle
    fn package_managers(&self) -> Vec<Box<dyn PackageManager>> {
        vec![]
    }

    /// Initialize the bundle
    async fn initialize(&mut self) -> Result<()> {
        Ok(())
    }

    /// Shutdown the bundle
    async fn shutdown(&mut self) -> Result<()> {
        Ok(())
    }

    /// Check if this bundle supports a specific tool
    fn supports_tool(&self, tool_name: &str) -> bool {
        self.tools()
            .iter()
            .any(|tool| tool.name() == tool_name || tool.aliases().contains(&tool_name))
    }

    /// Check if this bundle supports a specific package manager
    fn supports_package_manager(&self, pm_name: &str) -> bool {
        self.package_managers()
            .iter()
            .any(|pm| pm.name() == pm_name)
    }

    /// Get a specific tool by name
    fn get_tool(&self, tool_name: &str) -> Option<Box<dyn Tool>> {
        self.tools()
            .into_iter()
            .find(|tool| tool.name() == tool_name || tool.aliases().contains(&tool_name))
    }

    /// Get a specific package manager by name
    fn get_package_manager(&self, pm_name: &str) -> Option<Box<dyn PackageManager>> {
        self.package_managers()
            .into_iter()
            .find(|pm| pm.name() == pm_name)
    }

    /// Check bundle compatibility with vx version
    fn is_compatible_with(&self, vx_version: &str) -> bool {
        let _ = vx_version;
        true
    }

    /// Get bundle dependencies
    fn dependencies(&self) -> Vec<&str> {
        vec![]
    }

    /// Get bundle configuration schema
    fn config_schema(&self) -> Option<serde_json::Value> {
        None
    }

    /// Validate bundle configuration
    fn validate_config(&self, _config: &serde_json::Value) -> Result<()> {
        Ok(())
    }

    /// Additional metadata for the bundle
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
