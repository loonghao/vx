//! Install command implementation

use crate::ui::UI;
use anyhow::Result;
use tracing::{info_span, Instrument};
use vx_plugin::PluginRegistry;

pub async fn handle(
    registry: &PluginRegistry,
    tool_name: &str,
    version: Option<&str>,
    force: bool,
) -> Result<()> {
    // Parse tool@version format if present
    let (actual_tool_name, parsed_version) = if tool_name.contains('@') {
        let parts: Vec<&str> = tool_name.splitn(2, '@').collect();
        if parts.len() == 2 {
            (parts[0], Some(parts[1]))
        } else {
            (tool_name, version)
        }
    } else {
        (tool_name, version)
    };

    // Get the tool from registry
    let tool = registry
        .get_tool(actual_tool_name)
        .ok_or_else(|| anyhow::anyhow!("Tool not found: {}", actual_tool_name))?;

    // Determine version to install (prefer parsed version over explicit version)
    let target_version = if let Some(v) = parsed_version.or(version) {
        v.to_string()
    } else {
        // Get latest version with progress span
        let span = info_span!("Fetching latest version", tool = actual_tool_name);
        let versions = async {
            UI::info(&format!(
                "Fetching latest version for {}...",
                actual_tool_name
            ));
            tool.fetch_versions(false).await
        }
        .instrument(span)
        .await?;

        if versions.is_empty() {
            return Err(anyhow::anyhow!(
                "No versions found for tool: {}",
                actual_tool_name
            ));
        }
        versions[0].version.clone()
    };

    UI::info(&format!(
        "Installing {} {}...",
        actual_tool_name, target_version
    ));

    // Check if already installed
    if !force && tool.is_version_installed(&target_version).await? {
        UI::success(&format!(
            "{} {} is already installed",
            actual_tool_name, target_version
        ));
        UI::hint("Use --force to reinstall");
        return Ok(());
    }

    // Install the version with progress span
    let install_span =
        info_span!("Installing tool", tool = actual_tool_name, version = %target_version);
    let install_result = async { tool.install_version(&target_version, force).await }
        .instrument(install_span)
        .await;

    match install_result {
        Ok(()) => {
            UI::success(&format!(
                "Successfully installed {} {}",
                actual_tool_name, target_version
            ));

            // Show installation path
            let install_dir = tool.get_version_install_dir(&target_version);
            UI::detail(&format!("Installed to: {}", install_dir.display()));

            // Show usage hint
            UI::hint(&format!(
                "Use 'vx {} --version' to verify installation",
                actual_tool_name
            ));
        }
        Err(e) => {
            UI::error(&format!(
                "Failed to install {} {}: {}",
                actual_tool_name, target_version, e
            ));
            return Err(e);
        }
    }

    Ok(())
}
