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
use crate::provider::{
    InstallLayout, PostExtractAction, ProviderMeta, RuntimeMeta, StarlarkProvider,
};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use vx_paths::VxPaths;

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
        let paths = VxPaths::new()
            .map_err(|e| Error::EvalError(format!("Failed to initialize VxPaths: {e}")))?;

        Ok(Self {
            name,
            star: Arc::new(star),
            paths: Arc::new(paths),
        })
    }

    /// Load a ProviderHandle from a file path (for user-defined providers)
    pub async fn load(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let star = StarlarkProvider::load(path).await?;
        let name = star.name().to_string();
        let paths = VxPaths::new()
            .map_err(|e| Error::EvalError(format!("Failed to initialize VxPaths: {e}")))?;

        Ok(Self {
            name,
            star: Arc::new(star),
            paths: Arc::new(paths),
        })
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
        let versions = self.star.fetch_versions().await?;

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

    /// Get list of installed versions by scanning the store directory
    pub fn installed_versions(&self) -> Vec<String> {
        let store_root = self.store_root();
        if !store_root.exists() {
            return vec![];
        }

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

        // Sort versions in descending order (newest first)
        versions.sort_by(|a, b| b.cmp(a));
        versions
    }

    /// Check if a specific version is installed
    pub fn is_installed(&self, version: &str) -> bool {
        self.store_root().join(version).exists()
    }

    // ── Path Queries ───────────────────────────────────────────────────────

    /// Get the store root directory for this provider (sync, convention-based)
    ///
    /// Convention-based default: `{vx_home}/store/{provider_name}`
    ///
    /// Corresponds to `vx where <tool>` (no version)
    pub fn store_root(&self) -> PathBuf {
        self.paths.store_dir.join(&self.name)
    }

    /// Get the store root directory, preferring provider.star::store_root (async)
    ///
    /// Calls `store_root(ctx)` in provider.star if defined; falls back to
    /// the convention-based path `{vx_home}/store/{provider_name}`.
    pub async fn store_root_from_star(&self) -> PathBuf {
        if let Ok(Some(template)) = self.star.call_store_root().await {
            // Replace {vx_home} placeholder with the actual vx home directory
            let resolved = template.replace("{vx_home}", &self.paths.base_dir.to_string_lossy());
            return PathBuf::from(resolved);
        }
        // Convention fallback
        self.paths.store_dir.join(&self.name)
    }

    /// Get the executable path for a specific version
    ///
    /// Calls `get_execute_path(ctx, version)` in provider.star if defined;
    /// falls back to convention-based path scanning.
    ///
    /// Corresponds to `vx where <tool>@<version>`
    pub async fn get_execute_path(&self, version: &str) -> Option<PathBuf> {
        // Try provider.star::get_execute_path first
        if let Ok(Some(template)) = self.star.call_get_execute_path(version).await {
            let store_root = self.store_root_from_star().await;
            let install_dir = store_root.join(version);
            let resolved = template
                .replace("{install_dir}", &install_dir.to_string_lossy())
                .replace("{vx_home}", &self.paths.base_dir.to_string_lossy());
            return Some(PathBuf::from(resolved));
        }
        // Convention fallback
        self.convention_execute_path(version)
    }

    /// Get the executable path for the latest installed version
    ///
    /// Corresponds to `vx where <tool>` (when versions are installed)
    pub async fn get_latest_execute_path(&self) -> Option<PathBuf> {
        let versions = self.installed_versions();
        let latest = versions.first()?.clone();
        self.get_execute_path(&latest).await
    }

    // ── Installation ───────────────────────────────────────────────────────

    /// Get download URL for a specific version
    ///
    /// Delegates to provider.star::download_url
    pub async fn download_url(&self, version: &str) -> Result<Option<String>> {
        self.star.download_url(version).await
    }

    /// Get install layout for a specific version
    ///
    /// Delegates to provider.star::install_layout
    pub async fn install_layout(&self, version: &str) -> Result<Option<InstallLayout>> {
        self.star.install_layout(version).await
    }

    /// Get post-install operations for a specific version
    ///
    /// Calls `post_install(ctx, version, install_dir)` in provider.star (RFC-0037).
    /// Falls back to `post_extract()` for backward compatibility.
    pub async fn post_install(
        &self,
        version: &str,
        install_dir: &Path,
    ) -> Result<Vec<PostInstallOps>> {
        // Try provider.star::post_install first (RFC-0037)
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

    /// Get environment variables for a specific version
    ///
    /// Delegates to provider.star::environment
    pub async fn environment(
        &self,
        version: &str,
        _install_dir: &Path,
    ) -> Result<HashMap<String, String>> {
        self.star.prepare_environment(version).await
    }

    /// Get dependencies for a specific version
    ///
    /// Returns empty list; future work may call provider.star::deps()
    pub async fn deps(&self, _version: &str) -> Result<Vec<String>> {
        Ok(vec![])
    }

    // ── Internal Helpers ───────────────────────────────────────────────────

    /// Convention-based executable path: scan version dir for known executables
    fn convention_execute_path(&self, version: &str) -> Option<PathBuf> {
        let version_dir = self.store_root().join(version);
        if !version_dir.exists() {
            return None;
        }

        // Get executable name from provider metadata
        let exe_name = self
            .star
            .runtimes()
            .first()
            .map(|r| r.executable.clone())
            .unwrap_or_else(|| self.name.clone());

        // Try common locations
        let candidates = vec![
            // Direct in version dir
            version_dir.join(vx_paths::with_executable_extension(&exe_name)),
            version_dir.join(&exe_name),
            // In bin/ subdirectory
            version_dir
                .join("bin")
                .join(vx_paths::with_executable_extension(&exe_name)),
            version_dir.join("bin").join(&exe_name),
        ];

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

    /// Register a provider from a file path (user-defined providers)
    pub async fn register_from_file(&mut self, path: &Path) -> Result<()> {
        let handle = ProviderHandle::load(path).await?;
        self.insert(handle);
        Ok(())
    }

    /// Insert a handle and register all its runtime aliases
    fn insert(&mut self, handle: ProviderHandle) {
        let canonical = handle.name().to_string();

        // Register aliases from all runtimes
        for runtime in handle.runtime_metas() {
            for alias in &runtime.aliases {
                self.aliases
                    .entry(alias.clone())
                    .or_insert_with(|| canonical.clone());
            }
            // Also register runtime name as alias if different from provider name
            if runtime.name != canonical {
                self.aliases
                    .entry(runtime.name.clone())
                    .or_insert_with(|| canonical.clone());
            }
        }

        self.handles.insert(canonical, Arc::new(handle));
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
}

/// Global lazy-initialized ProviderHandleRegistry
///
/// Populated at startup by registering all built-in providers.
static GLOBAL_REGISTRY: once_cell::sync::Lazy<tokio::sync::RwLock<ProviderHandleRegistry>> =
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
