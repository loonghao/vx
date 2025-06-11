//! Basic usage example for vx
//!
//! This example demonstrates how to use the vx plugin system to manage tools.
//!
//! Run with: cargo run --example basic_usage

use vx_core::{PluginRegistry};
use vx::{create_default_plugins};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 VX - Universal Development Tool Manager");
    println!("==========================================");
    
    // Create a new plugin registry
    let mut registry = PluginRegistry::new();
    
    // Register all default plugins
    println!("📦 Registering plugins...");
    for plugin in create_default_plugins() {
        println!("  - {}: {}", plugin.name(), plugin.description());
        registry.register(plugin)?;
    }
    
    // Initialize all plugins
    println!("\n🔧 Initializing plugins...");
    registry.initialize_all().await?;
    
    // List all available tools
    println!("\n🛠️  Available tools:");
    for tool_name in registry.get_tool_names() {
        if let Some(tool) = registry.get_tool(&tool_name) {
            println!("  - {}: {}", tool.name(), tool.description());
        }
    }
    
    // List all available package managers
    println!("\n📦 Available package managers:");
    for pm_name in registry.get_package_manager_names() {
        if let Some(pm) = registry.get_package_manager(&pm_name) {
            println!("  - {} ({}): {}", pm.name(), format!("{:?}", pm.ecosystem()), pm.description());
        }
    }
    
    // Example: Fetch Node.js versions
    if let Some(node_tool) = registry.get_tool("node") {
        println!("\n🔍 Fetching Node.js versions...");
        match node_tool.fetch_versions(false).await {
            Ok(versions) => {
                println!("  Found {} versions", versions.len());
                for (i, version) in versions.iter().take(5).enumerate() {
                    let lts_marker = if version.metadata.get("lts") == Some(&"true".to_string()) {
                        " (LTS)"
                    } else {
                        ""
                    };
                    println!("  {}. {}{}", i + 1, version.version, lts_marker);
                }
                if versions.len() > 5 {
                    println!("  ... and {} more", versions.len() - 5);
                }
            }
            Err(e) => {
                println!("  ❌ Failed to fetch versions: {}", e);
            }
        }
    }
    
    println!("\n✅ Example completed successfully!");
    Ok(())
}
