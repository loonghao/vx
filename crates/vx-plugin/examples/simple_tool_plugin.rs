//! Simple tool plugin example
//!
//! This example demonstrates how to create a basic tool plugin for vx.

use async_trait::async_trait;
use vx_plugin::{Result, VersionInfo, VxPlugin, VxTool};

/// A simple example tool
struct ExampleTool;

#[async_trait]
impl VxTool for ExampleTool {
    fn name(&self) -> &str {
        "example-tool"
    }

    fn description(&self) -> &str {
        "An example tool for demonstration purposes"
    }

    fn aliases(&self) -> Vec<&str> {
        vec!["example", "demo-tool"]
    }

    async fn fetch_versions(&self, include_prerelease: bool) -> Result<Vec<VersionInfo>> {
        // In a real implementation, this would fetch from an API or registry
        let mut versions = vec![
            VersionInfo::with_url("1.0.0", "https://example.com/releases/1.0.0.tar.gz"),
            VersionInfo::with_url("1.1.0", "https://example.com/releases/1.1.0.tar.gz"),
            VersionInfo::with_url("1.2.0", "https://example.com/releases/1.2.0.tar.gz"),
        ];

        if include_prerelease {
            versions.push(
                VersionInfo::with_url(
                    "1.3.0-beta.1",
                    "https://example.com/releases/1.3.0-beta.1.tar.gz",
                )
                .as_prerelease(),
            );
        }

        Ok(versions)
    }
}

/// A simple plugin that provides the example tool
struct ExamplePlugin;

#[async_trait]
impl VxPlugin for ExamplePlugin {
    fn name(&self) -> &str {
        "example-plugin"
    }

    fn description(&self) -> &str {
        "A simple example plugin"
    }

    fn version(&self) -> &str {
        "0.1.0"
    }

    fn author(&self) -> Option<&str> {
        Some("Example Author <author@example.com>")
    }

    fn tools(&self) -> Vec<Box<dyn VxTool>> {
        vec![Box::new(ExampleTool)]
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Create and register the plugin
    let plugin = Box::new(ExamplePlugin);

    println!("Plugin: {}", plugin.name());
    println!("Description: {}", plugin.description());
    println!("Version: {}", plugin.version());

    // List tools provided by the plugin
    let tools = plugin.tools();
    println!("Tools provided:");
    for tool in &tools {
        println!("  - {} ({})", tool.name(), tool.description());

        // Fetch and display versions
        match tool.fetch_versions(true).await {
            Ok(versions) => {
                println!("    Available versions:");
                for version in versions {
                    let prerelease = if version.prerelease {
                        " (prerelease)"
                    } else {
                        ""
                    };
                    println!("      - {}{}", version.version, prerelease);
                }
            }
            Err(e) => {
                println!("    Error fetching versions: {}", e);
            }
        }
    }

    Ok(())
}
