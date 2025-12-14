//! Which command implementation - Find vx-managed tools

use crate::ui::UI;
use anyhow::Result;
use vx_paths::{PathManager, PathResolver};
use vx_plugin::BundleRegistry;

pub async fn handle(_registry: &BundleRegistry, tool: &str, all: bool) -> Result<()> {
    UI::debug(&format!("Looking for vx-managed tool: {}", tool));

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
