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
        UI::debug("Found vx-managed tool, checking if installed...");

        // Check if the tool is actually installed
        let installed_versions = tool.get_installed_versions().await.unwrap_or_default();
        if !installed_versions.is_empty() {
            UI::debug("Tool is installed, executing...");
            return execute_vx_tool(tool, args).await;
        } else {
            // Tool is supported but not installed - offer auto-installation
            return handle_auto_installation(tool, tool_name, args, use_system_path, registry)
                .await;
        }
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

    // Final fallback: suggest installation if tool is supported
    if registry.get_tool(tool_name).is_some() {
        UI::error(&format!(
            "Tool '{}' is supported but not available",
            tool_name
        ));
        UI::hint(&format!(
            "Try installing it manually with: vx install {}",
            tool_name
        ));
        UI::hint("Or use --use-system-path to use system-installed tools");
    } else {
        UI::error(&format!("Tool '{}' is not supported by vx", tool_name));
        UI::hint("Run 'vx list --all' to see supported tools");
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
        let versions = parent.get_installed_versions().await.unwrap_or_default();

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

            if let Err(e) =
                install_tool_with_dependencies(parent.as_ref(), "latest", registry).await
            {
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
                let full_path = parent_install_dir.join(relative_path);

                // Use our PATHEXT-aware search to find the actual executable
                if let Some(found_path) =
                    vx_paths::find_executable_with_extensions(&full_path, dependent_tool)
                {
                    found_path
                } else {
                    return Err(anyhow::anyhow!(
                        "Dependent tool '{}' not found in parent installation at: {}",
                        dependent_tool,
                        full_path.display()
                    ));
                }
            } else {
                return Err(anyhow::anyhow!(
                    "No installed versions found for parent tool: {}",
                    parent_tool
                ));
            }
        } else {
            // Fallback: try to find the executable using the standard path detection with PATHEXT support
            let parent_versions = path_manager.list_tool_versions(parent_tool)?;
            if let Some(latest_version) = parent_versions.first() {
                let base_path = path_manager.tool_executable_path(dependent_tool, latest_version);

                // Use our PATHEXT-aware search
                if let Some(found_path) =
                    vx_paths::find_executable_with_extensions(&base_path, dependent_tool)
                {
                    found_path
                } else {
                    return Err(anyhow::anyhow!(
                        "Dependent tool '{}' not found in parent installation",
                        dependent_tool
                    ));
                }
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
    // Create shim for dependent tool with proper argument handling
    // This ensures dependent tools (like npx, bunx) work correctly
    shim_manager
        .create_tool_shim(dependent_tool, executable_path, "latest")
        .await
        .map_err(|e| anyhow::anyhow!("Failed to create shim for {}: {}", dependent_tool, e))?;

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
    // Use our custom which function to find the tool
    match vx_paths::which_tool(tool_name, true)? {
        Some(tool_path) => {
            UI::debug(&format!("Found tool at: {}", tool_path.display()));
            let status = std::process::Command::new(&tool_path)
                .args(args)
                .status()
                .map_err(|e| anyhow::anyhow!("Failed to execute {}: {}", tool_name, e))?;

            Ok(status.code().unwrap_or(1))
        }
        None => Err(anyhow::anyhow!("Tool not found: {}", tool_name)),
    }
}

/// Handle auto-installation with improved user experience
async fn handle_auto_installation(
    tool: Box<dyn vx_plugin::VxTool>,
    tool_name: &str,
    args: &[String],
    use_system_path: bool,
    registry: &PluginRegistry,
) -> Result<i32> {
    UI::info(&format!(
        "Tool '{}' is supported but not installed.",
        tool_name
    ));

    // Try auto-installation first
    UI::info("Attempting automatic installation...");
    match install_tool_with_dependencies(tool.as_ref(), "latest", registry).await {
        Ok(_) => {
            UI::success(&format!("Successfully installed {}", tool_name));
            return execute_vx_tool(tool, args).await;
        }
        Err(e) => {
            UI::error(&format!("Auto-installation failed: {}", e));

            // Provide detailed error information
            UI::info("Installation failed for the following reason:");
            UI::info(&format!("  {}", e));

            // Offer alternatives based on use_system_path setting
            if use_system_path {
                UI::info("Checking if tool is available in system PATH...");
                match execute_system_tool(tool_name, args).await {
                    Ok(exit_code) => {
                        UI::success(&format!(
                            "Found and executed {} from system PATH",
                            tool_name
                        ));
                        return Ok(exit_code);
                    }
                    Err(_) => {
                        UI::warn(&format!("{} not found in system PATH either", tool_name));
                    }
                }
            }

            // Provide helpful suggestions
            UI::error(&format!("Unable to execute '{}'", tool_name));
            UI::hint("Possible solutions:");
            UI::hint(&format!(
                "  1. Try manual installation: vx install {}",
                tool_name
            ));
            UI::hint("  2. Check your internet connection and try again");
            if !use_system_path {
                UI::hint("  3. Use --use-system-path to try system-installed tools");
            }
            UI::hint(&format!(
                "  4. Check if {} is available in your system PATH",
                tool_name
            ));

            return Err(anyhow::anyhow!("Tool installation and execution failed"));
        }
    }
}

/// Install a tool with automatic dependency resolution
async fn install_tool_with_dependencies(
    tool: &dyn vx_plugin::VxTool,
    version: &str,
    registry: &PluginRegistry,
) -> Result<()> {
    UI::debug(&format!("Installing {} with dependencies...", tool.name()));

    // Get tool dependencies
    let dependencies = tool.get_dependencies();

    // Install dependencies first
    for dep in &dependencies {
        if dep.required {
            UI::info(&format!(
                "Installing required dependency: {}",
                dep.tool_name
            ));

            if let Some(dep_tool) = registry.get_tool(&dep.tool_name) {
                // Check if dependency is already installed
                let installed_versions =
                    dep_tool.get_installed_versions().await.unwrap_or_default();
                if installed_versions.is_empty() {
                    // Install the dependency
                    if let Err(e) = dep_tool.install_version("latest", false).await {
                        return Err(anyhow::anyhow!(
                            "Failed to install required dependency {}: {}",
                            dep.tool_name,
                            e
                        ));
                    }
                    UI::success(&format!(
                        "Successfully installed dependency: {}",
                        dep.tool_name
                    ));
                } else {
                    UI::debug(&format!(
                        "Dependency {} is already installed",
                        dep.tool_name
                    ));
                }
            } else {
                return Err(anyhow::anyhow!(
                    "Required dependency {} is not supported by vx",
                    dep.tool_name
                ));
            }
        }
    }

    // Install the main tool with better error context
    UI::info(&format!(
        "Installing {} version {}...",
        tool.name(),
        version
    ));
    match tool.install_version(version, false).await {
        Ok(_) => {
            UI::success(&format!("Successfully installed {} {}", tool.name(), version));
            UI::debug(&format!(
                "Successfully installed {} and its dependencies",
                tool.name()
            ));
            Ok(())
        }
        Err(e) => {
            Err(anyhow::anyhow!(
                "Failed to install {} {}: {}. This could be due to network issues, insufficient permissions, or incompatible system configuration.",
                tool.name(),
                version,
                e
            ))
        }
    }
}
