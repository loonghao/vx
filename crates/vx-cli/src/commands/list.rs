//! List command implementation

use crate::ui::UI;
use anyhow::Result;
use vx_paths::{PathManager, PathResolver};
use vx_plugin::PluginRegistry;

pub async fn handle(
    registry: &PluginRegistry,
    tool: Option<&str>,
    show_status: bool,
) -> Result<()> {
    // Create path manager and resolver
    let path_manager = PathManager::new()
        .map_err(|e| anyhow::anyhow!("Failed to initialize path manager: {}", e))?;
    let resolver = PathResolver::new(path_manager);

    match tool {
        Some(tool_name) => {
            // List versions for a specific tool
            list_tool_versions(registry, &resolver, tool_name, show_status).await?;
        }
        None => {
            // List all tools
            list_all_tools(registry, &resolver, show_status).await?;
        }
    }
    Ok(())
}

async fn list_tool_versions(
    registry: &PluginRegistry,
    resolver: &PathResolver,
    tool_name: &str,
    show_status: bool,
) -> Result<()> {
    // Check if tool is supported
    let tool = registry.get_tool(tool_name);
    if tool.is_none() {
        UI::error(&format!("Tool '{}' is not supported", tool_name));
        UI::hint("Use 'vx list' to see all supported tools");
        return Ok(());
    }

    UI::info(&format!("📦 {}", tool_name));

    // Get installed versions
    let installed_versions = resolver.manager().list_tool_versions(tool_name)?;

    if installed_versions.is_empty() {
        UI::hint("  No versions installed");
        if show_status {
            UI::hint(&format!(
                "  Use 'vx install {}' to install this tool",
                tool_name
            ));

            // Show dependency information for uninstalled tools
            if let Some(tool) = registry.get_tool(tool_name) {
                let dependencies = tool.get_dependencies();
                if !dependencies.is_empty() {
                    UI::info("  📋 Dependencies:");
                    for dep in dependencies {
                        let dep_versions = resolver
                            .manager()
                            .list_tool_versions(&dep.tool_name)
                            .unwrap_or_default();
                        let status = if dep_versions.is_empty() {
                            "❌ not installed"
                        } else {
                            "✅ installed"
                        };
                        UI::info(&format!(
                            "     • {} - {} ({})",
                            dep.tool_name, dep.description, status
                        ));
                    }
                }
            }
        }
        return Ok(());
    }

    // Show installed versions
    for version in &installed_versions {
        let status_icon = if show_status { "✅" } else { "  " };
        println!("  {} {}", status_icon, version);

        if show_status {
            let exe_path = resolver.manager().tool_executable_path(tool_name, version);
            println!("     📁 {}", exe_path.display());
        }
    }

    if show_status {
        UI::success(&format!(
            "Total: {} version(s) installed",
            installed_versions.len()
        ));
    }

    Ok(())
}

async fn list_all_tools(
    registry: &PluginRegistry,
    resolver: &PathResolver,
    show_status: bool,
) -> Result<()> {
    UI::info("📦 Available Tools:");

    // Get all supported tools from registry
    let supported_tools = registry.list_tools();

    // Get all installed tools
    let installed_tools = resolver.manager().list_installed_tools()?;

    for tool_name in &supported_tools {
        let is_installed = installed_tools.contains(tool_name);
        let status_icon = if is_installed { "✅" } else { "❌" };

        if let Some(tool) = registry.get_tool(tool_name) {
            println!("  {} {} - {}", status_icon, tool_name, tool.description());

            if show_status && is_installed {
                let versions = resolver.manager().list_tool_versions(tool_name)?;
                if !versions.is_empty() {
                    println!("     Versions: {}", versions.join(", "));
                }
            }
        }
    }

    // Show summary
    if show_status {
        let total_supported = supported_tools.len();
        let total_installed = installed_tools.len();
        UI::info(&format!(
            "\n📊 Summary: {}/{} tools installed",
            total_installed, total_supported
        ));
    }

    Ok(())
}
