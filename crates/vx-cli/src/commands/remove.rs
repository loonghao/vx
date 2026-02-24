//! Remove command implementation
//!
//! Uses `ProviderHandle` as the primary source for installed version queries
//! and uninstall operations (RFC-0037). Falls back to the legacy `Runtime`
//! trait path when no ProviderHandle is registered for the tool.

use crate::ui::UI;
use anyhow::Result;
use vx_runtime::{ProviderRegistry, RuntimeContext};
use vx_starlark::handle::global_registry;

pub async fn handle(
    registry: &ProviderRegistry,
    context: &RuntimeContext,
    tool_name: &str,
    version: Option<&str>,
    force: bool,
) -> Result<()> {
    // ── Step 1: Try ProviderHandle (RFC-0037) ─────────────────────────────
    {
        let reg = global_registry().await;
        if let Some(handle) = reg.get(tool_name) {
            return handle_via_provider_handle(
                &handle, tool_name, version, force, registry, context,
            )
            .await;
        }
    }

    // ── Step 2: Fallback to legacy Runtime trait ──────────────────────────
    handle_via_runtime(registry, context, tool_name, version, force).await
}

// ---------------------------------------------------------------------------
// ProviderHandle path (RFC-0037)
// ---------------------------------------------------------------------------

async fn handle_via_provider_handle(
    handle: &vx_starlark::handle::ProviderHandle,
    tool_name: &str,
    version: Option<&str>,
    force: bool,
    registry: &ProviderRegistry,
    context: &RuntimeContext,
) -> Result<()> {
    let installed = handle.installed_versions();

    if installed.is_empty() {
        UI::warn(&format!("No versions of {} are installed", tool_name));
        return Ok(());
    }

    if let Some(requested) = version {
        // Resolve partial/exact version against installed list
        let target = handle
            .resolve_installed_version(requested)
            .map_err(|e| anyhow::anyhow!("{}", e))?;

        if requested != target {
            UI::detail(&format!("Resolved {} → {}", requested, target));
        }

        UI::info(&format!("Removing {} {}...", tool_name, target));

        // Run pre-uninstall hook via legacy runtime (hooks still live there)
        run_pre_uninstall_hook(registry, context, tool_name, &target).await;

        handle
            .uninstall(&target)
            .await
            .map_err(|e| anyhow::anyhow!("{}", e))?;

        // Invalidate caches
        invalidate_caches_for_runtime(tool_name, context);
        handle.invalidate_version_cache().await;

        // Run post-uninstall hook
        run_post_uninstall_hook(registry, context, tool_name, &target).await;

        UI::success(&format!("Successfully removed {} {}", tool_name, target));
    } else {
        // Remove all versions
        if !force {
            UI::warn(&format!(
                "This will remove all {} versions: {}",
                tool_name,
                installed.join(", ")
            ));
            UI::hint("Use --force to confirm removal of all versions");
            return Ok(());
        }

        UI::info(&format!("Removing all {} versions...", tool_name));

        let mut errors = 0usize;
        for ver in &installed {
            run_pre_uninstall_hook(registry, context, tool_name, ver).await;

            match handle
                .uninstall(ver)
                .await
                .map_err(|e| anyhow::anyhow!("{}", e))
            {
                Ok(()) => {
                    run_post_uninstall_hook(registry, context, tool_name, ver).await;
                    UI::detail(&format!("Removed {} {}", tool_name, ver));
                }
                Err(e) => {
                    UI::error(&format!("Failed to remove {} {}: {}", tool_name, ver, e));
                    errors += 1;
                }
            }
        }

        // Invalidate caches once after all removals
        invalidate_caches_for_runtime(tool_name, context);
        handle.invalidate_version_cache().await;

        if errors == 0 {
            UI::success(&format!("Successfully removed all {} versions", tool_name));
        } else {
            UI::warn(&format!(
                "Removed some versions, but {} error(s) occurred",
                errors
            ));
        }
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Legacy Runtime trait path (fallback)
// ---------------------------------------------------------------------------

async fn handle_via_runtime(
    registry: &ProviderRegistry,
    context: &RuntimeContext,
    tool_name: &str,
    version: Option<&str>,
    force: bool,
) -> Result<()> {
    let runtime = match registry.get_runtime(tool_name) {
        Some(r) => r,
        None => {
            let available_tools = registry.runtime_names();
            UI::tool_not_found(tool_name, &available_tools);
            return Err(anyhow::anyhow!("Tool not found: {}", tool_name));
        }
    };

    let installed_versions = runtime.installed_versions(context).await?;

    if installed_versions.is_empty() {
        UI::warn(&format!("No versions of {} are installed", tool_name));
        return Ok(());
    }

    if let Some(requested_version) = version {
        let target_version =
            resolve_version_from_installed(tool_name, requested_version, &installed_versions)?;

        if requested_version != target_version {
            UI::detail(&format!(
                "Resolved {} → {}",
                requested_version, target_version
            ));
        }

        UI::info(&format!("Removing {} {}...", tool_name, target_version));
        runtime.pre_uninstall(&target_version, context).await?;

        match runtime.uninstall(&target_version, context).await {
            Ok(()) => {
                runtime.post_uninstall(&target_version, context).await?;
                invalidate_caches_for_runtime(tool_name, context);
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
        for ver in &installed_versions {
            if let Err(e) = runtime.pre_uninstall(ver, context).await {
                UI::error(&format!(
                    "Pre-uninstall hook failed for {} {}: {}",
                    tool_name, ver, e
                ));
                errors.push(e);
                continue;
            }
            match runtime.uninstall(ver, context).await {
                Ok(()) => {
                    let _ = runtime.post_uninstall(ver, context).await;
                    UI::detail(&format!("Removed {} {}", tool_name, ver));
                }
                Err(e) => {
                    UI::error(&format!("Failed to remove {} {}: {}", tool_name, ver, e));
                    errors.push(e);
                }
            }
        }

        invalidate_caches_for_runtime(tool_name, context);

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

// ---------------------------------------------------------------------------
// Hook helpers (best-effort, non-fatal)
// ---------------------------------------------------------------------------

async fn run_pre_uninstall_hook(
    registry: &ProviderRegistry,
    context: &RuntimeContext,
    tool_name: &str,
    version: &str,
) {
    if let Some(runtime) = registry.get_runtime(tool_name)
        && let Err(e) = runtime.pre_uninstall(version, context).await
    {
        UI::warn(&format!(
            "Pre-uninstall hook failed for {} {}: {}",
            tool_name, version, e
        ));
    }
}

async fn run_post_uninstall_hook(
    registry: &ProviderRegistry,
    context: &RuntimeContext,
    tool_name: &str,
    version: &str,
) {
    if let Some(runtime) = registry.get_runtime(tool_name)
        && let Err(e) = runtime.post_uninstall(version, context).await
    {
        UI::warn(&format!(
            "Post-uninstall hook failed for {} {}: {}",
            tool_name, version, e
        ));
    }
}

// ---------------------------------------------------------------------------
// Version resolution for legacy path
// ---------------------------------------------------------------------------

fn resolve_version_from_installed(
    tool_name: &str,
    requested: &str,
    installed: &[String],
) -> Result<String> {
    use vx_resolver::VersionRequest;

    let request = VersionRequest::parse(requested);

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
        return Err(anyhow::anyhow!(
            "No installed version matches '{}'. Installed: {}\n\nTip: Use 'vx versions {}' to see available versions.",
            requested,
            if installed.is_empty() {
                "none".to_string()
            } else {
                installed.join(", ")
            },
            tool_name,
        ));
    }

    matching.sort_by(|(a, _), (b, _)| b.cmp(a));
    Ok(matching.remove(0).1)
}

fn matches_constraint(
    version: &vx_resolver::Version,
    constraint: &vx_resolver::VersionConstraint,
) -> bool {
    use vx_resolver::VersionConstraint;
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
            if v.major > 0 {
                version.major == v.major && version >= v
            } else if v.minor > 0 {
                version.major == 0 && version.minor == v.minor && version >= v
            } else {
                version.major == 0 && version.minor == 0 && version.patch == v.patch
            }
        }
        VersionConstraint::Tilde(v) => {
            version.major == v.major && version.minor == v.minor && version >= v
        }
        VersionConstraint::Range(constraints) => constraints.iter().all(|c| c.satisfies(version)),
    }
}

// ---------------------------------------------------------------------------
// Cache invalidation
// ---------------------------------------------------------------------------

fn invalidate_caches_for_runtime(tool_name: &str, context: &RuntimeContext) {
    let runtime_store_dir = context.paths.runtime_store_dir(tool_name);
    let prefix = runtime_store_dir.to_string_lossy().to_string();
    vx_resolver::invalidate_bin_dir_cache(&prefix);

    let cache_dir = context.paths.cache_dir();
    let mut exec_cache = vx_cache::ExecPathCache::load(&cache_dir);
    exec_cache.invalidate_runtime(&runtime_store_dir);
    if let Err(e) = exec_cache.save(&cache_dir) {
        tracing::debug!("Failed to save exec path cache after invalidation: {}", e);
    }
}
