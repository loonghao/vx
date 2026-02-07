//! Install command handler

use super::Args;
use crate::commands::CommandContext;
use crate::ui::{ProgressSpinner, UI};
use anyhow::Result;
use std::env;
use vx_paths::project::{find_vx_config, LOCK_FILE_NAME};
use vx_resolver::{LockFile, LockedTool};
use vx_runtime::{InstallResult, ProviderRegistry, RuntimeContext};

/// Parse tool specification in format "tool" or "tool@version"
fn parse_tool_spec(spec: &str) -> (&str, Option<&str>) {
    if let Some((tool, version)) = spec.split_once('@') {
        (tool, Some(version))
    } else {
        (spec, None)
    }
}

/// Handle install command with Args
pub async fn handle(ctx: &CommandContext, args: &Args) -> Result<()> {
    handle_install(
        ctx.registry(),
        ctx.runtime_context(),
        &args.tools,
        args.force,
    )
    .await
}

/// Legacy handle function for backwards compatibility
pub async fn handle_install(
    registry: &ProviderRegistry,
    context: &RuntimeContext,
    tools: &[String],
    force: bool,
) -> Result<()> {
    let mut success_count = 0;
    let mut fail_count = 0;
    let total = tools.len();
    let is_multi = total > 1;

    for (idx, tool_spec) in tools.iter().enumerate() {
        let (tool_name, version) = parse_tool_spec(tool_spec);

        if is_multi {
            UI::section(&format!("[{}/{}] {}", idx + 1, total, tool_spec));
        }

        match install_single(registry, context, tool_name, version, force, is_multi).await {
            Ok(()) => success_count += 1,
            Err(e) => {
                UI::error(&format!("Failed to install {}: {}", tool_spec, e));
                fail_count += 1;
            }
        }
    }

    // Summary for multiple tools
    if is_multi {
        println!();
        if fail_count == 0 {
            UI::success(&format!("Successfully installed {} tool(s)", success_count));
        } else {
            UI::warn(&format!(
                "Installed {} tool(s), {} failed",
                success_count, fail_count
            ));
        }
    }

    if fail_count > 0 {
        Err(anyhow::anyhow!("{} tool(s) failed to install", fail_count))
    } else {
        Ok(())
    }
}

async fn install_single(
    registry: &ProviderRegistry,
    context: &RuntimeContext,
    tool_name: &str,
    version: Option<&str>,
    force: bool,
    is_multi: bool,
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

    // Check if this runtime is bundled with another
    // If so, redirect installation to the parent runtime
    if let Some(bundled_with) = runtime.metadata().get("bundled_with") {
        if bundled_with != tool_name {
            UI::info(&format!(
                "'{}' is bundled with '{}'. Installing '{}' instead...",
                tool_name, bundled_with, bundled_with
            ));
            // Recursively install the parent runtime
            return Box::pin(install_single(
                registry,
                context,
                bundled_with,
                version,
                force,
                is_multi,
            ))
            .await;
        }
    }

    // Determine version to install
    let requested_version = version.unwrap_or("latest");

    // Try to load lock file and get download URL
    let mut context_with_cache = context.clone();
    if let Some((locked_version, download_url)) =
        get_download_url_from_lock(tool_name, requested_version)
    {
        if context.config.verbose {
            UI::detail(&format!(
                "Using locked version {} from {} (download URL cached)",
                locked_version, LOCK_FILE_NAME
            ));
        }
        // Cache the download URL
        let mut cache = std::collections::HashMap::new();
        cache.insert(tool_name.to_string(), download_url);
        context_with_cache.set_download_url_cache(cache);
    }

    // Resolve version (handles "latest", partial versions like "3.11", etc.)
    let resolve_msg = if is_multi {
        format!("Resolving {}...", requested_version)
    } else {
        format!(
            "Resolving version {} for {}...",
            requested_version, tool_name
        )
    };
    let spinner = ProgressSpinner::new(&resolve_msg);

    // Update spinner message to show network activity
    spinner.set_message(&format!("{} (fetching versions...)", resolve_msg));
    let target_version = runtime
        .resolve_version(requested_version, &context_with_cache)
        .await?;
    spinner.finish_and_clear();

    if requested_version != target_version {
        UI::detail(&format!(
            "Resolved {} → {}",
            requested_version, target_version
        ));
    }

    // Check if already installed
    if !force
        && runtime
            .is_installed(&target_version, &context_with_cache)
            .await?
    {
        UI::success(&format!(
            "{} {} is already installed",
            tool_name, target_version
        ));
        UI::hint("Use --force to reinstall");
        return Ok(());
    }

    // Run pre-install hook
    runtime
        .pre_install(&target_version, &context_with_cache)
        .await?;

    // Install the version
    let install_result = if is_multi {
        // In multi-tool mode, use simpler output without spinner
        // to avoid visual clutter
        runtime.install(&target_version, &context_with_cache).await
    } else {
        // In single-tool mode, show spinner
        // Note: new_install template already includes "Installing" prefix
        let spinner = ProgressSpinner::new_install(&format!("{} {}...", tool_name, target_version));
        let result = runtime.install(&target_version, &context_with_cache).await;
        match &result {
            Ok(_) => spinner.finish_with_message(&format!(
                "✓ Successfully installed {} {}",
                tool_name, target_version
            )),
            Err(e) => spinner.finish_with_error(&format!(
                "Failed to install {} {}: {}",
                tool_name, target_version, e
            )),
        }
        result
    };

    match install_result {
        Ok(result) => {
            if is_multi {
                UI::success(&format!("Installed {} {}", tool_name, target_version));
            }

            // Run post-install hook
            runtime
                .post_install(&target_version, &context_with_cache)
                .await?;

            // Invalidate exec path caches so stale entries are not used
            invalidate_caches_for_runtime(tool_name, context);

            // Show installation path
            UI::detail(&format!("Installed to: {}", result.install_path.display()));

            // Update lock file if it exists
            update_lockfile_if_exists(
                tool_name,
                &target_version,
                requested_version,
                runtime.ecosystem(),
            );

            // Show usage hint
            if !is_multi {
                UI::hint(&format!(
                    "Use 'vx {} --version' to verify installation",
                    tool_name
                ));
            }
        }
        Err(e) => {
            if is_multi {
                UI::error(&format!(
                    "Failed to install {} {}: {}",
                    tool_name, target_version, e
                ));
            }
            return Err(e);
        }
    }

    Ok(())
}

/// Update lock file if it exists in the current project
fn update_lockfile_if_exists(
    tool_name: &str,
    version: &str,
    resolved_from: &str,
    ecosystem: vx_runtime::Ecosystem,
) {
    // Try to find project root with vx.toml
    let current_dir = match env::current_dir() {
        Ok(d) => d,
        Err(_) => return,
    };

    let config_path = match find_vx_config(&current_dir) {
        Ok(p) => p,
        Err(_) => return, // No project config, skip lock file update
    };

    let project_root = match config_path.parent() {
        Some(p) => p,
        None => return,
    };

    let lock_path = project_root.join(LOCK_FILE_NAME);

    // Only update if lock file already exists
    if !lock_path.exists() {
        return;
    }

    // Load existing lock file
    let mut lockfile = match LockFile::load(&lock_path) {
        Ok(lf) => lf,
        Err(_) => return,
    };

    // Convert ecosystem
    let resolver_ecosystem = match ecosystem {
        vx_runtime::Ecosystem::NodeJs => vx_resolver::Ecosystem::Node,
        vx_runtime::Ecosystem::Python => vx_resolver::Ecosystem::Python,
        vx_runtime::Ecosystem::Rust => vx_resolver::Ecosystem::Rust,
        vx_runtime::Ecosystem::Go => vx_resolver::Ecosystem::Go,
        _ => vx_resolver::Ecosystem::Generic,
    };

    // Create locked tool entry
    let locked_tool = LockedTool::new(version, "vx install")
        .with_resolved_from(resolved_from)
        .with_ecosystem(resolver_ecosystem);

    // Update lock file
    lockfile.lock_tool(tool_name, locked_tool);

    // Save lock file
    if let Err(e) = lockfile.save(&lock_path) {
        UI::warn(&format!("Failed to update {}: {}", LOCK_FILE_NAME, e));
    } else {
        UI::detail(&format!(
            "Updated {} with {} = {}",
            LOCK_FILE_NAME, tool_name, version
        ));
    }
}

/// Get download URL from lock file for a tool
///
/// Returns (locked_version, download_url) if found, None otherwise.
/// This allows install commands to use pre-resolved URLs from vx.lock
/// instead of re-fetching them from providers.
fn get_download_url_from_lock(
    tool_name: &str,
    _requested_version: &str,
) -> Option<(String, String)> {
    // Try to find project root with vx.toml
    let current_dir = env::current_dir().ok()?;
    let config_path = find_vx_config(&current_dir).ok()?;
    let project_root = config_path.parent()?;
    let lock_path = project_root.join(LOCK_FILE_NAME);

    // Load lock file
    let lockfile = LockFile::load(&lock_path).ok()?;

    // Get locked tool
    let locked_tool = lockfile.get_tool(tool_name)?;

    // Get download URL
    let download_url = locked_tool.download_url.clone()?;

    // Use locked version
    let locked_version = locked_tool.version.clone();

    // Check if requested version matches locked version or uses version constraint
    // For "latest", always use locked version
    // For specific versions, check if they match
    // Version constraint or partial version - use locked version anyway
    // since lock file already resolved to a specific version
    // Always use locked version when lock file exists
    Some((locked_version, download_url))
}

/// Install a runtime quietly (for CI testing)
/// Returns the InstallResult on success, including executable path
pub async fn install_quiet(
    registry: &ProviderRegistry,
    context: &RuntimeContext,
    tool_name: &str,
) -> Result<InstallResult> {
    // Get the runtime from registry
    let runtime = match registry.get_runtime(tool_name) {
        Some(r) => r,
        None => {
            return Err(anyhow::anyhow!("Tool not found: {}", tool_name));
        }
    };

    // Check if this runtime is bundled with another
    if let Some(bundled_with) = runtime.metadata().get("bundled_with") {
        if bundled_with != tool_name {
            return Box::pin(install_quiet(registry, context, bundled_with)).await;
        }
    }

    // Resolve latest version
    let target_version = runtime.resolve_version("latest", context).await?;

    // Try to use lock file URL
    let mut context_with_cache = context.clone();
    if let Some((_locked_version, download_url)) = get_download_url_from_lock(tool_name, "latest") {
        let mut cache = std::collections::HashMap::new();
        cache.insert(tool_name.to_string(), download_url);
        context_with_cache.set_download_url_cache(cache);
    }

    // Check if already installed
    if runtime
        .is_installed(&target_version, &context_with_cache)
        .await?
    {
        // For already installed, use the runtime's method to get the correct executable path
        // This handles different directory structures (e.g., node-v24.13.0-win-x64/node.exe)
        let store_name = runtime.store_name();
        let install_path = context.paths.version_store_dir(store_name, &target_version);

        // Use get_executable_path_for_version which properly handles each runtime's layout
        if let Ok(Some(exe_path)) = runtime
            .get_executable_path_for_version(&target_version, context)
            .await
        {
            return Ok(InstallResult::already_installed(
                install_path,
                exe_path,
                target_version,
            ));
        }

        // Fall back to system PATH if store path not found
        let exe_name = runtime.name();
        if let Ok(exe_path) = which::which(exe_name) {
            return Ok(InstallResult::system_installed(
                target_version,
                Some(exe_path),
            ));
        }

        // Last resort: return a reasonable default (may not exist)
        let platform = vx_runtime::Platform::current();
        let exe_relative = runtime.executable_relative_path(&target_version, &platform);
        let exe_path = install_path.join(&exe_relative);
        return Ok(InstallResult::already_installed(
            install_path,
            exe_path,
            target_version,
        ));
    }

    // Run pre-install hook
    runtime
        .pre_install(&target_version, &context_with_cache)
        .await?;

    // Install the version
    let install_result = runtime
        .install(&target_version, &context_with_cache)
        .await?;

    // Run post-install hook
    runtime.post_install(&target_version, context).await?;

    Ok(install_result)
}

/// Invalidate exec path caches after install/uninstall.
///
/// Clears the process-level bin-dir cache and the on-disk exec-path cache
/// for the given runtime so that subsequent lookups re-discover executables.
fn invalidate_caches_for_runtime(tool_name: &str, context: &RuntimeContext) {
    // 1. Process-level BIN_DIR_CACHE (used by build_vx_tools_path)
    let runtime_store_dir = context.paths.runtime_store_dir(tool_name);
    let prefix = runtime_store_dir.to_string_lossy().to_string();
    vx_resolver::invalidate_bin_dir_cache(&prefix);

    // 2. On-disk exec path cache (used by PathResolver::find_executable_in_dir)
    let cache_dir = context.paths.cache_dir();
    let mut exec_cache = vx_cache::ExecPathCache::load(&cache_dir);
    exec_cache.invalidate_runtime(&runtime_store_dir);
    if let Err(e) = exec_cache.save(&cache_dir) {
        tracing::debug!("Failed to save exec path cache after invalidation: {}", e);
    }
}
