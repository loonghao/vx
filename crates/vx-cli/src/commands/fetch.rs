//! Fetch command implementation

use crate::ui::{ProgressSpinner, UI};
use anyhow::Result;
use vx_runtime::{ProviderRegistry, RuntimeContext};

/// Handle the fetch command
pub async fn handle(
    registry: &ProviderRegistry,
    context: &RuntimeContext,
    tool_name: &str,
    latest: Option<usize>,
    include_prerelease: bool,
    detailed: bool,
    interactive: bool,
) -> Result<()> {
    let runtime = registry
        .get_runtime(tool_name)
        .ok_or_else(|| anyhow::anyhow!("Tool not found: {}", tool_name))?;

    let spinner = ProgressSpinner::new(&format!("Fetching versions for {}...", tool_name));
    let mut versions = runtime.fetch_versions(context).await?;
    spinner.finish_and_clear();

    // Filter out prereleases if not requested
    if !include_prerelease {
        versions.retain(|v| !v.prerelease);
    }

    if versions.is_empty() {
        UI::warn("No versions found");
        return Ok(());
    }

    // Limit versions if requested
    if let Some(limit) = latest {
        versions.truncate(limit);
    }

    UI::success(&format!("Found {} versions:", versions.len()));

    for (i, version) in versions.iter().enumerate() {
        let prerelease_marker = if version.prerelease {
            " (prerelease)"
        } else {
            ""
        };
        let lts_marker = if version.lts { " (LTS)" } else { "" };

        if detailed {
            UI::item(&format!(
                "{}. {}{}{}",
                i + 1,
                version.version,
                prerelease_marker,
                lts_marker
            ));

            if let Some(date) = &version.released_at {
                UI::detail(&format!("   Released: {}", date));
            }

            if let Some(url) = &version.download_url {
                UI::detail(&format!("   Download: {}", url));
            }
        } else {
            UI::item(&format!(
                "{}. {}{}{}",
                i + 1,
                version.version,
                prerelease_marker,
                lts_marker
            ));
        }

        // Limit output for non-detailed view
        if !detailed && i >= 19 {
            UI::detail(&format!("   ... and {} more versions", versions.len() - 20));
            UI::hint("Use --detailed to see all versions");
            break;
        }
    }

    if interactive {
        UI::hint("Interactive version selection not yet implemented");
        UI::hint(&format!("Use: vx install {}@<version>", tool_name));
    }

    Ok(())
}
