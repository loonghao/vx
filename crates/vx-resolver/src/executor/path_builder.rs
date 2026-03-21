//! PATH builder and version selection for vx-managed tools
//!
//! This module provides:
//! - `build_vx_tools_path()`: Build a PATH string containing all vx-managed tool bin directories
//! - `select_version_for_runtime()`: Select the best version for a runtime based on project config
//!
//! NOTE: These functions are the modularized versions of the corresponding methods on
//! `EnvironmentManager`. They will fully replace those methods in a future refactor.

use tracing::{trace, warn};
use vx_runtime::RuntimeContext;

use super::bin_dir_cache::{find_bin_dir, record_warned_tool};
use super::project_config::ProjectToolsConfig;
use super::version_utils;

/// Build PATH string containing all vx-managed tool bin directories
///
/// **Performance optimization**: Instead of calling `registry.supported_runtimes()`
/// (which triggers `materialize_all()` and instantiates all ~45 providers), we
/// directly scan `~/.vx/store/` to discover installed runtimes. This avoids
/// provider materialization entirely, reducing prepare stage from ~400ms to <50ms.
pub fn build_vx_tools_path(
    context: &RuntimeContext,
    project_config: Option<&ProjectToolsConfig>,
) -> Option<String> {
    let mut paths: Vec<String> = Vec::new();

    // Add vx bin directory first (for shims)
    let vx_bin = context.paths.bin_dir();
    if vx_bin.exists() {
        paths.push(vx_bin.to_string_lossy().to_string());
    }

    // Scan store directory directly to find installed runtimes.
    // This is much faster than materialize_all() + supported_runtimes() because
    // it only does a shallow read_dir of ~/.vx/store/ and doesn't instantiate
    // any provider factories.
    let store_dir = context.paths.store_dir();
    if !store_dir.exists() {
        return if paths.is_empty() {
            None
        } else {
            Some(vx_paths::join_paths_simple(&paths))
        };
    }

    let installed_runtimes: Vec<String> = match std::fs::read_dir(&store_dir) {
        Ok(entries) => entries
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_dir())
            .filter_map(|e| e.file_name().into_string().ok())
            .collect(),
        Err(_) => Vec::new(),
    };

    for runtime_name in &installed_runtimes {
        let runtime_store_dir = context.paths.runtime_store_dir(runtime_name);

        if let Ok(entries) = std::fs::read_dir(&runtime_store_dir) {
            let installed_versions: Vec<String> = entries
                .filter_map(|e| e.ok())
                .filter(|e| e.path().is_dir())
                .filter_map(|e| e.file_name().into_string().ok())
                .collect();

            let version_to_use =
                select_version_for_runtime(runtime_name, &installed_versions, project_config);

            if let Some(version) = version_to_use {
                let store_dir = context.paths.version_store_dir(runtime_name, &version);

                if let Some(bin_dir) = find_bin_dir(&store_dir, runtime_name)
                    && bin_dir.exists()
                {
                    let bin_path = bin_dir.to_string_lossy().to_string();
                    if !paths.contains(&bin_path) {
                        paths.push(bin_path);
                    }
                }
            }
        }
    }

    if paths.is_empty() {
        None
    } else {
        Some(vx_paths::join_paths_simple(&paths))
    }
}

/// Select the version to use for a runtime, prioritizing project configuration
pub fn select_version_for_runtime(
    runtime_name: &str,
    installed_versions: &[String],
    project_config: Option<&ProjectToolsConfig>,
) -> Option<String> {
    if installed_versions.is_empty() {
        return None;
    }

    // Check if project configuration specifies a version for this runtime
    if let Some(project_config) = project_config
        && let Some(requested_version) = project_config.get_version_with_fallback(runtime_name)
    {
        // "latest" means "use the latest installed version" — skip matching
        if requested_version == "latest" {
            let mut versions = installed_versions.to_vec();
            versions.sort_by(|a, b| version_utils::compare_versions(a, b));
            if let Some(latest) = versions.last() {
                trace!(
                    "Using {} version {} (latest installed) from vx.toml",
                    runtime_name, latest
                );
                return Some(latest.clone());
            }
        }

        let matching_version =
            version_utils::find_matching_version(requested_version, installed_versions);

        if let Some(version) = matching_version {
            trace!("Using {} version {} from vx.toml", runtime_name, version);
            return Some(version);
        } else {
            // Requested version not installed, warn and fall back to latest
            if record_warned_tool(runtime_name) {
                warn!(
                    "Version {} specified in vx.toml for {} is not installed. \
                         Using latest installed version instead. \
                         Run 'vx install {}@{}' to install the specified version.",
                    requested_version, runtime_name, runtime_name, requested_version
                );
            }
        }
    }

    // Fall back to latest installed version
    let mut versions = installed_versions.to_vec();
    versions.sort_by(|a, b| version_utils::compare_versions(a, b));

    versions.last().cloned()
}
