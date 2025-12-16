//! Which command implementation - Find vx-managed tools

use crate::ui::UI;
use anyhow::Result;
use vx_paths::{PathManager, PathResolver};
use vx_runtime::ProviderRegistry;

pub async fn handle(
    _registry: &ProviderRegistry,
    tool: &str,
    all: bool,
    use_system_path: bool,
) -> Result<()> {
    UI::debug(&format!("Looking for tool: {}", tool));

    // If --use-system-path is specified, only check system PATH
    if use_system_path {
        match which::which(tool) {
            Ok(path) => {
                println!("{}", path.display());
                return Ok(());
            }
            Err(_) => {
                UI::error(&format!("Tool '{}' not found in system PATH", tool));
                std::process::exit(1);
            }
        }
    }

    // Create path manager and resolver
    let path_manager = PathManager::new()
        .map_err(|e| anyhow::anyhow!("Failed to initialize path manager: {}", e))?;
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
        // Not found in vx-managed installations, check system PATH as fallback
        match which::which(tool) {
            Ok(path) => {
                // Found in system PATH
                println!("{} (system)", path.display());
                return Ok(());
            }
            Err(_) => {
                // Not found anywhere
                UI::error(&format!(
                    "Tool '{}' not found in vx-managed installations or system PATH",
                    tool
                ));
                UI::hint("Use 'vx list' to see installed tools");
                UI::hint(&format!("Use 'vx install {}' to install this tool", tool));
                std::process::exit(1);
            }
        }
    }

    for location in locations {
        println!("{}", location.display());
    }

    Ok(())
}
