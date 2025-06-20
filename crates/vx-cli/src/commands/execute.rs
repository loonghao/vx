//! Execute command implementation - Transparent proxy for tool execution

use crate::ui::UI;
use anyhow::Result;
use std::collections::HashMap;
use vx_paths::{PathManager, ShimManager};
use vx_plugin::{DependencyResolver, PluginRegistry};

/// Handle the execute command
pub async fn handle(
    registry: &PluginRegistry,
    tool_name: &str,
    args: &[String],
    use_system_path: bool,
) -> Result<()> {
    let exit_code = execute_tool(registry, tool_name, args, use_system_path).await?;
    if exit_code != 0 {
        std::process::exit(exit_code);
    }
    Ok(())
}

/// Execute tool with given arguments with smart tool resolution
pub async fn execute_tool(
    registry: &PluginRegistry,
    tool_name: &str,
    args: &[String],
    use_system_path: bool,
) -> Result<i32> {
    UI::debug(&format!("Executing: {} {}", tool_name, args.join(" ")));

    // First, try to use shim if available
    UI::debug("Checking for shim configuration...");
    if let Ok(exit_code) = execute_shim_tool(tool_name, args).await {
        UI::debug("Executed via shim");
        return Ok(exit_code);
    }

    // Check if this is a dependent tool (e.g., npx -> node)
    let dependency_resolver = DependencyResolver::new();
    let (actual_tool, dependent_tool) = dependency_resolver.resolve_dependency(tool_name);

    // If this is a dependent tool, try to handle it specially
    if let Some(dependent) = dependent_tool {
        UI::debug(&format!(
            "Handling as dependent tool: {} -> {}",
            dependent, actual_tool
        ));
        return handle_dependent_tool(registry, &actual_tool, &dependent, args).await;
    }

    // Try to find the tool in vx-managed tools first
    UI::debug("Checking vx-managed tools...");
    if let Some(tool) = registry.get_tool(tool_name) {
        UI::debug("Found vx-managed tool, executing...");
        return execute_vx_tool(tool, args).await;
    }

    // If use_system_path is true, try system PATH
    if use_system_path {
        UI::debug("Trying system PATH...");
        if let Ok(exit_code) = execute_system_tool(tool_name, args).await {
            UI::debug("Executed via system PATH");
            return Ok(exit_code);
        }
    } else {
        UI::debug("System PATH disabled (use_system_path=false)");
    }

    // Tool not found, try to auto-install if supported
    if let Some(tool) = registry.get_tool(tool_name) {
        UI::info(&format!(
            "Tool '{}' not found, attempting to install...",
            tool_name
        ));

        // Try to install the latest version
        if let Err(e) = tool.install_version("latest", false).await {
            UI::warn(&format!("Failed to auto-install {}: {}", tool_name, e));
            return Err(anyhow::anyhow!(
                "Tool not found and auto-install failed: {}",
                tool_name
            ));
        }

        UI::success(&format!("Successfully installed {}", tool_name));
        return execute_vx_tool(tool, args).await;
    }

    Err(anyhow::anyhow!("Tool not found: {}", tool_name))
}

/// Execute a vx-managed tool
async fn execute_vx_tool(tool: Box<dyn vx_plugin::VxTool>, args: &[String]) -> Result<i32> {
    let context = vx_plugin::ToolContext {
        working_directory: std::env::current_dir().ok(),
        environment_variables: HashMap::new(),
        use_system_path: false,
        options: HashMap::new(),
    };

    let result = tool.execute(args, &context).await?;
    Ok(result.exit_code)
}

/// Handle execution of dependent tools (e.g., npx -> node)
async fn handle_dependent_tool(
    registry: &PluginRegistry,
    parent_tool: &str,
    dependent_tool: &str,
    args: &[String],
) -> Result<i32> {
    UI::debug(&format!(
        "Handling dependent tool: {} -> {}",
        dependent_tool, parent_tool
    ));

    // Check if parent tool is installed
    if let Some(parent) = registry.get_tool(parent_tool) {
        let path_manager = PathManager::new()
            .map_err(|e| anyhow::anyhow!("Failed to initialize path manager: {}", e))?;

        // Try to find an installed version of the parent tool
        let versions = parent.get_installed_versions().await?;

        // Also check if there's a shim for the parent tool
        let shim_manager = ShimManager::new(
            PathManager::new()
                .map_err(|e| anyhow::anyhow!("Failed to initialize path manager: {}", e))?,
        );
        let has_shim = shim_manager.get_current_version(parent_tool).is_ok();

        if versions.is_empty() && !has_shim {
            // Parent tool not installed, auto-install it
            UI::info(&format!(
                "Tool '{}' requires '{}' which is not installed. Installing {}...",
                dependent_tool, parent_tool, parent_tool
            ));

            if let Err(e) = parent.install_version("latest", false).await {
                UI::error(&format!("Failed to auto-install {}: {}", parent_tool, e));
                UI::hint(&format!(
                    "You can try installing manually with 'vx install {}'",
                    parent_tool
                ));
                return Err(anyhow::anyhow!(
                    "Tool '{}' requires '{}' but auto-install failed: {}",
                    dependent_tool,
                    parent_tool,
                    e
                ));
            }

            UI::success(&format!("Successfully installed {}", parent_tool));
        } else {
            UI::debug(&format!(
                "Parent tool '{}' is already installed with versions: {:?}",
                parent_tool, versions
            ));
        }

        // Now create a shim for the dependent tool
        create_dependent_tool_shim(dependent_tool, parent_tool, &path_manager).await?;

        // Try to execute using the newly created shim
        let shim_manager = ShimManager::new(path_manager);
        return shim_manager.execute_tool_shim(dependent_tool, args);
    }

    Err(anyhow::anyhow!(
        "Parent tool '{}' not found for dependent tool '{}'",
        parent_tool,
        dependent_tool
    ))
}

/// Create a shim for a dependent tool
pub async fn create_dependent_tool_shim(
    dependent_tool: &str,
    parent_tool: &str,
    path_manager: &PathManager,
) -> Result<()> {
    let dependency_resolver = DependencyResolver::new();

    // Get the executable path for the dependent tool within the parent installation
    let executable_path =
        if let Some(relative_path) = dependency_resolver.get_executable_path(dependent_tool) {
            // Find the latest installed version of the parent tool
            let parent_versions = path_manager.list_tool_versions(parent_tool)?;
            if let Some(latest_version) = parent_versions.first() {
                let parent_install_dir = path_manager.tool_version_dir(parent_tool, latest_version);
                parent_install_dir.join(relative_path)
            } else {
                return Err(anyhow::anyhow!(
                    "No installed versions found for parent tool: {}",
                    parent_tool
                ));
            }
        } else {
            // Fallback: try to find the executable using the standard path detection
            let parent_versions = path_manager.list_tool_versions(parent_tool)?;
            if let Some(latest_version) = parent_versions.first() {
                path_manager.tool_executable_path(dependent_tool, latest_version)
            } else {
                return Err(anyhow::anyhow!(
                    "No installed versions found for parent tool: {}",
                    parent_tool
                ));
            }
        };

    // Create shim for the dependent tool with special argument handling
    let new_path_manager = PathManager::new()
        .map_err(|e| anyhow::anyhow!("Failed to initialize path manager: {}", e))?;
    let shim_manager = ShimManager::new(new_path_manager);

    // Create shim with special argument handling for certain tools
    create_dependent_tool_shim_with_args(dependent_tool, &executable_path, &shim_manager).await?;

    UI::success(&format!(
        "Created shim for {} -> {}",
        dependent_tool,
        executable_path.display()
    ));

    Ok(())
}

/// Create a shim with special argument handling for dependent tools
async fn create_dependent_tool_shim_with_args(
    dependent_tool: &str,
    executable_path: &std::path::Path,
    shim_manager: &ShimManager,
) -> Result<()> {
    // For now, just use the standard shim creation without special args
    // TODO: Implement special argument handling when shimexe-core API is clarified
    shim_manager
        .create_tool_shim(dependent_tool, executable_path, "latest")
        .await?;

    Ok(())
}

/// Execute a tool using shim configuration
async fn execute_shim_tool(tool_name: &str, args: &[String]) -> Result<i32> {
    // Create path manager and shim manager
    let path_manager = PathManager::new()
        .map_err(|e| anyhow::anyhow!("Failed to initialize path manager: {}", e))?;
    let shim_manager = ShimManager::new(path_manager);

    // Try to execute using shim
    shim_manager.execute_tool_shim(tool_name, args)
}

/// Execute a tool using system PATH
async fn execute_system_tool(tool_name: &str, args: &[String]) -> Result<i32> {
    let status = std::process::Command::new(tool_name)
        .args(args)
        .status()
        .map_err(|_| anyhow::anyhow!("Tool not found: {}", tool_name))?;

    Ok(status.code().unwrap_or(1))
}
