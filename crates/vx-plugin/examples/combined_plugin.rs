//! Combined plugin example
//!
//! This example demonstrates how to create a plugin that provides both tools and package managers.

use vx_plugin::{
    VxTool, VxPackageManager, VxPlugin, PluginRegistry, PluginRegistryBuilder,
    Ecosystem, PackageSpec, PackageInfo, VersionInfo, Result
};
use async_trait::async_trait;
use std::path::Path;

/// A simple example tool
struct ExampleTool;

#[async_trait]
impl VxTool for ExampleTool {
    fn name(&self) -> &str {
        "example-tool"
    }

    fn description(&self) -> &str {
        "An example development tool"
    }

    fn aliases(&self) -> Vec<&str> {
        vec!["et", "example"]
    }

    async fn fetch_versions(&self, include_prerelease: bool) -> Result<Vec<VersionInfo>> {
        let mut versions = vec![
            VersionInfo::new("1.0.0"),
            VersionInfo::new("1.1.0"),
            VersionInfo::new("1.2.0"),
        ];

        if include_prerelease {
            versions.push(VersionInfo::new("1.3.0-beta.1").as_prerelease());
        }

        Ok(versions)
    }
}

/// A simple example package manager
struct ExamplePackageManager;

#[async_trait]
impl VxPackageManager for ExamplePackageManager {
    fn name(&self) -> &str {
        "example-pm"
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::Node
    }

    fn description(&self) -> &str {
        "An example package manager"
    }

    fn get_config_files(&self) -> Vec<&str> {
        vec!["example.json", "example.lock"]
    }

    async fn install_packages(&self, packages: &[PackageSpec], project_path: &Path) -> Result<()> {
        println!("Installing packages in {:?}:", project_path);
        for package in packages {
            println!("  - {}", package.name);
        }
        Ok(())
    }
}

/// A combined plugin that provides both tools and package managers
struct CombinedPlugin {
    name: String,
    description: String,
    version: String,
}

impl CombinedPlugin {
    pub fn new() -> Self {
        Self {
            name: "combined-example-plugin".to_string(),
            description: "A plugin that provides both tools and package managers".to_string(),
            version: "1.0.0".to_string(),
        }
    }
}

#[async_trait]
impl VxPlugin for CombinedPlugin {
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
        Some("Example Author <author@example.com>")
    }

    fn homepage(&self) -> Option<&str> {
        Some("https://github.com/example/combined-plugin")
    }

    fn tools(&self) -> Vec<Box<dyn VxTool>> {
        vec![Box::new(ExampleTool)]
    }

    fn package_managers(&self) -> Vec<Box<dyn VxPackageManager>> {
        vec![Box::new(ExamplePackageManager)]
    }

    async fn initialize(&mut self) -> Result<()> {
        println!("Initializing combined plugin: {}", self.name);
        Ok(())
    }

    async fn shutdown(&mut self) -> Result<()> {
        println!("Shutting down combined plugin: {}", self.name);
        Ok(())
    }

    fn dependencies(&self) -> Vec<&str> {
        vec![] // No dependencies for this example
    }

    fn is_compatible_with(&self, vx_version: &str) -> bool {
        // Simple version compatibility check
        vx_version.starts_with("0.") || vx_version.starts_with("1.")
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== Combined Plugin Example ===\n");

    // Create the plugin
    let plugin = Box::new(CombinedPlugin::new());
    
    println!("Plugin Information:");
    println!("  Name: {}", plugin.name());
    println!("  Description: {}", plugin.description());
    println!("  Version: {}", plugin.version());
    println!("  Author: {}", plugin.author().unwrap_or("Unknown"));
    println!("  Homepage: {}", plugin.homepage().unwrap_or("None"));
    println!("  Compatible with vx 0.2.6: {}", plugin.is_compatible_with("0.2.6"));
    println!();

    // Test tool functionality
    println!("Tools provided:");
    let tools = plugin.tools();
    for tool in &tools {
        println!("  - {} ({})", tool.name(), tool.description());
        println!("    Aliases: {:?}", tool.aliases());
        
        // Test version fetching
        match tool.fetch_versions(false).await {
            Ok(versions) => {
                println!("    Available versions:");
                for version in versions {
                    println!("      - {}", version.version);
                }
            }
            Err(e) => {
                println!("    Error fetching versions: {}", e);
            }
        }
    }
    println!();

    // Test package manager functionality
    println!("Package managers provided:");
    let package_managers = plugin.package_managers();
    for pm in &package_managers {
        println!("  - {} ({})", pm.name(), pm.description());
        println!("    Ecosystem: {:?}", pm.ecosystem());
        println!("    Config files: {:?}", pm.get_config_files());
    }
    println!();

    // Demonstrate plugin registry usage
    println!("=== Plugin Registry Demo ===\n");
    
    // Create registry using builder pattern
    let registry = PluginRegistryBuilder::new()
        .with_plugin(Box::new(CombinedPlugin::new()))
        .build()
        .await?;

    println!("Registry contents:");
    println!("  Plugins: {:?}", registry.list_plugins());
    println!("  Tools: {:?}", registry.list_tools());
    println!("  Package managers: {:?}", registry.list_package_managers());
    println!();

    // Test tool lookup
    if let Some(tool) = registry.get_tool("example-tool") {
        println!("Found tool: {}", tool.name());
    }

    // Test alias lookup
    if let Some(tool) = registry.get_tool("et") {
        println!("Found tool by alias: {}", tool.name());
    }

    // Test package manager lookup
    if let Some(pm) = registry.get_package_manager("example-pm") {
        println!("Found package manager: {}", pm.name());
    }

    // Test availability checks
    println!("Tool availability:");
    println!("  example-tool: {}", registry.has_tool("example-tool"));
    println!("  et (alias): {}", registry.has_tool("et"));
    println!("  nonexistent: {}", registry.has_tool("nonexistent"));
    
    println!("Package manager availability:");
    println!("  example-pm: {}", registry.has_package_manager("example-pm"));
    println!("  nonexistent: {}", registry.has_package_manager("nonexistent"));
    println!();

    // Get plugin metadata
    if let Some(metadata) = registry.get_plugin_info("combined-example-plugin") {
        println!("Plugin metadata:");
        for (key, value) in metadata {
            println!("  {}: {}", key, value);
        }
    }

    // Shutdown registry (this will call shutdown on all plugins)
    println!("\nShutting down registry...");
    registry.shutdown_all().await?;

    Ok(())
}