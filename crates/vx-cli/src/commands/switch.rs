//! Switch command implementation

use crate::ui::UI;
use anyhow::Result;
use vx_paths::{PathManager, ShimManager};
use vx_plugin::{DependencyResolver, PluginRegistry};

pub async fn handle(registry: &PluginRegistry, tool_version: &str, global: bool) -> Result<()> {
    // Parse tool@version format
    let (tool_name, version) = parse_tool_version(tool_version)?;

    UI::info(&format!("Switching {} to version {}", tool_name, version));

    // Create path manager and shim manager
    let path_manager = PathManager::new()
        .map_err(|e| anyhow::anyhow!("Failed to initialize path manager: {}", e))?;
    let shim_manager = ShimManager::new(path_manager);

    // Check if the target version is installed
    if !shim_manager.is_version_installed(&tool_name, &version) {
        UI::info(&format!(
            "Version {} of {} is not installed. Installing...",
            version, tool_name
        ));

        // Try to find the tool in the registry and install it
        if let Some(tool) = registry.get_tool(&tool_name) {
            match tool.install_version(&version, false).await {
                Ok(()) => {
                    UI::success(&format!(
                        "Successfully installed {} version {}",
                        tool_name, version
                    ));
                }
                Err(e) => {
                    UI::error(&format!(
                        "Failed to install {} version {}: {}",
                        tool_name, version, e
                    ));
                    UI::hint(&format!(
                        "You can try installing manually with 'vx install {}@{}'",
                        tool_name, version
                    ));
                    return Err(anyhow::anyhow!("Auto-install failed: {}", e));
                }
            }
        } else {
            UI::error(&format!("Tool '{}' not found in registry", tool_name));
            UI::hint("Use 'vx list' to see available tools");
            return Err(anyhow::anyhow!("Tool not found"));
        }
    }

    // Switch to the new version
    match shim_manager.switch_tool_version(&tool_name, &version).await {
        Ok(()) => {
            UI::success(&format!(
                "Successfully switched {} to version {}{}",
                tool_name,
                version,
                if global { " (global)" } else { "" }
            ));

            // Update dependent tools' shims
            update_dependent_tools_shims(&tool_name, &version, &shim_manager).await?;

            // Show current version
            if let Ok(Some(current)) = shim_manager.get_current_version(&tool_name) {
                UI::info(&format!("Current version: {}", current));
            }
        }
        Err(e) => {
            UI::error(&format!(
                "Failed to switch {} to version {}: {}",
                tool_name, version, e
            ));
            return Err(e);
        }
    }

    Ok(())
}

/// Update dependent tools' shims when parent tool version changes
async fn update_dependent_tools_shims(
    parent_tool: &str,
    new_version: &str,
    shim_manager: &ShimManager,
) -> Result<()> {
    let dependency_resolver = DependencyResolver::new();
    let dependent_tools = dependency_resolver.get_dependent_tools(parent_tool);

    if dependent_tools.is_empty() {
        return Ok(());
    }

    UI::info(&format!(
        "Updating dependent tools for {}: {:?}",
        parent_tool, dependent_tools
    ));

    for dependent_tool in dependent_tools {
        // Check if this dependent tool has a shim
        UI::info(&format!(
            "Checking shim for dependent tool: {}",
            dependent_tool
        ));
        match shim_manager.get_current_version(dependent_tool) {
            Ok(Some(current_version)) => {
                UI::info(&format!(
                    "Found {} shim with version: {}",
                    dependent_tool, current_version
                ));

                // Get the new executable path for the dependent tool
                let path_manager = PathManager::new()
                    .map_err(|e| anyhow::anyhow!("Failed to initialize path manager: {}", e))?;

                let new_executable_path = if let Some(relative_path) =
                    dependency_resolver.get_executable_path(dependent_tool)
                {
                    // Use the specific executable path for this dependent tool
                    let parent_install_dir =
                        path_manager.tool_version_dir(parent_tool, new_version);
                    parent_install_dir.join(relative_path)
                } else {
                    // Fallback to the parent tool's executable
                    path_manager.tool_executable_path(parent_tool, new_version)
                };

                // Update the dependent tool's shim
                match shim_manager
                    .create_tool_shim(dependent_tool, &new_executable_path, new_version)
                    .await
                {
                    Ok(()) => {
                        UI::success(&format!(
                            "Updated {} shim to point to {} version {}",
                            dependent_tool, parent_tool, new_version
                        ));
                    }
                    Err(e) => {
                        UI::warn(&format!("Failed to update {} shim: {}", dependent_tool, e));
                    }
                }
            }
            Ok(None) => {
                UI::info(&format!(
                    "No shim found for dependent tool: {}",
                    dependent_tool
                ));
            }
            Err(e) => {
                UI::info(&format!(
                    "Error checking shim for {}: {}",
                    dependent_tool, e
                ));
            }
        }
    }

    Ok(())
}

/// Parse tool@version format
pub fn parse_tool_version(tool_version: &str) -> Result<(String, String)> {
    if let Some((tool, version)) = tool_version.split_once('@') {
        if tool.is_empty() || version.is_empty() {
            return Err(anyhow::anyhow!(
                "Invalid tool@version format: {}",
                tool_version
            ));
        }
        Ok((tool.to_string(), version.to_string()))
    } else {
        Err(anyhow::anyhow!(
            "Invalid format: {}. Expected format: tool@version (e.g., node@20.10.0)",
            tool_version
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_tool_version() {
        // Valid cases
        assert_eq!(
            parse_tool_version("node@20.10.0").unwrap(),
            ("node".to_string(), "20.10.0".to_string())
        );
        assert_eq!(
            parse_tool_version("python@3.11.0").unwrap(),
            ("python".to_string(), "3.11.0".to_string())
        );

        // Invalid cases
        assert!(parse_tool_version("node").is_err());
        assert!(parse_tool_version("@20.10.0").is_err());
        assert!(parse_tool_version("node@").is_err());
        assert!(parse_tool_version("").is_err());
    }
}
