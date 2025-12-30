//! Install command implementation

use crate::ui::{ProgressSpinner, UI};
use anyhow::Result;
use vx_runtime::{ProviderRegistry, RuntimeContext};

pub async fn handle(
    registry: &ProviderRegistry,
    context: &RuntimeContext,
    tool_name: &str,
    version: Option<&str>,
    force: bool,
) -> Result<()> {
    // Get the runtime from registry
    let runtime = match registry.get_runtime(tool_name) {
        Some(r) => r,
        None => {
            // Show friendly error with suggestions
            let available_tools = registry.runtime_names();
            UI::tool_not_found(tool_name, &available_tools);
            return Err(anyhow::anyhow!("Tool not found: {}", tool_name));
        }
    };

    // Determine version to install
    let requested_version = version.unwrap_or("latest");

    // Resolve version (handles "latest", partial versions like "3.11", etc.)
    let spinner = ProgressSpinner::new(&format!(
        "Resolving version {} for {}...",
        requested_version, tool_name
    ));
    let target_version = runtime.resolve_version(requested_version, context).await?;
    spinner.finish_and_clear();

    if requested_version != target_version {
        UI::detail(&format!(
            "Resolved {} → {}",
            requested_version, target_version
        ));
    }

    // Check if already installed
    if !force && runtime.is_installed(&target_version, context).await? {
        UI::success(&format!(
            "{} {} is already installed",
            tool_name, target_version
        ));
        UI::hint("Use --force to reinstall");
        return Ok(());
    }

    // Run pre-install hook
    runtime.pre_install(&target_version, context).await?;

    // Install the version with progress spinner
    let spinner =
        ProgressSpinner::new_install(&format!("Installing {} {}...", tool_name, target_version));
    let install_result = runtime.install(&target_version, context).await;

    match install_result {
        Ok(result) => {
            spinner.finish_with_message(&format!(
                "✓ Successfully installed {} {}",
                tool_name, target_version
            ));

            // Run post-install hook
            runtime.post_install(&target_version, context).await?;

            // Show installation path
            UI::detail(&format!("Installed to: {}", result.install_path.display()));

            // Show usage hint
            UI::hint(&format!(
                "Use 'vx {} --version' to verify installation",
                tool_name
            ));
        }
        Err(e) => {
            spinner.finish_with_error(&format!(
                "Failed to install {} {}: {}",
                tool_name, target_version, e
            ));
            return Err(e);
        }
    }

    Ok(())
}
