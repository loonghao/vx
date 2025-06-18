//! Package manager plugin example
//!
//! This example demonstrates how to create a package manager plugin for vx.

use vx_plugin::{VxPackageManager, VxPlugin, StandardPackageManager, Ecosystem, PackageSpec, PackageInfo, Result};
use async_trait::async_trait;
use std::path::Path;

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
        "An example package manager for demonstration purposes"
    }

    fn get_config_files(&self) -> Vec<&str> {
        vec!["example.json", "example.lock"]
    }

    async fn install_packages(&self, packages: &[PackageSpec], project_path: &Path) -> Result<()> {
        println!("Installing packages in {:?}:", project_path);
        for package in packages {
            if let Some(version) = &package.version {
                println!("  - {}@{}", package.name, version);
            } else {
                println!("  - {} (latest)", package.name);
            }
        }
        Ok(())
    }

    async fn list_packages(&self, _project_path: &Path) -> Result<Vec<PackageInfo>> {
        // In a real implementation, this would parse lock files or run list commands
        Ok(vec![
            PackageInfo {
                name: "example-package".to_string(),
                version: "1.0.0".to_string(),
                description: Some("An example package".to_string()),
                dev_dependency: false,
                metadata: std::collections::HashMap::new(),
            },
            PackageInfo {
                name: "dev-package".to_string(),
                version: "2.1.0".to_string(),
                description: Some("A development package".to_string()),
                dev_dependency: true,
                metadata: std::collections::HashMap::new(),
            },
        ])
    }

    async fn search_packages(&self, query: &str) -> Result<Vec<PackageInfo>> {
        println!("Searching for packages matching: {}", query);
        // In a real implementation, this would query a package registry
        Ok(vec![
            PackageInfo {
                name: format!("{}-result", query),
                version: "1.0.0".to_string(),
                description: Some(format!("A package matching {}", query)),
                dev_dependency: false,
                metadata: std::collections::HashMap::new(),
            },
        ])
    }
}

/// A plugin that provides the example package manager
struct ExamplePackageManagerPlugin;

#[async_trait]
impl VxPlugin for ExamplePackageManagerPlugin {
    fn name(&self) -> &str {
        "example-pm-plugin"
    }

    fn description(&self) -> &str {
        "A simple example package manager plugin"
    }

    fn version(&self) -> &str {
        "0.1.0"
    }

    fn author(&self) -> Option<&str> {
        Some("Example Author <author@example.com>")
    }

    fn package_managers(&self) -> Vec<Box<dyn VxPackageManager>> {
        vec![Box::new(ExamplePackageManager)]
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Create and register the plugin
    let plugin = Box::new(ExamplePackageManagerPlugin);
    
    println!("Plugin: {}", plugin.name());
    println!("Description: {}", plugin.description());
    println!("Version: {}", plugin.version());
    
    // List package managers provided by the plugin
    let package_managers = plugin.package_managers();
    println!("Package managers provided:");
    for pm in &package_managers {
        println!("  - {} ({})", pm.name(), pm.description());
        println!("    Ecosystem: {:?}", pm.ecosystem());
        println!("    Config files: {:?}", pm.get_config_files());
        
        // Test package listing
        let temp_dir = std::env::temp_dir();
        match pm.list_packages(&temp_dir).await {
            Ok(packages) => {
                println!("    Installed packages:");
                for package in packages {
                    let dev_marker = if package.dev_dependency { " (dev)" } else { "" };
                    println!("      - {}@{}{}", package.name, package.version, dev_marker);
                }
            }
            Err(e) => {
                println!("    Error listing packages: {}", e);
            }
        }
        
        // Test package search
        match pm.search_packages("test").await {
            Ok(results) => {
                println!("    Search results for 'test':");
                for result in results {
                    println!("      - {}@{}: {}", 
                        result.name, 
                        result.version, 
                        result.description.unwrap_or_else(|| "No description".to_string())
                    );
                }
            }
            Err(e) => {
                println!("    Error searching packages: {}", e);
            }
        }
    }
    
    // Demonstrate StandardPackageManager
    println!("\nDemonstrating StandardPackageManager:");
    let standard_pm = StandardPackageManager::new("npm", "Node Package Manager", Ecosystem::Node)
        .with_config_file("package.json")
        .with_config_file("package-lock.json");
    
    println!("Standard PM: {} ({})", standard_pm.name(), standard_pm.description());
    println!("Config files: {:?}", standard_pm.get_config_files());
    println!("Install command: {:?}", standard_pm.get_install_command());
    
    Ok(())
}