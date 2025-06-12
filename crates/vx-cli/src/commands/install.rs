//! Install command implementation

use crate::ui::UI;
use vx_core::{PluginRegistry, Result, VxError};

pub async fn handle(
    registry: &PluginRegistry,
    tool_name: &str,
    version: Option<&str>,
    force: bool,
) -> Result<()> {
    // Get the tool from registry
    let tool = registry
        .get_tool(tool_name)
        .ok_or_else(|| VxError::ToolNotFound {
            tool_name: tool_name.to_string(),
        })?;

    // Determine version to install
    let target_version = if let Some(v) = version {
        v.to_string()
    } else {
        // Get latest version
        UI::info(&format!("Fetching latest version for {}...", tool_name));
        let versions = tool.fetch_versions(false).await?;
        if versions.is_empty() {
            return Err(VxError::VersionNotFound {
                tool_name: tool_name.to_string(),
                version: "latest".to_string(),
            });
        }
        versions[0].version.clone()
    };

    UI::info(&format!("Installing {} {}...", tool_name, target_version));

    // Check if already installed
    if !force && tool.is_version_installed(&target_version).await? {
        UI::success(&format!(
            "{} {} is already installed",
            tool_name, target_version
        ));
        UI::hint("Use --force to reinstall");
        return Ok(());
    }

    // Install the version
    match tool.install_version(&target_version, force).await {
        Ok(()) => {
            UI::success(&format!(
                "Successfully installed {} {}",
                tool_name, target_version
            ));

            // Show installation path
            let install_dir = tool.get_version_install_dir(&target_version);
            UI::detail(&format!("Installed to: {}", install_dir.display()));

            // Show usage hint
            UI::hint(&format!(
                "Use 'vx {} --version' to verify installation",
                tool_name
            ));
        }
        Err(e) => {
            UI::error(&format!(
                "Failed to install {} {}: {}",
                tool_name, target_version, e
            ));
            return Err(e);
        }
    }

    Ok(())
}
