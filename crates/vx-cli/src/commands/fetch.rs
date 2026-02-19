//! Fetch command implementation (RFC 0031, RFC 0035: unified structured output)

use crate::cli::OutputFormat;
use crate::output::{OutputRenderer, VersionEntry, VersionsOutput};
use crate::ui::{ProgressSpinner, UI};
use anyhow::Result;
use vx_paths::{PathManager, PathResolver};
use vx_runtime::{ProviderRegistry, RuntimeContext};

/// Handle the fetch command
#[allow(clippy::too_many_arguments)]
pub async fn handle(
    registry: &ProviderRegistry,
    context: &RuntimeContext,
    tool_name: &str,
    latest: Option<usize>,
    include_prerelease: bool,
    _detailed: bool,
    interactive: bool,
    format: OutputFormat,
) -> Result<()> {
    let runtime = match registry.get_runtime(tool_name) {
        Some(r) => r,
        None => {
            // Show friendly error with suggestions
            let available_tools = registry.runtime_names();
            UI::tool_not_found(tool_name, &available_tools);
            return Err(anyhow::anyhow!("Tool not found: {}", tool_name));
        }
    };

    let spinner = ProgressSpinner::new(&format!("Fetching versions for {}...", tool_name));
    let mut versions = runtime.fetch_versions(context).await?;
    spinner.finish_and_clear();

    // Filter out prereleases if not requested
    if !include_prerelease {
        versions.retain(|v| !v.prerelease);
    }

    if versions.is_empty() {
        let output = VersionsOutput {
            tool: tool_name.to_string(),
            versions: vec![],
            total: 0,
            latest: None,
            lts: None,
        };

        let renderer = OutputRenderer::new(format);
        renderer.render(&output)?;
        return Ok(());
    }

    // Limit versions if requested
    if let Some(limit) = latest {
        versions.truncate(limit);
    }

    // Get installed versions for marking
    let installed_versions = get_installed_versions(tool_name)?;

    // Find latest and LTS versions
    let latest_version = versions.first().map(|v| v.version.clone());
    let lts_version = versions.iter().find(|v| v.lts).map(|v| v.version.clone());

    // Build version entries
    let version_entries: Vec<VersionEntry> = versions
        .iter()
        .map(|v| VersionEntry {
            version: v.version.clone(),
            installed: installed_versions.contains(&v.version),
            lts: v.lts,
            lts_name: v.metadata.get("lts_name").cloned(),
            date: v.released_at.as_ref().map(|dt| dt.to_rfc3339()),
            prerelease: v.prerelease,
            download_url: v.download_url.clone(),
        })
        .collect();

    let output = VersionsOutput {
        tool: tool_name.to_string(),
        versions: version_entries,
        total: versions.len(),
        latest: latest_version,
        lts: lts_version,
    };

    let renderer = OutputRenderer::new(format);

    if renderer.is_json() {
        renderer.render(&output)?;
    } else {
        // Text mode: use UI for header, then render results
        UI::success(&format!("Found {} versions:", versions.len()));
        renderer.render(&output)?;

        if interactive {
            UI::hint("Interactive version selection not yet implemented");
            UI::hint(&format!("Use: vx install {}@<version>", tool_name));
        }
    }

    Ok(())
}

/// Get installed versions for a tool
fn get_installed_versions(tool_name: &str) -> Result<Vec<String>> {
    let path_manager = PathManager::new()?;
    let resolver = PathResolver::new(path_manager);

    // Try to find installed executables and extract versions
    let executables = resolver.find_tool_executables(tool_name)?;
    let versions: Vec<String> = executables
        .iter()
        .map(|path| extract_version_from_path(path))
        .collect();

    Ok(versions)
}

/// Extract version from executable path
fn extract_version_from_path(path: &std::path::Path) -> String {
    for ancestor in path.ancestors() {
        if let Some(name) = ancestor.file_name().and_then(|n| n.to_str())
            && name.chars().any(|c| c.is_ascii_digit())
            && (name.contains('.') || name.chars().all(|c| c.is_ascii_digit()))
            && !name.contains('-')
        {
            return name.to_string();
        }
    }
    "unknown".to_string()
}
