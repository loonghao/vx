//! List command implementation

use crate::ui::UI;
use anyhow::Result;
use std::collections::HashSet;
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
        // Show friendly error with suggestions
        let available_tools = registry.runtime_names();
        UI::tool_not_found(tool_name, &available_tools);
        return Ok(());
    }

    let runtime = runtime.unwrap();
    UI::info(&format!("📦 {}", tool_name));

    // Check if this tool is bundled with another tool
    let bundled_with = runtime.metadata().get("bundled_with").cloned();

    // Get installed versions - check both the tool itself and its parent (if bundled)
    let mut installed_executables = resolver.find_tool_executables(tool_name)?;

    // If this tool is bundled with another and has no direct installations,
    // check the parent tool's installations
    if installed_executables.is_empty() {
        if let Some(parent_tool) = &bundled_with {
            let parent_executables = resolver.find_tool_executables(parent_tool)?;
            if !parent_executables.is_empty() {
                // Tool is available via parent - show parent's versions
                for exe_path in &parent_executables {
                    let version = extract_version_from_path(exe_path);
                    installed_executables.push(exe_path.clone());
                    let status_icon = if show_status { "✅" } else { "  " };
                    println!(
                        "  {} {} (bundled with {})",
                        status_icon, version, parent_tool
                    );
                }

                if show_status {
                    UI::success(&format!(
                        "Total: {} version(s) available (bundled with {})",
                        parent_executables.len(),
                        parent_tool
                    ));
                }
                return Ok(());
            }
        }
    }

    if installed_executables.is_empty() {
        UI::hint("  No versions installed");
        if show_status {
            if let Some(parent_tool) = bundled_with {
                UI::hint(&format!(
                    "  This tool is bundled with '{}'. Install {} to get {}.",
                    parent_tool, parent_tool, tool_name
                ));
            } else {
                UI::hint(&format!(
                    "  Use 'vx install {}' to install this tool",
                    tool_name
                ));
            }
        }
        return Ok(());
    }

    // Show installed versions
    for exe_path in &installed_executables {
        let status_icon = if show_status { "✅" } else { "  " };
        // Extract version from path if possible
        let version = extract_version_from_path(exe_path);
        println!("  {} {}", status_icon, version);

        if show_status {
            println!("     📁 {}", exe_path.display());
        }
    }

    if show_status {
        UI::success(&format!(
            "Total: {} version(s) installed",
            installed_executables.len()
        ));
    }

    Ok(())
}

/// Extract version from executable path
/// Paths are like: ~/.vx/store/uv/0.9.17/uv-platform/uv
/// or: ~/.vx/tools/node/18.17.0/node
fn extract_version_from_path(path: &std::path::Path) -> String {
    // Walk up the path to find a version-like component
    for ancestor in path.ancestors() {
        if let Some(name) = ancestor.file_name().and_then(|n| n.to_str()) {
            // Check if this looks like a version (contains digits and dots)
            if name.chars().any(|c| c.is_ascii_digit())
                && (name.contains('.') || name.chars().all(|c| c.is_ascii_digit()))
                && !name.contains('-')
            {
                return name.to_string();
            }
        }
    }
    "unknown".to_string()
}

async fn list_all_tools(
    registry: &ProviderRegistry,
    resolver: &PathResolver,
    show_status: bool,
) -> Result<()> {
    UI::info("📦 Available Tools:");

    // Get all supported tools from registry
    let supported_tools = registry.runtime_names();

    // Get all installed tools (from both store and tools directories)
    let installed_tools_with_versions = resolver.get_installed_tools_with_versions()?;
    let directly_installed: HashSet<_> = installed_tools_with_versions
        .iter()
        .map(|(name, _)| name.as_str())
        .collect();

    // Build a set of tools that are available (either directly installed or bundled with an installed tool)
    let mut available_tools: HashSet<String> =
        directly_installed.iter().map(|s| s.to_string()).collect();

    // Check for bundled tools - if a parent tool is installed, its bundled tools are also available
    for tool_name in &supported_tools {
        if let Some(runtime) = registry.get_runtime(tool_name) {
            if let Some(parent_tool) = runtime.metadata().get("bundled_with") {
                if directly_installed.contains(parent_tool.as_str()) {
                    available_tools.insert(tool_name.clone());
                }
            }
        }
    }

    let mut installed_count = 0;

    for tool_name in &supported_tools {
        let is_directly_installed = directly_installed.contains(tool_name.as_str());
        let is_available = available_tools.contains(tool_name);
        let status_icon = if is_available { "✅" } else { "❌" };

        if is_available {
            installed_count += 1;
        }

        if let Some(runtime) = registry.get_runtime(tool_name) {
            println!(
                "  {} {} - {}",
                status_icon,
                tool_name,
                runtime.description()
            );

            if show_status && is_available {
                // Find versions for this tool
                if is_directly_installed {
                    if let Some((_, versions)) = installed_tools_with_versions
                        .iter()
                        .find(|(name, _)| name == tool_name)
                    {
                        if !versions.is_empty() {
                            println!("     Versions: {}", versions.join(", "));
                        }
                    }
                } else {
                    // Bundled tool - show parent's versions
                    if let Some(parent_tool) = runtime.metadata().get("bundled_with") {
                        if let Some((_, versions)) = installed_tools_with_versions
                            .iter()
                            .find(|(name, _)| name == parent_tool)
                        {
                            if !versions.is_empty() {
                                println!(
                                    "     Versions: {} (via {})",
                                    versions.join(", "),
                                    parent_tool
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    // Show summary
    if show_status {
        let total_supported = supported_tools.len();
        UI::info(&format!(
            "\n📊 Summary: {}/{} tools installed",
            installed_count, total_supported
        ));
    }

    Ok(())
}
