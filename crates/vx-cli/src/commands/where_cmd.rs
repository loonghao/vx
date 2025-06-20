//! Which command implementation - Find vx-managed tools

use crate::commands::execute::create_dependent_tool_shim;
use crate::ui::UI;
use anyhow::Result;
use shimexe_core::ShimConfig;
use vx_paths::{PathManager, PathResolver, ShimManager};
use vx_plugin::{DependencyResolver, PluginRegistry};

pub async fn handle(
    registry: &PluginRegistry,
    tool: &str,
    all: bool,
    use_system_path: bool,
) -> Result<()> {
    if use_system_path {
        UI::debug(&format!("Looking for system tool: {}", tool));

        // Use system PATH to find the tool
        match which::which(tool) {
            Ok(path) => {
                println!("{}", path.display());
                return Ok(());
            }
            Err(_) => {
                UI::error(&format!("Tool '{}' not found in system PATH", tool));
                UI::hint("Make sure the tool is installed and available in your PATH");
                std::process::exit(1);
            }
        }
    }

    UI::debug(&format!("Looking for vx-managed tool: {}", tool));

    // Create path manager and shim manager
    let path_manager = PathManager::new()
        .map_err(|e| anyhow::anyhow!("Failed to initialize path manager: {}", e))?;
    let shim_manager = ShimManager::new(
        PathManager::new()
            .map_err(|e| anyhow::anyhow!("Failed to initialize path manager: {}", e))?,
    );

    // First, check if there's a shim for this tool
    if let Ok(Some(version)) = shim_manager.get_current_version(tool) {
        // Get the shim config path to read the actual executable path
        let config_path = path_manager.tool_current_shim_config_path(tool);

        if config_path.exists() {
            // Read the shim config to get the actual executable path
            match ShimConfig::from_file(&config_path) {
                Ok(shim_config) => {
                    match shim_config.get_executable_path() {
                        Ok(executable_path) => {
                            if executable_path.exists() {
                                // Show the path with version info
                                println!("{} ({}@{})", executable_path.display(), tool, version);
                                return Ok(());
                            } else {
                                UI::warn(&format!(
                                    "Shim points to non-existent path: {}",
                                    executable_path.display()
                                ));
                            }
                        }
                        Err(e) => {
                            UI::warn(&format!("Failed to read shim config: {}", e));
                        }
                    }
                }
                Err(e) => {
                    UI::warn(&format!("Failed to parse shim config: {}", e));
                }
            }
        }
    }

    // Fallback: use the old method
    let resolver = PathResolver::new(path_manager);
    let locations = if all {
        // Find all versions
        resolver.find_tool_executables(tool)?
    } else {
        // Find only the latest version
        match resolver.find_latest_executable(tool)? {
            Some(path) => vec![path],
            None => vec![],
        }
    };

    if locations.is_empty() {
        // Check if this is a dependent tool that can be auto-installed
        let dependency_resolver = DependencyResolver::new();
        let (actual_tool, dependent_tool) = dependency_resolver.resolve_dependency(tool);

        if let Some(dependent) = dependent_tool {
            // This is a dependent tool, try to auto-install the parent
            UI::info(&format!(
                "Tool '{}' requires '{}' which is not installed. Installing {}...",
                dependent, actual_tool, actual_tool
            ));

            if let Some(parent_tool) = registry.get_tool(&actual_tool) {
                // Check if parent tool is already installed
                let versions = parent_tool
                    .get_installed_versions()
                    .await
                    .unwrap_or_default();

                if versions.is_empty() {
                    // Parent tool not installed, install it
                    match parent_tool.install_version("latest", false).await {
                        Ok(()) => {
                            UI::success(&format!("Successfully installed {}", actual_tool));
                        }
                        Err(e) => {
                            UI::error(&format!("Failed to auto-install {}: {}", actual_tool, e));
                            // Continue to try creating shim anyway
                        }
                    }
                } else {
                    UI::info(&format!("{} is already installed", actual_tool));
                }

                // Create shim for the dependent tool
                let path_manager = PathManager::new()
                    .map_err(|e| anyhow::anyhow!("Failed to initialize path manager: {}", e))?;
                let shim_manager =
                    ShimManager::new(PathManager::new().map_err(|e| {
                        anyhow::anyhow!("Failed to initialize path manager: {}", e)
                    })?);

                // Create dependent tool shim
                if let Err(e) =
                    create_dependent_tool_shim(&dependent, &actual_tool, &path_manager).await
                {
                    UI::warn(&format!("Failed to create shim for {}: {}", dependent, e));
                } else {
                    UI::success(&format!("Created shim for {}", dependent));
                }

                // Now try to find the tool again
                if let Ok(Some(current_version)) = shim_manager.get_current_version(tool) {
                    let config_path = path_manager.tool_current_shim_config_path(tool);
                    if config_path.exists() {
                        if let Ok(shim_config) = ShimConfig::from_file(&config_path) {
                            if let Ok(executable_path) = shim_config.get_executable_path() {
                                if executable_path.exists() {
                                    println!(
                                        "{} ({}@{})",
                                        executable_path.display(),
                                        tool,
                                        current_version
                                    );
                                    return Ok(());
                                }
                            }
                        }
                    }
                }
            }
        }

        UI::error(&format!(
            "Tool '{}' not found in vx-managed installations",
            tool
        ));
        UI::hint("Use 'vx list' to see installed tools");
        UI::hint(&format!("Use 'vx install {}' to install this tool", tool));
        std::process::exit(1);
    }

    for location in locations {
        println!("{}", location.display());
    }

    Ok(())
}
