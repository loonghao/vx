//! Remove command implementation

use crate::ui::UI;
use anyhow::Result;
use vx_resolver::{VersionConstraint, VersionRequest};
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

    if let Some(requested_version) = version {
        // Get installed versions first
        let installed_versions = runtime.installed_versions(context).await?;

        if installed_versions.is_empty() {
            UI::warn(&format!("No versions of {} are installed", tool_name));
            return Ok(());
        }

        // Resolve version from installed versions (supports partial versions like "3.7" -> "3.7.13")
        let target_version =
            resolve_version_from_installed(tool_name, requested_version, &installed_versions)?;

        if requested_version != target_version {
            UI::detail(&format!(
                "Resolved {} → {}",
                requested_version, target_version
            ));
        }

        // Remove specific version
        UI::info(&format!("Removing {} {}...", tool_name, target_version));

        // Run pre-uninstall hook
        runtime.pre_uninstall(&target_version, context).await?;

        match runtime.uninstall(&target_version, context).await {
            Ok(()) => {
                // Run post-uninstall hook
                runtime.post_uninstall(&target_version, context).await?;

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

/// Resolve a version request against installed versions
///
/// Supports:
/// - Exact version: "3.7.13" -> "3.7.13"
/// - Partial version: "3.7" -> "3.7.13" (latest matching 3.7.x)
/// - Major version: "3" -> "3.12.0" (latest matching 3.x.x)
fn resolve_version_from_installed(
    tool_name: &str,
    requested: &str,
    installed: &[String],
) -> Result<String> {
    // Parse the request to understand what kind of constraint it is
    let request = VersionRequest::parse(requested);

    // Convert installed versions to parsed versions for matching
    let mut matching: Vec<_> = installed
        .iter()
        .filter_map(|v| {
            let parsed = vx_resolver::Version::parse(v)?;
            if matches_constraint(&parsed, &request.constraint) {
                Some((parsed, v.clone()))
            } else {
                None
            }
        })
        .collect();

    if matching.is_empty() {
        let installed_str = if installed.is_empty() {
            "none".to_string()
        } else {
            installed.join(", ")
        };

        return Err(anyhow::anyhow!(
            "No installed version matches '{}'. Installed versions: {}\n\nTip: Use 'vx versions {}' to see available versions, or 'vx install {}@{}' to install it.",
            requested,
            installed_str,
            tool_name,
            tool_name,
            requested
        ));
    }

    // Sort by version (descending) and return the highest matching version
    matching.sort_by(|(a, _), (b, _)| b.cmp(a));

    Ok(matching.first().unwrap().1.clone())
}

/// Check if a version matches a constraint
fn matches_constraint(version: &vx_resolver::Version, constraint: &VersionConstraint) -> bool {
    match constraint {
        VersionConstraint::Exact(v) => version == v,
        VersionConstraint::Partial { major, minor } => {
            version.major == *major && version.minor == *minor
        }
        VersionConstraint::Major(major) => version.major == *major,
        VersionConstraint::Latest
        | VersionConstraint::LatestPrerelease
        | VersionConstraint::Any => true,
        VersionConstraint::Wildcard { major, minor } => {
            version.major == *major && version.minor == *minor
        }
        VersionConstraint::Caret(v) => {
            // ^1.2.3 means >=1.2.3, <2.0.0 (for major > 0)
            // ^0.2.3 means >=0.2.3, <0.3.0 (for major == 0, minor > 0)
            // ^0.0.3 means >=0.0.3, <0.0.4 (for major == 0, minor == 0)
            if v.major > 0 {
                version.major == v.major && version >= v
            } else if v.minor > 0 {
                version.major == 0 && version.minor == v.minor && version >= v
            } else {
                version.major == 0 && version.minor == 0 && version.patch == v.patch
            }
        }
        VersionConstraint::Tilde(v) => {
            // ~1.2.3 means >=1.2.3, <1.3.0
            version.major == v.major && version.minor == v.minor && version >= v
        }
        VersionConstraint::Range(constraints) => constraints.iter().all(|c| c.satisfies(version)),
    }
}
