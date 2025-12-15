//! List command implementation

use crate::ui::UI;
use anyhow::Result;
use vx_paths::{PathManager, PathResolver};
use vx_runtime::{ProviderRegistry, RuntimeContext};

pub async fn handle(
    registry: &ProviderRegistry,
    _context: &RuntimeContext,
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
    registry: &ProviderRegistry,
    resolver: &PathResolver,
    tool_name: &str,
    show_status: bool,
) -> Result<()> {
    // Check if tool is supported
    let runtime = registry.get_runtime(tool_name);
    if runtime.is_none() {
        UI::error(&format!("Tool '{}' is not supported", tool_name));
        UI::hint("Use 'vx list' to see all supported tools");
        return Ok(());
    }

    let runtime = runtime.unwrap();
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

    let _ = runtime; // Silence unused warning
    Ok(())
}

async fn list_all_tools(
    registry: &ProviderRegistry,
    resolver: &PathResolver,
    show_status: bool,
) -> Result<()> {
    UI::info("📦 Available Tools:");

    // Get all supported tools from registry
    let supported_tools = registry.runtime_names();

    // Get all installed tools
    let installed_tools = resolver.manager().list_installed_tools()?;

    for tool_name in &supported_tools {
        let is_installed = installed_tools.contains(tool_name);
        let status_icon = if is_installed { "✅" } else { "❌" };

        if let Some(runtime) = registry.get_runtime(tool_name) {
            println!(
                "  {} {} - {}",
                status_icon,
                tool_name,
                runtime.description()
            );

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
