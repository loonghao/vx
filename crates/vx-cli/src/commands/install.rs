//! Install command implementation

use crate::ui::UI;
use anyhow::Result;
use tracing::{info_span, Instrument};
use vx_runtime::{ProviderRegistry, RuntimeContext};

pub async fn handle(
    registry: &ProviderRegistry,
    context: &RuntimeContext,
    tool_name: &str,
    version: Option<&str>,
    force: bool,
) -> Result<()> {
    // Get the runtime from registry
    let runtime = registry
        .get_runtime(tool_name)
        .ok_or_else(|| anyhow::anyhow!("Tool not found: {}", tool_name))?;

    // Determine version to install
    let target_version = if let Some(v) = version {
        v.to_string()
    } else {
        // Get latest version with progress span
        let span = info_span!("Fetching latest version", tool = tool_name);
        let versions = async {
            UI::info(&format!("Fetching latest version for {}...", tool_name));
            runtime.fetch_versions(context).await
        }
        .instrument(span)
        .await?;

        if versions.is_empty() {
            return Err(anyhow::anyhow!("No versions found for tool: {}", tool_name));
        }

        // Find first non-prerelease version
        versions
            .iter()
            .find(|v| !v.prerelease)
            .map(|v| v.version.clone())
            .unwrap_or_else(|| versions[0].version.clone())
    };

    UI::info(&format!("Installing {} {}...", tool_name, target_version));

    // Check if already installed
    if !force && runtime.is_installed(&target_version, context).await? {
        UI::success(&format!(
            "{} {} is already installed",
            tool_name, target_version
        ));
        UI::hint("Use --force to reinstall");
        return Ok(());
    }

    // Install the version with progress span
    let install_span = info_span!("Installing tool", tool = tool_name, version = %target_version);
    let install_result = async { runtime.install(&target_version, context).await }
        .instrument(install_span)
        .await;

    match install_result {
        Ok(result) => {
            UI::success(&format!(
                "Successfully installed {} {}",
                tool_name, target_version
            ));

            // Show installation path
            UI::detail(&format!("Installed to: {}", result.install_path.display()));

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
