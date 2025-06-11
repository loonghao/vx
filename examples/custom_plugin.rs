//! Example showing how to create a custom plugin
//!
//! This demonstrates the minimal code needed to add support for a new tool.
//!
//! Run with: cargo run --example custom_plugin

use vx_core::{VxTool, VxPlugin, VersionInfo, Result, PluginRegistry};

/// Example: Custom Python tool implementation
/// 
/// This shows how easy it is to add a new tool to vx.
#[derive(Default)]
struct CustomPythonTool;

#[async_trait::async_trait]
impl VxTool for CustomPythonTool {
    fn name(&self) -> &str {
        "python"
    }
    
    fn description(&self) -> &str {
        "Python programming language"
    }
    
    fn aliases(&self) -> Vec<&str> {
        vec!["py", "python3"]
    }
    
    /// This is the main method developers need to implement
    async fn fetch_versions(&self, _include_prerelease: bool) -> Result<Vec<VersionInfo>> {
        // In a real implementation, this would fetch from python.org or GitHub API
        // For this example, we'll return some mock data
        Ok(vec![
            VersionInfo::new("3.12.1".to_string())
                .with_release_date("2023-12-07".to_string())
                .with_release_notes("Latest stable release".to_string()),
            VersionInfo::new("3.11.7".to_string())
                .with_release_date("2023-12-04".to_string())
                .with_release_notes("Security and bug fixes".to_string()),
            VersionInfo::new("3.10.13".to_string())
                .with_release_date("2023-08-24".to_string())
                .with_release_notes("Maintenance release".to_string()),
        ])
    }
    
    /// Optional: Override download URL generation
    async fn get_download_url(&self, version: &str) -> Result<Option<String>> {
        // Generate platform-specific download URLs
        let (os, arch) = if cfg!(target_os = "windows") {
            ("windows", if cfg!(target_arch = "x86_64") { "amd64" } else { "win32" })
        } else if cfg!(target_os = "macos") {
            ("macos", if cfg!(target_arch = "aarch64") { "universal2" } else { "universal2" })
        } else if cfg!(target_os = "linux") {
            ("linux", if cfg!(target_arch = "aarch64") { "aarch64" } else { "x86_64" })
        } else {
            return Ok(None);
        };

        let ext = if cfg!(target_os = "windows") { "exe" } else { "tgz" };
        
        Ok(Some(format!(
            "https://www.python.org/ftp/python/{}/python-{}-{}-{}.{}",
            version, version, os, arch, ext
        )))
    }
}

/// Example: Custom plugin that provides the Python tool
#[derive(Default)]
struct CustomPythonPlugin;

#[async_trait::async_trait]
impl VxPlugin for CustomPythonPlugin {
    fn name(&self) -> &str {
        "custom-python"
    }
    
    fn description(&self) -> &str {
        "Custom Python support for vx"
    }
    
    fn version(&self) -> &str {
        "1.0.0"
    }
    
    fn tools(&self) -> Vec<Box<dyn VxTool>> {
        vec![Box::new(CustomPythonTool::default())]
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🐍 Custom Plugin Example");
    println!("========================");
    
    // Create a registry and register our custom plugin
    let mut registry = PluginRegistry::new();
    
    println!("📦 Registering custom Python plugin...");
    registry.register(Box::new(CustomPythonPlugin::default()))?;
    
    // Initialize plugins
    registry.initialize_all().await?;
    
    // Test our custom tool
    if let Some(python_tool) = registry.get_tool("python") {
        println!("\n🔍 Testing custom Python tool:");
        println!("  Name: {}", python_tool.name());
        println!("  Description: {}", python_tool.description());
        println!("  Aliases: {:?}", python_tool.aliases());
        
        // Fetch versions
        println!("\n📋 Available Python versions:");
        let versions = python_tool.fetch_versions(false).await?;
        for version in &versions {
            println!("  - {} ({})", 
                version.version, 
                version.release_date.as_deref().unwrap_or("unknown date")
            );
        }
        
        // Test download URL generation
        if let Some(version) = versions.first() {
            if let Ok(Some(url)) = python_tool.get_download_url(&version.version).await {
                println!("\n🔗 Download URL for {}:", version.version);
                println!("  {}", url);
            }
        }
    }
    
    // Show that aliases work
    println!("\n🔄 Testing aliases:");
    for alias in &["py", "python3"] {
        if registry.supports_tool(alias) {
            println!("  ✅ '{}' is supported", alias);
        } else {
            println!("  ❌ '{}' is not supported", alias);
        }
    }
    
    println!("\n✅ Custom plugin example completed!");
    println!("\n💡 This demonstrates how easy it is to add new tools to vx:");
    println!("   1. Implement VxTool trait (~50 lines)");
    println!("   2. Implement VxPlugin trait (~20 lines)");
    println!("   3. Register the plugin");
    println!("   That's it! 🎉");
    
    Ok(())
}
