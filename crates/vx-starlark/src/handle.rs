//! ProviderHandle - Unified facade for all provider operations (RFC 0037)
//!
//! `ProviderHandle` is the single entry point for CLI commands to interact with
//! providers. All business logic is delegated to `provider.star`; Rust only
//! acts as a registration stub and execution bridge.
//!
//! # Architecture
//!
//! ```text
//! CLI commands
//!     └── ProviderHandle (unified facade)
//!             └── StarlarkProvider (Starlark execution)
//!                     └── provider.star (all business logic)
//! ```

use crate::context::VersionInfo;
use crate::error::{Error, Result};
use crate::provider::version_cache::{VersionCacheStats, global_version_cache};
use crate::provider::{
    EnvOp, InstallLayout, PostExtractAction, ProviderMeta, RuntimeMeta, StarlarkProvider,
    apply_env_ops,
};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tracing::warn;
use vx_paths::VxPaths;

// ---------------------------------------------------------------------------
// DepRequirement - structured dependency descriptor
// ---------------------------------------------------------------------------

/// A dependency requirement returned by `deps()` in provider.star
#[derive(Debug, Clone)]
pub struct DepRequirement {
    /// The runtime name this provider depends on (e.g. "git", "node")
    pub runtime: String,
    /// Version constraint (e.g. "*", ">=2.0", "^18")
    pub version_req: String,
    /// Whether this dependency is optional
    pub optional: bool,
    /// Human-readable reason for the dependency
    pub reason: Option<String>,
}

// ---------------------------------------------------------------------------
// PostInstallOps - post-install operations descriptor
// ---------------------------------------------------------------------------

/// Post-install operations returned by `post_install()` in provider.star
#[derive(Debug, Clone)]
pub enum PostInstallOps {
    /// Create a symbolic link
    Symlink {
        /// Source path (the file to link from)
        source: String,
        /// Target path (the link to create)
        target: String,
    },
    /// Set file permissions
    SetPermissions {
        /// Path to the file
        path: String,
        /// Unix permission mode (e.g. "755")
        mode: String,
    },
    /// Run a command after installation
    RunCommand {
        /// Executable to run
        executable: String,
        /// Arguments
        args: Vec<String>,
        /// Working directory
        working_dir: Option<String>,
    },
}

// ---------------------------------------------------------------------------
// VersionFilter
// ---------------------------------------------------------------------------

/// Filter for version queries
#[derive(Debug, Clone, Default)]
pub struct VersionFilter {
    /// Include pre-release versions
    pub include_prerelease: bool,
    /// Maximum number of versions to return (0 = all)
    pub limit: usize,
    /// Filter by LTS only
    pub lts_only: bool,
}

// ---------------------------------------------------------------------------
// ProviderHandle
// ---------------------------------------------------------------------------

/// Unified facade for all provider operations (RFC 0037)
///
/// All business logic is delegated to `provider.star`. Rust only acts as
/// a registration stub and execution bridge.
#[derive(Debug, Clone)]
pub struct ProviderHandle {
    /// Provider name (e.g. "7zip", "node")
    name: String,
    /// Runtime name within a multi-runtime provider (e.g. "yazi" within "shell-tools").
    /// When set, path queries use this name for store directory lookup instead of
    /// the provider name. This allows each runtime to have its own store directory.
    runtime_name: Option<String>,
    /// Starlark provider instance
    star: Arc<StarlarkProvider>,
    /// VX paths
    paths: Arc<VxPaths>,
}

impl ProviderHandle {
    // ── Construction ──────────────────────────────────────────────────────

    /// Load a ProviderHandle from embedded provider.star content (for built-in providers)
    pub async fn from_content(name: impl Into<String>, content: &'static str) -> Result<Self> {
        let name = name.into();
        let star = StarlarkProvider::from_content(&name, content).await?;
        check_vx_version_requirement(star.meta(), &name);
        let paths = VxPaths::new()
            .map_err(|e| Error::EvalError(format!("Failed to initialize VxPaths: {e}")))?;

        Ok(Self {
            name,
            runtime_name: None,
            star: Arc::new(star),
            paths: Arc::new(paths),
        })
    }

    /// Create a ProviderHandle from a dynamically-loaded string (e.g. user-defined providers).
    pub async fn from_string(name: impl Into<String>, content: impl Into<String>) -> Result<Self> {
        let name = name.into();
        let star = StarlarkProvider::from_content(&name, content).await?;
        check_vx_version_requirement(star.meta(), &name);
        let paths = VxPaths::new()
            .map_err(|e| Error::EvalError(format!("Failed to initialize VxPaths: {e}")))?;

        Ok(Self {
            name,
            runtime_name: None,
            star: Arc::new(star),
            paths: Arc::new(paths),
        })
    }

    /// Load a ProviderHandle from a file path (for user-defined providers)
    pub async fn load(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let star = StarlarkProvider::load(path).await?;
        let name = star.name().to_string();
        check_vx_version_requirement(star.meta(), &name);
        let paths = VxPaths::new()
            .map_err(|e| Error::EvalError(format!("Failed to initialize VxPaths: {e}")))?;

        Ok(Self {
            name,
            runtime_name: None,
            star: Arc::new(star),
            paths: Arc::new(paths),
        })
    }

    /// Create a runtime-specific handle from an existing handle.
    ///
    /// For multi-runtime providers (e.g. shell-tools with starship/atuin/yazi),
    /// this creates a handle that uses the runtime name for store directory lookup
    /// and passes it as `ctx.runtime_name` to Starlark functions.
    pub fn for_runtime(&self, runtime_name: impl Into<String>) -> Self {
        Self {
            name: self.name.clone(),
            runtime_name: Some(runtime_name.into()),
            star: self.star.clone(),
            paths: self.paths.clone(),
        }
    }

    // ── Metadata ──────────────────────────────────────────────────────────

    /// Provider name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Provider description
    pub fn description(&self) -> &str {
        self.star.description()
    }

    /// Provider metadata
    pub fn provider_meta(&self) -> &ProviderMeta {
        self.star.meta()
    }

    /// Runtime definitions
    pub fn runtime_metas(&self) -> &[RuntimeMeta] {
        self.star.runtimes()
    }

    // ── Version Management ─────────────────────────────────────────────────

    /// Fetch available versions from provider.star::fetch_versions
    ///
    /// Corresponds to `vx versions <tool>`
    pub async fn versions(&self, filter: VersionFilter) -> Result<Vec<VersionInfo>> {
        let versions = self
            .star
            .fetch_versions_for_runtime(self.runtime_name.as_deref())
            .await?;

        let mut filtered: Vec<VersionInfo> = versions
            .into_iter()
            .filter(|v| {
                if !filter.include_prerelease && !v.stable {
                    return false;
                }
                if filter.lts_only && !v.lts {
                    return false;
                }
                true
            })
            .collect();

        if filter.limit > 0 && filtered.len() > filter.limit {
            filtered.truncate(filter.limit);
        }

        Ok(filtered)
    }

    /// Fetch versions with explicit cache bypass (force refresh from network)
    ///
    /// Invalidates the cache for this provider first, then fetches fresh versions.
    /// Useful for `vx versions --refresh <tool>`.
    pub async fn versions_refresh(&self, filter: VersionFilter) -> Result<Vec<VersionInfo>> {
        // Invalidate cache for this provider
        let cache = global_version_cache();
        cache.invalidate(&self.name).await;
        tracing::debug!(provider = %self.name, "Invalidated version cache for refresh");

        // Fetch fresh versions (will miss cache and re-fetch)
        self.versions(filter).await
    }

    /// Invalidate the version cache for this provider
    ///
    /// Next call to `versions()` will fetch fresh data from the network.
    pub async fn invalidate_version_cache(&self) {
        let cache = global_version_cache();
        cache.invalidate(&self.name).await;
        tracing::debug!(provider = %self.name, "Version cache invalidated");
    }

    /// Get version cache statistics for this provider
    pub async fn version_cache_stats(&self) -> VersionCacheStats {
        let cache = global_version_cache();
        cache.stats().await
    }

    /// Get list of installed versions by scanning the store directory.
    ///
    /// If no vx-managed versions are found, checks whether the tool is available
    /// on the system PATH (installed via a system package manager such as winget,
    /// brew, or apt).  In that case `["system"]` is returned so that
    /// `vx uninstall <tool>` can delegate to the provider's `uninstall()` hook.
    pub fn installed_versions(&self) -> Vec<String> {
        let store_root = self.store_root();
        if store_root.exists() {
            let mut versions: Vec<String> = std::fs::read_dir(&store_root)
                .ok()
                .map(|entries| {
                    entries
                        .filter_map(|e| e.ok())
                        .filter(|e| e.path().is_dir())
                        .filter_map(|e| e.file_name().into_string().ok())
                        .collect()
                })
                .unwrap_or_default();

            if !versions.is_empty() {
                // Sort versions in descending order (newest first)
                versions.sort_by(|a, b| b.cmp(a));
                return versions;
            }
        }

        // No vx-managed versions found — check if the tool is system-installed.
        // For multi-runtime providers, look up the executable for the specific runtime.
        let exe_name = self.runtime_executable_name();

        if which::which(&exe_name).is_ok() {
            tracing::debug!(
                provider = %self.name,
                executable = %exe_name,
                "Found system-installed tool"
            );
            return vec!["system".to_string()];
        }

        vec![]
    }

    /// Check if a specific version is installed
    pub fn is_installed(&self, version: &str) -> bool {
        if version == "system" {
            let exe_name = self.runtime_executable_name();
            return which::which(&exe_name).is_ok();
        }
        self.store_root().join(version).exists()
    }

    /// Get the executable name for the current runtime.
    ///
    /// For multi-runtime providers, returns the executable for the specific runtime
    /// (matched by runtime_name). Falls back to the first runtime's executable.
    fn runtime_executable_name(&self) -> String {
        if let Some(ref rt_name) = self.runtime_name {
            // Find the matching runtime definition
            if let Some(rt) = self.star.runtimes().iter().find(|r| &r.name == rt_name) {
                return rt.executable.clone();
            }
        }
        // Fallback: use the first runtime's executable
        self.star
            .runtimes()
            .first()
            .map(|r| r.executable.clone())
            .unwrap_or_else(|| self.name.clone())
    }

    /// Uninstall a specific version.
    ///
    /// **Two-phase strategy** (mirrors the install pipeline):
    ///
    /// 1. **provider.star hook** — if `uninstall(ctx, version)` is defined in the
    ///    provider script, it is called first.  The hook can perform tool-specific
    ///    cleanup (e.g. removing pip caches, npm global packages, Go module cache).
    ///    If the hook returns `false` the default phase still runs.
    ///
    /// 2. **Default fallback** — removes the version directory from the store
    ///    (`{vx_home}/store/{provider}/{version}`).  This runs whenever the hook
    ///    is absent *or* returns `false`.
    ///
    /// Returns `Ok(())` on success, `Err` if the version is not installed or
    /// any removal step fails.
    pub async fn uninstall(&self, version: &str) -> Result<()> {
        // ── System-installed tool path ────────────────────────────────────
        // When the tool was installed via a system package manager (winget,
        // brew, apt, …) there is no vx store directory.  Delegate entirely
        // to the provider.star::uninstall() hook.
        if version == "system" {
            let hook_handled = self.star.uninstall(version).await.unwrap_or_else(|e| {
                tracing::warn!(
                    provider = %self.name,
                    error    = %e,
                    "provider.star uninstall() failed for system version"
                );
                false
            });

            if hook_handled {
                tracing::info!(
                    provider = %self.name,
                    "Uninstalled system version via provider hook"
                );
                return Ok(());
            }

            // Hook returned false or is absent — we cannot remove a system
            // installation without knowing the package manager.  Surface a
            // helpful message instead of a cryptic error.
            return Err(Error::EvalError(format!(
                "{} is installed via a system package manager. \
                 Please uninstall it manually (e.g. `winget uninstall`, `brew uninstall`, etc.).",
                self.name
            )));
        }

        // ── vx-managed version path ───────────────────────────────────────
        let version_dir = self.store_root().join(version);
        if !version_dir.exists() {
            return Err(Error::EvalError(format!(
                "{} {} is not installed",
                self.name, version
            )));
        }

        // Phase 1: try provider.star::uninstall hook
        let hook_handled = self.star.uninstall(version).await.unwrap_or_else(|e| {
            tracing::warn!(
                provider = %self.name,
                version  = %version,
                error    = %e,
                "provider.star uninstall() failed, falling back to default removal"
            );
            false
        });

        // Phase 2: default directory removal (runs when hook is absent or returns false)
        if !hook_handled {
            std::fs::remove_dir_all(&version_dir).map_err(|e| {
                Error::EvalError(format!("Failed to remove {} {}: {}", self.name, version, e))
            })?;
        }

        tracing::info!(
            provider    = %self.name,
            version     = %version,
            custom_hook = hook_handled,
            "Uninstalled version"
        );
        Ok(())
    }

    /// Uninstall all installed versions of this provider.
    ///
    /// Calls [`uninstall`] for each installed version in sequence.
    /// Continues on individual failures and returns the first error encountered.
    pub async fn uninstall_all(&self) -> Result<()> {
        let versions = self.installed_versions();
        if versions.is_empty() {
            return Ok(());
        }
        let mut first_err: Option<Error> = None;
        for version in &versions {
            if let Err(e) = self.uninstall(version).await {
                tracing::warn!(
                    provider = %self.name,
                    version  = %version,
                    error    = %e,
                    "Failed to uninstall version"
                );
                if first_err.is_none() {
                    first_err = Some(e);
                }
            }
        }
        match first_err {
            Some(e) => Err(e),
            None => Ok(()),
        }
    }

    /// Resolve a version request against installed versions.
    ///
    /// Supports:
    /// - Exact: `"3.7.13"` → `"3.7.13"`
    /// - Partial: `"3.7"` → `"3.7.13"` (latest matching 3.7.x)
    /// - Major: `"3"` → `"3.12.0"` (latest matching 3.x.x)
    /// - `"latest"` / `"*"` → newest installed version
    ///
    /// Returns `Err` if no installed version matches.
    pub fn resolve_installed_version(&self, requested: &str) -> Result<String> {
        let installed = self.installed_versions();
        if installed.is_empty() {
            return Err(Error::EvalError(format!(
                "No versions of {} are installed",
                self.name
            )));
        }

        // "latest" / "*" → return newest
        if requested == "latest" || requested == "*" {
            return Ok(installed[0].clone());
        }

        // Try exact match first
        if installed.contains(&requested.to_string()) {
            return Ok(requested.to_string());
        }

        // Partial match: split by '.' and check prefix
        let parts: Vec<&str> = requested.split('.').collect();
        let mut matching: Vec<&String> = installed
            .iter()
            .filter(|v| {
                let v_parts: Vec<&str> = v.split('.').collect();
                parts
                    .iter()
                    .enumerate()
                    .all(|(i, p)| v_parts.get(i).map(|vp| *vp == *p).unwrap_or(false))
            })
            .collect();

        if matching.is_empty() {
            return Err(Error::EvalError(format!(
                "No installed version of {} matches '{}'. Installed: {}",
                self.name,
                requested,
                installed.join(", ")
            )));
        }

        // Return the first (newest, since installed_versions is sorted descending)
        Ok(matching.remove(0).clone())
    }

    // ── Path Queries ───────────────────────────────────────────────────────

    /// Get the store root directory for this provider (sync, convention-based)
    ///
    /// Convention-based default: `{vx_home}/store/{provider_name}`
    ///
    /// Corresponds to `vx where <tool>` (no version)
    pub fn store_root(&self) -> PathBuf {
        // For multi-runtime providers, use the runtime name as the store directory.
        // e.g. "yazi" within "shell-tools" stores at ~/.vx/store/yazi, not ~/.vx/store/shell-tools
        let store_name = self.runtime_name.as_deref().unwrap_or(&self.name);
        self.paths.store_dir.join(store_name)
    }

    /// Get the store root directory (convention-based: `{vx_home}/store/{provider_name}`).
    ///
    /// Previously this called `store_root(ctx)` in provider.star, but since no
    /// provider.star defines that function, we use the convention directly.
    pub fn store_root_from_star(&self) -> PathBuf {
        self.paths.store_dir.join(&self.name)
    }

    /// Get the executable path for a specific version.
    ///
    /// Uses convention-based path scanning. Previously this called
    /// `get_execute_path(ctx, version)` in provider.star, but since no
    /// provider.star defines that function, we use the convention directly.
    ///
    /// Corresponds to `vx where <tool>@<version>`
    pub fn get_execute_path(&self, version: &str) -> Option<PathBuf> {
        self.convention_execute_path(version)
    }

    /// Get the executable path for the latest installed version
    ///
    /// Corresponds to `vx where <tool>` (when versions are installed)
    pub fn get_latest_execute_path(&self) -> Option<PathBuf> {
        let versions = self.installed_versions();
        let latest = versions.first()?.clone();
        self.get_execute_path(&latest)
    }

    // ── Installation ───────────────────────────────────────────────────────

    /// Get download URL for a specific version
    ///
    /// Delegates to provider.star::download_url
    pub async fn download_url(&self, version: &str) -> Result<Option<String>> {
        self.star
            .download_url_for_runtime(version, self.runtime_name.as_deref())
            .await
    }

    /// Get install layout for a specific version
    ///
    /// Delegates to provider.star::install_layout
    pub async fn install_layout(&self, version: &str) -> Result<Option<InstallLayout>> {
        self.star
            .install_layout_for_runtime(version, self.runtime_name.as_deref())
            .await
    }

    /// Get post-install operations for a specific version
    ///
    /// Calls `post_install(ctx, version, install_dir)` in provider.star first;
    /// falls back to `post_extract()` for backward compatibility.
    pub async fn post_install(
        &self,
        version: &str,
        install_dir: &Path,
    ) -> Result<Vec<PostInstallOps>> {
        // Try provider.star::post_install first
        let post_install_actions = self.star.call_post_install(version, install_dir).await?;
        if !post_install_actions.is_empty() {
            return Ok(post_install_actions
                .into_iter()
                .filter_map(post_extract_to_post_install)
                .collect());
        }
        // Fall back to post_extract() for backward compatibility
        let actions = self.star.post_extract(version, install_dir).await?;
        Ok(actions
            .into_iter()
            .filter_map(post_extract_to_post_install)
            .collect())
    }

    // ── Execution ──────────────────────────────────────────────────────────

    /// Get environment variable operations for a specific version (new API)
    ///
    /// Returns a list of [`EnvOp`]s that describe how to set up the environment.
    /// Use [`apply_env_ops`] to apply them to a mutable env map.
    ///
    /// **System-installed tools** (`version == "system"`): instead of calling
    /// `provider.star::environment()` (which uses `ctx.install_dir` and would
    /// produce a wrong path), we locate the executable via `which` and return
    /// a `PATH` prepend for its parent directory.  This ensures that tools
    /// declared as deps (e.g. `git`) are visible inside `vx dev` even when
    /// they are managed by the OS package manager rather than vx.
    pub async fn environment_ops(&self, version: &str) -> Result<Vec<EnvOp>> {
        if version == "system" {
            // Locate the primary executable on the system PATH
            let exe_name = self
                .star
                .runtimes()
                .first()
                .map(|r| r.executable.clone())
                .unwrap_or_else(|| self.name.clone());

            if let Ok(exe_path) = which::which(&exe_name)
                && let Some(bin_dir) = exe_path.parent()
            {
                let sep = if cfg!(windows) {
                    ";".to_string()
                } else {
                    ":".to_string()
                };
                return Ok(vec![EnvOp::Prepend {
                    key: "PATH".to_string(),
                    value: bin_dir.to_string_lossy().to_string(),
                    sep,
                }]);
            }
            // System tool not found — return empty (caller may warn)
            return Ok(vec![]);
        }
        self.star.environment(version).await
    }
    /// Get environment variables for a specific version (legacy API)
    ///
    /// Applies all [`EnvOp`]s from `environment()` and returns the resulting map.
    /// For multi-tool composition, prefer [`environment_ops`] + [`apply_env_ops`].
    pub async fn environment(
        &self,
        version: &str,
        _install_dir: &Path,
    ) -> Result<HashMap<String, String>> {
        let ops = self.star.environment(version).await?;
        Ok(apply_env_ops(&ops, None))
    }

    /// Get dependencies for a specific version
    ///
    /// Calls `deps(ctx, version)` in provider.star and returns structured dependency requirements.
    /// Returns an empty list if `deps()` is not defined in provider.star.
    pub async fn deps(&self, version: &str) -> Result<Vec<DepRequirement>> {
        let raw = self.star.deps(version).await?;
        let deps = raw
            .into_iter()
            .filter_map(|item| {
                let runtime = item.get("runtime").and_then(|v| v.as_str())?.to_string();
                let version_req = item
                    .get("version")
                    .and_then(|v| v.as_str())
                    .unwrap_or("*")
                    .to_string();
                let optional = item
                    .get("optional")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                let reason = item
                    .get("reason")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                Some(DepRequirement {
                    runtime,
                    version_req,
                    optional,
                    reason,
                })
            })
            .collect();
        Ok(deps)
    }

    // ── Internal Helpers ───────────────────────────────────────────────────

    /// Convention-based executable path: scan version dir for known executables
    fn convention_execute_path(&self, version: &str) -> Option<PathBuf> {
        let version_dir = self.store_root().join(version);
        if !version_dir.exists() {
            return None;
        }

        // Get executable name for the current runtime
        let exe_name = self.runtime_executable_name();

        // Build all possible executable file names.
        // On Windows, many bundled runtimes (npm, npx, yarn, corepack) use .cmd,
        // not .exe. We must search for both extensions.
        let exe_names: Vec<String> = {
            let with_ext = vx_paths::with_executable_extension(&exe_name);
            if cfg!(windows) && with_ext != exe_name {
                // with_executable_extension adds .exe; also try .cmd
                vec![with_ext, format!("{}.cmd", exe_name), exe_name.clone()]
            } else {
                vec![with_ext, exe_name.clone()]
            }
        };
        // Deduplicate (e.g. when exe_name already has an extension)
        let exe_names: Vec<String> = {
            let mut seen = std::collections::HashSet::new();
            exe_names
                .into_iter()
                .filter(|n| seen.insert(n.clone()))
                .collect()
        };

        // Directories to search within the version dir
        let platform_dir_name = vx_paths::platform_dir_name();
        let search_dirs: Vec<std::path::PathBuf> = vec![
            version_dir.clone(),
            version_dir.join("bin"),
            version_dir.join(platform_dir_name),
            version_dir.join(platform_dir_name).join("bin"),
            version_dir.join(&self.name),
        ];

        // Build candidate paths: each directory × each executable name
        let mut candidates: Vec<PathBuf> = Vec::new();
        for dir in &search_dirs {
            for name in &exe_names {
                candidates.push(dir.join(name));
            }
        }

        for candidate in &candidates {
            if candidate.exists() {
                return Some(candidate.clone());
            }
        }

        // Return the most likely path even if it doesn't exist yet
        Some(candidates[0].clone())
    }
}

/// Convert a PostExtractAction to a PostInstallOps (backward compatibility)
fn post_extract_to_post_install(action: PostExtractAction) -> Option<PostInstallOps> {
    match action {
        PostExtractAction::SetPermissions { path, mode } => {
            Some(PostInstallOps::SetPermissions { path, mode })
        }
        PostExtractAction::RunCommand {
            executable,
            args,
            working_dir,
            ..
        } => Some(PostInstallOps::RunCommand {
            executable,
            args,
            working_dir,
        }),
        // CreateShim is not directly mappable to PostInstallOps
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// ProviderHandleRegistry
// ---------------------------------------------------------------------------

/// Global registry of ProviderHandles
///
/// Supports lookup by name or alias.
#[derive(Debug, Default)]
pub struct ProviderHandleRegistry {
    /// Handles indexed by canonical provider name
    handles: HashMap<String, Arc<ProviderHandle>>,
    /// Alias → canonical name mapping
    aliases: HashMap<String, String>,
}

impl ProviderHandleRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a built-in provider from embedded star content
    pub async fn register_builtin(&mut self, name: &str, star_content: &'static str) -> Result<()> {
        let handle = ProviderHandle::from_content(name, star_content).await?;
        self.insert(handle);
        Ok(())
    }

    /// Register a dynamically-loaded provider (e.g. from user-defined provider.star files)
    pub async fn register_dynamic(&mut self, name: &str, star_content: String) -> Result<()> {
        let handle = ProviderHandle::from_string(name, star_content).await?;
        self.insert(handle);
        Ok(())
    }

    /// Register a provider from a file path (user-defined providers)
    pub async fn register_from_file(&mut self, path: &Path) -> Result<()> {
        let handle = ProviderHandle::load(path).await?;
        self.insert(handle);
        Ok(())
    }

    /// Insert a handle and register all its runtime aliases
    fn insert(&mut self, handle: ProviderHandle) {
        let canonical = handle.name().to_string();
        let runtimes = handle.runtime_metas().to_vec();

        // Register the provider-level handle (keyed by provider name)
        self.handles
            .insert(canonical.clone(), Arc::new(handle.clone()));

        // For multi-runtime providers, register a runtime-specific handle for each runtime.
        // This ensures that path queries (store_root, installed_versions, etc.) use the
        // correct runtime name instead of the provider name.
        for runtime in &runtimes {
            let rt_name = &runtime.name;

            // Create a runtime-specific handle if the runtime name differs from provider name
            if rt_name != &canonical {
                let rt_handle = Arc::new(handle.for_runtime(rt_name.clone()));
                self.handles.insert(rt_name.clone(), rt_handle);
            }

            // Register aliases pointing to the runtime-specific handle
            for alias in &runtime.aliases {
                if alias != &canonical && alias != rt_name {
                    // Alias points to the runtime name (which has its own handle)
                    self.aliases
                        .entry(alias.clone())
                        .or_insert_with(|| rt_name.clone());
                }
            }
        }
    }

    /// Get a ProviderHandle by name or alias
    pub fn get(&self, name: &str) -> Option<Arc<ProviderHandle>> {
        // Direct lookup
        if let Some(handle) = self.handles.get(name) {
            return Some(handle.clone());
        }
        // Alias lookup
        if let Some(canonical) = self.aliases.get(name) {
            return self.handles.get(canonical).cloned();
        }
        None
    }

    /// List all registered provider names
    pub fn names(&self) -> Vec<&str> {
        self.handles.keys().map(|s| s.as_str()).collect()
    }

    /// List all registered provider names and their aliases
    pub fn all_names_and_aliases(&self) -> Vec<&str> {
        let mut names: Vec<&str> = self.handles.keys().map(|s| s.as_str()).collect();
        names.extend(self.aliases.keys().map(|s| s.as_str()));
        names
    }

    /// Number of registered providers
    pub fn len(&self) -> usize {
        self.handles.len()
    }

    /// Whether the registry is empty
    pub fn is_empty(&self) -> bool {
        self.handles.is_empty()
    }

    /// Iterate over all handles
    pub fn iter(&self) -> impl Iterator<Item = (&str, &Arc<ProviderHandle>)> {
        self.handles.iter().map(|(k, v)| (k.as_str(), v))
    }

    /// Get all registered package prefixes across all providers.
    ///
    /// Package prefixes are declared in provider.star via `package_prefixes = ["deno", "npm"]`.
    /// They enable `vx <prefix>:<package>` syntax for ecosystem package execution.
    ///
    /// This is used by `is_package_request()` to dynamically determine
    /// if a command like `vx deno:cowsay` should be treated as a package request.
    pub fn get_all_package_prefixes(&self) -> Vec<&str> {
        self.handles
            .values()
            .flat_map(|handle| {
                handle
                    .provider_meta()
                    .package_prefixes
                    .iter()
                    .map(|s| s.as_str())
            })
            .collect()
    }

    /// Find the runtime name for a given `ecosystem:package` pair.
    ///
    /// Returns the provider's primary runtime name if any registered provider's runtime
    /// matches the `{ecosystem}-{package}` naming convention.
    ///
    /// This enables `vx cargo:audit` to be routed directly to the `cargo-audit` provider's
    /// pre-compiled binary instead of falling back to `cargo install audit`.
    pub fn get_runtime_for_ecosystem_package(
        &self,
        ecosystem: &str,
        package: &str,
    ) -> Option<&str> {
        // Construct the conventional runtime name: `{ecosystem}-{package}` (e.g. `cargo-audit`)
        let candidate = format!("{}-{}", ecosystem, package);
        self.handles.values().find_map(|handle| {
            let runtimes = handle.runtime_metas();
            runtimes.iter().find_map(|r| {
                if r.name.eq_ignore_ascii_case(&candidate)
                    || r.aliases
                        .iter()
                        .any(|a| a.eq_ignore_ascii_case(&candidate))
                {
                    Some(r.name.as_str())
                } else {
                    None
                }
            })
        })
    }
}

/// Global lazy-initialized ProviderHandleRegistry
///
/// Populated at startup by registering all built-in providers.
pub static GLOBAL_REGISTRY: once_cell::sync::Lazy<tokio::sync::RwLock<ProviderHandleRegistry>> =
    once_cell::sync::Lazy::new(|| tokio::sync::RwLock::new(ProviderHandleRegistry::new()));

/// Get a reference to the global ProviderHandleRegistry
pub async fn global_registry() -> tokio::sync::RwLockReadGuard<'static, ProviderHandleRegistry> {
    GLOBAL_REGISTRY.read().await
}

/// Get a mutable reference to the global ProviderHandleRegistry
pub async fn global_registry_mut() -> tokio::sync::RwLockWriteGuard<'static, ProviderHandleRegistry>
{
    GLOBAL_REGISTRY.write().await
}

// ---------------------------------------------------------------------------
// Version compatibility check
// ---------------------------------------------------------------------------

/// Check whether the running vx version satisfies the provider's `vx_version` requirement.
///
/// If the provider declares `vx_version = ">=0.7.0"` in its `provider.star`, this function
/// parses the constraint and compares it against the current vx binary version
/// (`CARGO_PKG_VERSION`).
///
/// # Behavior
/// - If no constraint is declared (`vx_version_req` is `None`), this is a no-op.
/// - If the constraint is satisfied, this is a no-op.
/// - If the constraint is **not** satisfied, a `warn!` log is emitted.
///   The provider is still loaded (graceful degradation) so that older vx versions
///   can continue to work with providers that have been updated for newer APIs.
/// - If the constraint string is malformed, a `warn!` log is emitted and loading continues.
fn check_vx_version_requirement(meta: &ProviderMeta, provider_name: &str) {
    let Some(req_str) = &meta.vx_version_req else {
        return;
    };

    let current_version_str = env!("CARGO_PKG_VERSION");

    // Parse the current vx version
    let current = match semver::Version::parse(current_version_str) {
        Ok(v) => v,
        Err(e) => {
            warn!(
                provider = %provider_name,
                vx_version = %current_version_str,
                "Failed to parse current vx version as semver: {e}"
            );
            return;
        }
    };

    // Parse the requirement string
    let req = match semver::VersionReq::parse(req_str) {
        Ok(r) => r,
        Err(e) => {
            warn!(
                provider = %provider_name,
                vx_version_req = %req_str,
                "Provider declares an invalid vx_version constraint '{req_str}': {e}"
            );
            return;
        }
    };

    if !req.matches(&current) {
        warn!(
            provider = %provider_name,
            vx_version = %current_version_str,
            vx_version_req = %req_str,
            "Provider '{provider_name}' requires vx {req_str}, but the current version is {current_version_str}. \
             Some features may not work correctly. Consider upgrading vx."
        );
    }
}
