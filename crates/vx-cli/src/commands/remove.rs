//! Remove command implementation

use crate::ui::UI;
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

    if let Some(target_version) = version {
        // Remove specific version
        UI::info(&format!("Removing {} {}...", tool_name, target_version));

        // Run pre-uninstall hook
        runtime.pre_uninstall(target_version, context).await?;

        match runtime.uninstall(target_version, context).await {
            Ok(()) => {
                // Run post-uninstall hook
                runtime.post_uninstall(target_version, context).await?;

                UI::success(&format!(
                    "Successfully removed {} {}",
                    tool_name, target_version
                ));
            }
            Err(e) => {
                UI::error(&format!(
                    "Failed to remove {} {}: {}",
                    tool_name, target_version, e
                ));
                return Err(e);
            }
        }
    } else {
        // Remove all versions
        let installed_versions = runtime.installed_versions(context).await?;

        if installed_versions.is_empty() {
            UI::warn(&format!("No versions of {} are installed", tool_name));
            return Ok(());
        }

        if !force {
            UI::warn(&format!(
                "This will remove all {} versions: {}",
                tool_name,
                installed_versions.join(", ")
            ));
            UI::hint("Use --force to confirm removal of all versions");
            return Ok(());
        }

        UI::info(&format!("Removing all {} versions...", tool_name));

        let mut errors = Vec::new();
        for version in &installed_versions {
            // Run pre-uninstall hook
            if let Err(e) = runtime.pre_uninstall(version, context).await {
                UI::error(&format!(
                    "Pre-uninstall hook failed for {} {}: {}",
                    tool_name, version, e
                ));
                errors.push(e);
                continue;
            }

            match runtime.uninstall(version, context).await {
                Ok(()) => {
                    // Run post-uninstall hook (best effort)
                    if let Err(e) = runtime.post_uninstall(version, context).await {
                        UI::warn(&format!(
                            "Post-uninstall hook failed for {} {}: {}",
                            tool_name, version, e
                        ));
                    }
                    UI::detail(&format!("Removed {} {}", tool_name, version));
                }
                Err(e) => {
                    UI::error(&format!(
                        "Failed to remove {} {}: {}",
                        tool_name, version, e
                    ));
                    errors.push(e);
                }
            }
        }

        if errors.is_empty() {
            UI::success(&format!("Successfully removed all {} versions", tool_name));
        } else {
            UI::warn(&format!(
                "Removed some versions, but {} errors occurred",
                errors.len()
            ));
        }
    }

    Ok(())
}
