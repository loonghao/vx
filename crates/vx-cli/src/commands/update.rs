//! Update command implementation

use crate::ui::UI;
use vx_core::{PluginRegistry, Result, VxError};

pub async fn handle(registry: &PluginRegistry, tool_name: Option<&str>, apply: bool) -> Result<()> {
    match tool_name {
        Some(tool_name) => {
            // Update specific tool
            update_single_tool(registry, tool_name, apply).await
        }
        None => {
            // Update all tools
            update_all_tools(registry, apply).await
        }
    }
}

async fn update_single_tool(registry: &PluginRegistry, tool_name: &str, apply: bool) -> Result<()> {
    let tool = registry
        .get_tool(tool_name)
        .ok_or_else(|| VxError::ToolNotFound {
            tool_name: tool_name.to_string(),
        })?;

    // Get installed versions
    let installed_versions = tool.get_installed_versions().await?;
    if installed_versions.is_empty() {
        UI::warn(&format!("{} is not installed", tool_name));
        UI::hint(&format!("Use 'vx install {}' to install it", tool_name));
        return Ok(());
    }

    // Get latest version
    UI::info(&format!("Checking for updates to {}...", tool_name));
    let versions = tool.fetch_versions(false).await?;
    if versions.is_empty() {
        UI::warn(&format!("No versions found for {}", tool_name));
        return Ok(());
    }

    let latest_version = &versions[0].version;
    let current_version = &installed_versions[0]; // Most recent installed

    if current_version == latest_version {
        UI::success(&format!(
            "{} {} is already up to date",
            tool_name, current_version
        ));
        return Ok(());
    }

    UI::info(&format!(
        "Update available: {} {} → {}",
        tool_name, current_version, latest_version
    ));

    if !apply {
        UI::hint("Use --apply to perform the update");
        return Ok(());
    }

    // Perform the update
    UI::info(&format!("Updating {} to {}...", tool_name, latest_version));

    match tool.install_version(latest_version, false).await {
        Ok(()) => {
            UI::success(&format!(
                "Successfully updated {} to {}",
                tool_name, latest_version
            ));
        }
        Err(e) => {
            UI::error(&format!("Failed to update {}: {}", tool_name, e));
            return Err(e);
        }
    }

    Ok(())
}

async fn update_all_tools(registry: &PluginRegistry, apply: bool) -> Result<()> {
    let tools = registry.get_all_tools();

    if tools.is_empty() {
        UI::warn("No tools available");
        return Ok(());
    }

    UI::info("Checking for updates to all tools...");

    let mut updates_available = Vec::new();
    let mut errors = Vec::new();

    for tool in tools {
        match check_tool_update(&*tool).await {
            Ok(Some((current, latest))) => {
                updates_available.push((tool.name().to_string(), current, latest));
            }
            Ok(None) => {
                // No update needed or not installed
            }
            Err(e) => {
                UI::warn(&format!(
                    "Failed to check updates for {}: {}",
                    tool.name(),
                    e
                ));
                errors.push((tool.name().to_string(), e));
            }
        }
    }

    if updates_available.is_empty() {
        UI::success("All tools are up to date");
        return Ok(());
    }

    UI::info(&format!(
        "Found {} updates available:",
        updates_available.len()
    ));
    for (tool_name, current, latest) in &updates_available {
        UI::item(&format!("{}: {} → {}", tool_name, current, latest));
    }

    if !apply {
        UI::hint("Use --apply to perform all updates");
        return Ok(());
    }

    // Perform all updates
    UI::info("Updating all tools...");

    for (tool_name, _current, latest) in updates_available {
        if let Some(tool) = registry.get_tool(&tool_name) {
            match tool.install_version(&latest, false).await {
                Ok(()) => {
                    UI::success(&format!("Updated {} to {}", tool_name, latest));
                }
                Err(e) => {
                    UI::error(&format!("Failed to update {}: {}", tool_name, e));
                    errors.push((tool_name, e));
                }
            }
        }
    }

    if errors.is_empty() {
        UI::success("All tools updated successfully");
    } else {
        UI::warn(&format!("Some updates failed ({} errors)", errors.len()));
    }

    Ok(())
}

async fn check_tool_update(tool: &dyn vx_core::VxTool) -> Result<Option<(String, String)>> {
    // Get installed versions
    let installed_versions = tool.get_installed_versions().await?;
    if installed_versions.is_empty() {
        return Ok(None); // Not installed
    }

    // Get latest version
    let versions = tool.fetch_versions(false).await?;
    if versions.is_empty() {
        return Ok(None); // No versions available
    }

    let current_version = &installed_versions[0];
    let latest_version = &versions[0].version;

    if current_version != latest_version {
        Ok(Some((current_version.clone(), latest_version.clone())))
    } else {
        Ok(None) // Up to date
    }
}
