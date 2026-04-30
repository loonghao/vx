//! StarlarkProvider - The main interface for loading and executing Starlark provider scripts.
//!
//! This module implements the core functionality for:
//! - Loading and parsing provider.star files
//! - Executing provider functions
//! - Incremental analysis caching (content-hash based, Buck2-inspired)
//! - Providing a trait-based interface compatible with vx's Provider system
//!
//! # Module structure
//!
//! - [`types`]   — Type definitions (InstallLayout, PostExtractAction, etc.)
//! - [`cache`]   — Incremental analysis cache
//! - [`versions`] — Version fetching and JSON transform strategies
//! - [`execute`] — execute_install / execute_download_url / etc.
//! - [`hooks`]   — Hook action parsing (post_extract, pre_run)
//! - [`store`]   — Store path query functions (store_root, get_execute_path, post_install)

pub mod bridge;
pub mod builder;
mod cache;
mod execute;
mod hooks;
mod store;
pub mod types;
pub mod version_cache;
mod versions;

use crate::context::{InstallResult, ProviderContext, VersionInfo};
use crate::engine::{FrozenProviderInfo, StarlarkEngine};
use crate::error::{Error, Result};
use crate::sandbox::SandboxConfig;
pub use bridge::{
    make_download_url_fn, make_fetch_versions_fn, make_install_layout_fn,
    make_version_info_fn_owned,
};
pub use builder::{build_runtimes, create_provider};
use cache::{ANALYSIS_CACHE, AnalysisCacheEntry, sha256_bytes};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::SystemTime;
use tracing::{debug, info};
pub use types::{
    EnvOp, InstallLayout, PostExtractAction, PreRunAction, ProviderMeta, RuntimeMeta,
    VersionInfoResult, apply_env_ops, has_starlark_provider, is_starlark_provider,
};

/// A loaded Starlark provider
#[derive(Debug, Clone)]
pub struct StarlarkProvider {
    /// Path to the provider script
    pub(super) script_path: PathBuf,

    /// Provider metadata
    pub(super) meta: ProviderMeta,

    /// Runtime definitions
    pub(super) runtimes: Vec<RuntimeMeta>,

    /// Sandbox configuration
    pub(super) sandbox: SandboxConfig,

    /// VX home directory
    pub(super) vx_home: PathBuf,

    /// Cached script content (for engine execution)
    pub(super) script_content: Arc<String>,

    /// SHA256 hash of the script content (for incremental analysis cache)
    pub(super) script_hash: [u8; 32],
}

impl StarlarkProvider {
    // ── Constructors ──────────────────────────────────────────────────────────

    /// Load a Starlark provider from a file
    ///
    /// Uses content-hash-based incremental analysis cache (Buck2-inspired):
    /// 1. Read the script content and compute its SHA256 hash
    /// 2. Check the analysis cache by content hash (not file path)
    /// 3. On cache hit: reuse the frozen ProviderInfo without re-executing
    /// 4. On cache miss: parse metadata, cache the result
    pub async fn load(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().to_path_buf();

        if !path.exists() {
            return Err(Error::ScriptNotFound(path));
        }

        let content = std::fs::read_to_string(&path)?;
        debug!("Loading Starlark provider from: {:?}", path);

        let script_hash = sha256_bytes(content.as_bytes());

        {
            let cache = ANALYSIS_CACHE.read().await;
            if let Some(entry) = cache.get(&script_hash) {
                debug!(path = %path.display(), "Using cached analysis result (content hash match)");
                let vx_home = Self::resolve_vx_home();
                return Ok(Self {
                    script_path: path,
                    meta: entry.meta.clone(),
                    runtimes: entry.runtimes.clone(),
                    sandbox: SandboxConfig::default(),
                    vx_home,
                    script_content: Arc::new(content),
                    script_hash: entry.script_hash,
                });
            }
        }

        let (meta, runtimes) = Self::parse_metadata(&content)?;
        let vx_home = Self::resolve_vx_home();

        let provider = Self {
            script_path: path.clone(),
            meta: meta.clone(),
            runtimes: runtimes.clone(),
            sandbox: SandboxConfig::default(),
            vx_home,
            script_content: Arc::new(content),
            script_hash,
        };

        {
            let mut cache = ANALYSIS_CACHE.write().await;
            cache.insert(
                script_hash,
                AnalysisCacheEntry {
                    script_hash,
                    frozen_info: FrozenProviderInfo {
                        versions_url: None,
                        download_url: None,
                        env_template: HashMap::new(),
                        metadata: HashMap::new(),
                    },
                    meta,
                    runtimes,
                    cached_at: SystemTime::now(),
                },
            );
        }

        info!("Loaded Starlark provider: {}", provider.meta.name);
        Ok(provider)
    }

    /// Load a provider with custom sandbox configuration
    pub async fn load_with_sandbox(path: impl AsRef<Path>, sandbox: SandboxConfig) -> Result<Self> {
        let mut provider = Self::load(path).await?;
        provider.sandbox = sandbox;
        Ok(provider)
    }

    /// Create a provider from in-memory script content (no filesystem access).
    ///
    /// This is the preferred entry point for built-in providers that embed their
    /// `provider.star` at compile time via `include_str!`.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// pub const PROVIDER_STAR: &str = include_str!("../provider.star");
    /// let provider = StarlarkProvider::from_content("just", PROVIDER_STAR).await?;
    /// ```
    pub async fn from_content(name: impl Into<String>, content: impl Into<String>) -> Result<Self> {
        let name = name.into();
        let content = content.into();
        let virtual_path = PathBuf::from(format!("<builtin:{}>", name));
        let script_hash = sha256_bytes(content.as_bytes());

        {
            let cache = ANALYSIS_CACHE.read().await;
            if let Some(entry) = cache.get(&script_hash) {
                debug!(provider = %name, "Using cached analysis result for built-in provider (content hash match)");
                let vx_home = Self::resolve_vx_home();
                return Ok(Self {
                    script_path: virtual_path,
                    meta: entry.meta.clone(),
                    runtimes: entry.runtimes.clone(),
                    sandbox: SandboxConfig::default(),
                    vx_home,
                    script_content: Arc::new(content),
                    script_hash: entry.script_hash,
                });
            }
        }

        let (meta, runtimes) = Self::parse_metadata(&content)?;
        let vx_home = Self::resolve_vx_home();

        let provider = Self {
            script_path: virtual_path,
            meta: meta.clone(),
            runtimes: runtimes.clone(),
            sandbox: SandboxConfig::default(),
            vx_home,
            script_content: Arc::new(content),
            script_hash,
        };

        {
            let mut cache = ANALYSIS_CACHE.write().await;
            cache.insert(
                script_hash,
                AnalysisCacheEntry {
                    script_hash,
                    frozen_info: FrozenProviderInfo {
                        versions_url: None,
                        download_url: None,
                        env_template: HashMap::new(),
                        metadata: HashMap::new(),
                    },
                    meta,
                    runtimes,
                    cached_at: SystemTime::now(),
                },
            );
        }

        debug!("Loaded built-in Starlark provider: {}", provider.meta.name);
        Ok(provider)
    }

    // ── Accessors ─────────────────────────────────────────────────────────────

    pub fn name(&self) -> &str {
        &self.meta.name
    }
    pub fn description(&self) -> &str {
        &self.meta.description
    }
    pub fn meta(&self) -> &ProviderMeta {
        &self.meta
    }
    pub fn runtimes(&self) -> &[RuntimeMeta] {
        &self.runtimes
    }
    pub fn script_path(&self) -> &Path {
        &self.script_path
    }

    pub fn script_hash(&self) -> &[u8; 32] {
        &self.script_hash
    }

    pub fn script_hash_hex(&self) -> String {
        self.script_hash
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect()
    }

    // ── Public provider functions ─────────────────────────────────────────────

    /// Call the `fetch_versions` function
    pub async fn fetch_versions(&self) -> Result<Vec<VersionInfo>> {
        self.fetch_versions_for_runtime(None).await
    }

    /// Call the `fetch_versions` function for a specific runtime within a multi-runtime provider.
    ///
    /// For providers that define multiple runtimes (e.g. shell-tools with starship/atuin/yazi),
    /// the `fetch_versions(ctx)` function dispatches on `ctx.runtime_name` to fetch versions
    /// from the correct source. Pass `None` to use the provider name as the runtime name.
    pub async fn fetch_versions_for_runtime(
        &self,
        runtime_name: Option<&str>,
    ) -> Result<Vec<VersionInfo>> {
        let mut ctx = ProviderContext::new(&self.meta.name, self.vx_home.clone())
            .with_description(&self.meta.description)
            .with_sandbox(self.sandbox.clone());
        if let Some(name) = runtime_name {
            ctx = ctx.with_runtime_name(name);
        }
        self.execute_fetch_versions(&ctx).await
    }

    /// Call the `install` function
    pub async fn install(&self, version: &str) -> Result<InstallResult> {
        let ctx = ProviderContext::new(&self.meta.name, self.vx_home.clone())
            .with_description(&self.meta.description)
            .with_sandbox(self.sandbox.clone())
            .with_version(version);
        self.execute_install(&ctx, version).await
    }

    /// Call the `environment` function (canonical name in provider.star)
    ///
    /// Returns a list of [`EnvOp`]s describing how to set up the environment.
    /// Use [`apply_env_ops`] to apply them to a mutable env map.
    ///
    /// Also tries `prepare_environment` as a fallback for forward compatibility.
    pub async fn environment(&self, version: &str) -> Result<Vec<EnvOp>> {
        let ctx = ProviderContext::new(&self.meta.name, self.vx_home.clone())
            .with_description(&self.meta.description)
            .with_sandbox(self.sandbox.clone())
            .with_version(version);
        self.execute_prepare_environment(&ctx, version).await
    }

    /// Alias for [`environment`] — kept for backward compatibility.
    #[inline]
    pub async fn prepare_environment(&self, version: &str) -> Result<Vec<EnvOp>> {
        self.environment(version).await
    }

    // -----------------------------------------------------------------------
    // RFC 0040: Toolchain Version Indirection
    // -----------------------------------------------------------------------

    /// Call `version_info(ctx, user_version)` if defined in provider.star.
    ///
    /// Returns `None` if the provider uses 1:1 version mapping (the default).
    /// Returns `Some(VersionInfoResult)` for toolchain-managed tools like Rust.
    pub async fn version_info(&self, user_version: &str) -> Result<Option<VersionInfoResult>> {
        let ctx = ProviderContext::new(&self.meta.name, self.vx_home.clone())
            .with_description(&self.meta.description)
            .with_sandbox(self.sandbox.clone())
            .with_version(user_version);
        self.execute_version_info(&ctx, user_version).await
    }

    /// Call the `download_url` function
    pub async fn download_url(&self, version: &str) -> Result<Option<String>> {
        self.download_url_for_runtime(version, None).await
    }

    /// Call the `download_url` function for a specific runtime within a multi-runtime provider.
    ///
    /// Passes `ctx.runtime_name` so that providers can dispatch to the correct
    /// download URL for each runtime (e.g. different GitHub repos for yazi vs starship).
    ///
    /// For providers that depend on `ctx.version_date` (e.g. python-build-standalone),
    /// this method will automatically trigger a `fetch_versions` call to populate the
    /// version cache if `lookup_version_date` returns `None` (cache miss). This ensures
    /// that the build tag is available even on first install or after cache expiry.
    pub async fn download_url_for_runtime(
        &self,
        version: &str,
        runtime_name: Option<&str>,
    ) -> Result<Option<String>> {
        // Look up the build tag (date) for this version from the version cache.
        // This is needed by providers like python-build-standalone where the
        // download URL requires a date-based release tag (e.g. "20240107").
        // The build tag is stored in VersionInfo.date by transform_python_build_standalone.
        let mut version_date = self.lookup_version_date(version, runtime_name).await;

        // If version_date is not in cache, trigger a fetch_versions call to populate it.
        // This handles first-install scenarios, cache expiry, and provider.star updates
        // where the version cache is empty but download_url needs the build tag.
        if version_date.is_none() {
            debug!(
                provider = %self.meta.name,
                version = %version,
                "version_date not in cache, triggering fetch_versions to populate"
            );
            let mut fetch_ctx = ProviderContext::new(&self.meta.name, self.vx_home.clone())
                .with_description(&self.meta.description)
                .with_sandbox(self.sandbox.clone());
            if let Some(name) = runtime_name {
                fetch_ctx = fetch_ctx.with_runtime_name(name);
            }
            // Ignore errors — fetch_versions may fail (network), but we still
            // want to try download_url with whatever cache state exists.
            if let Err(e) = self.execute_fetch_versions(&fetch_ctx).await {
                debug!(
                    provider = %self.meta.name,
                    error = %e,
                    "fetch_versions failed during version_date recovery, continuing without"
                );
            } else {
                // Retry the lookup now that the cache should be populated
                version_date = self.lookup_version_date(version, runtime_name).await;
            }
        }

        let mut ctx = ProviderContext::new(&self.meta.name, self.vx_home.clone())
            .with_description(&self.meta.description)
            .with_sandbox(self.sandbox.clone())
            .with_version(version);

        if let Some(date) = version_date {
            ctx = ctx.with_version_date(date);
        }
        if let Some(name) = runtime_name {
            ctx = ctx.with_runtime_name(name);
        }

        self.execute_download_url(&ctx, version).await
    }

    /// Call the `install_layout` function and resolve the returned descriptor
    pub async fn install_layout(&self, version: &str) -> Result<Option<InstallLayout>> {
        self.install_layout_for_runtime(version, None).await
    }

    /// Call the `install_layout` function for a specific runtime within a multi-runtime provider.
    pub async fn install_layout_for_runtime(
        &self,
        version: &str,
        runtime_name: Option<&str>,
    ) -> Result<Option<InstallLayout>> {
        let mut ctx = ProviderContext::new(&self.meta.name, self.vx_home.clone())
            .with_description(&self.meta.description)
            .with_sandbox(self.sandbox.clone())
            .with_version(version);
        if let Some(name) = runtime_name {
            ctx = ctx.with_runtime_name(name);
        }
        self.execute_install_layout(&ctx, version).await
    }

    /// Call the `install_layout` function and return the raw JSON dict.
    ///
    /// Unlike `install_layout()`, this method does **not** try to parse the
    /// returned value into a typed `InstallLayout` enum.  It is used as a
    /// fallback when the Starlark function returns a plain dict without a
    /// `__type` field (e.g. `{ "source_name": ..., "target_name": ... }`).
    pub async fn install_layout_raw(&self, version: &str) -> Result<Option<serde_json::Value>> {
        self.install_layout_raw_for_runtime(version, None).await
    }

    /// Call the `install_layout` function (raw) for a specific runtime within a multi-runtime provider.
    pub async fn install_layout_raw_for_runtime(
        &self,
        version: &str,
        runtime_name: Option<&str>,
    ) -> Result<Option<serde_json::Value>> {
        let mut ctx = ProviderContext::new(&self.meta.name, self.vx_home.clone())
            .with_description(&self.meta.description)
            .with_sandbox(self.sandbox.clone())
            .with_version(version);
        if let Some(name) = runtime_name {
            ctx = ctx.with_runtime_name(name);
        }
        self.execute_install_layout_raw(&ctx, version).await
    }

    /// Call the `post_extract` function and resolve the returned action list
    pub async fn post_extract(
        &self,
        version: &str,
        install_dir: &Path,
    ) -> Result<Vec<PostExtractAction>> {
        let ctx = ProviderContext::new(&self.meta.name, self.vx_home.clone())
            .with_description(&self.meta.description)
            .with_sandbox(self.sandbox.clone())
            .with_version(version);
        self.execute_post_extract(&ctx, version, install_dir).await
    }

    /// Call the `pre_run` function and resolve the returned action list
    pub async fn pre_run(&self, args: &[String], executable: &Path) -> Result<Vec<PreRunAction>> {
        let ctx = ProviderContext::new(&self.meta.name, self.vx_home.clone())
            .with_description(&self.meta.description)
            .with_sandbox(self.sandbox.clone());
        self.execute_pre_run(&ctx, args, executable).await
    }

    /// Call the `deps(ctx, version)` function from provider.star.
    ///
    /// Returns a list of raw JSON dependency descriptors. Each descriptor has:
    /// - `runtime`: the runtime name (e.g. "git")
    /// - `version`: version constraint (e.g. "*", ">=2.0")
    /// - `optional`: whether the dependency is optional
    /// - `reason`: human-readable reason for the dependency
    ///
    /// Returns an empty list if `deps()` is not defined in provider.star.
    pub async fn deps(&self, version: &str) -> Result<Vec<serde_json::Value>> {
        self.deps_for_runtime(version, None).await
    }

    /// Call the `deps(ctx, version)` function for a specific runtime within a multi-runtime provider.
    pub async fn deps_for_runtime(
        &self,
        version: &str,
        runtime_name: Option<&str>,
    ) -> Result<Vec<serde_json::Value>> {
        let mut ctx = ProviderContext::new(&self.meta.name, self.vx_home.clone())
            .with_description(&self.meta.description)
            .with_sandbox(self.sandbox.clone())
            .with_version(version);
        if let Some(name) = runtime_name {
            ctx = ctx.with_runtime_name(name);
        }
        self.execute_deps(&ctx, version).await
    }

    /// Call the `uninstall` function in provider.star (if defined).
    ///
    /// Returns:
    /// - `Ok(true)`  — provider.star handled the uninstall (custom logic ran)
    /// - `Ok(false)` — `uninstall()` not defined; caller should use default logic
    /// - `Err(_)`    — provider.star returned an error
    ///
    /// This enables per-tool customization of uninstall behavior while keeping
    /// the default (directory removal) as a safe fallback in `ProviderHandle`.
    pub async fn uninstall(&self, version: &str) -> Result<bool> {
        let ctx = ProviderContext::new(&self.meta.name, self.vx_home.clone())
            .with_description(&self.meta.description)
            .with_sandbox(self.sandbox.clone())
            .with_version(version);
        self.execute_uninstall(&ctx, version).await
    }

    // ── Cache management ──────────────────────────────────────────────────────

    /// Clear the incremental analysis cache
    pub async fn clear_cache() {
        let mut cache = ANALYSIS_CACHE.write().await;
        cache.clear();
        info!("Cleared Starlark incremental analysis cache");
    }

    /// Get cache statistics: (entry_count, total_runtimes)
    pub async fn cache_stats() -> (usize, usize) {
        let cache = ANALYSIS_CACHE.read().await;
        (cache.len(), 0)
    }

    /// Invalidate a specific cache entry by script content hash
    pub async fn invalidate_cache_entry(script_hash: &[u8; 32]) {
        let mut cache = ANALYSIS_CACHE.write().await;
        if cache.remove(script_hash).is_some() {
            debug!(
                "Invalidated analysis cache entry for hash {:?}",
                &script_hash[..4]
            );
        }
    }

    // ── Internal helpers ──────────────────────────────────────────────────────



    /// Look up the build tag (date) for a specific version from the version cache.
    ///
    /// Returns `Some(date)` if the version was found in cache and has a `date` field.
    /// Returns `None` if the version is not cached or has no date.
    ///
    /// This is used by `download_url` to pass the build tag to providers like
    /// python-build-standalone, where the download URL requires a date-based
    /// release tag (e.g. "20240107") that is stored in `VersionInfo.date`.
    ///
    /// **Cache key**: Uses `provider_name/runtime_name` format to match the key
    /// used by `execute_fetch_versions`. Without this, multi-runtime providers
    /// would write to `python/python` but read from `python`, causing a miss.
    ///
    /// **Fallback strategy**: If no exact version match is found, this method
    /// looks for the most recent build tag from the same minor version series
    /// (e.g. for "3.12.13", it will try any "3.12.x" entry). This handles the
    /// common case where a new patch release exists on the python-build-standalone
    /// server but hasn't been added to the wellknown fallback list yet — the
    /// build tag from the same minor series is almost always valid because
    /// python-build-standalone releases typically include all active patch versions.
    async fn lookup_version_date(
        &self,
        version: &str,
        runtime_name: Option<&str>,
    ) -> Option<String> {
        let cache = version_cache::global_version_cache();
        let hash_hex = self.script_hash_hex();
        let provider_name = &self.meta.name;

        // Build cache key matching execute_fetch_versions format
        let cache_key = match runtime_name {
            Some(rt) if !rt.is_empty() => format!("{}/{}", provider_name, rt),
            _ => provider_name.clone(),
        };

        // Try both cache_key variants: with runtime_name and without.
        // This handles cases where fetch_versions was called with or without runtime_name.
        let cached = cache.get(&cache_key, &hash_hex).await.or_else(|| {
            // Synchronous fallback: if the runtime-prefixed key didn't match,
            // try the plain provider name (covers single-runtime providers and
            // cases where fetch_versions was called without runtime_name).
            if cache_key != *provider_name {
                // We can't do async inside or_else, so we use try_blocking_get
                // Unfortunately the cache is async. As a workaround, we return None
                // and let the caller handle the full fallback flow.
                None
            } else {
                None
            }
        });

        let cached = match cached {
            Some(c) => c,
            None => {
                // Also try the plain provider name if we used a runtime-prefixed key
                if cache_key != *provider_name {
                    cache.get(provider_name, &hash_hex).await?
                } else {
                    return None;
                }
            }
        };

        // 1. Try exact version match first
        if let Some(date) = cached
            .iter()
            .find(|v| v.version == version)
            .and_then(|v| v.date.clone())
        {
            return Some(date);
        }

        // 2. Fallback: find the most recent build tag from the same minor series.
        //    For version "3.12.13", extract prefix "3.12." and find the entry with
        //    the highest build tag (date) among all "3.12.x" versions.
        let minor_prefix = extract_minor_prefix(version)?;
        debug!(
            provider = %self.meta.name,
            version = %version,
            minor_prefix = %minor_prefix,
            "Exact version_date match not found, trying minor series fallback"
        );

        let best = cached
            .iter()
            .filter(|v| v.version.starts_with(&minor_prefix) && v.date.is_some())
            .max_by(|a, b| {
                let da = a.date.as_deref().unwrap_or("");
                let db = b.date.as_deref().unwrap_or("");
                da.cmp(db)
            });

        if let Some(entry) = best {
            debug!(
                provider = %self.meta.name,
                version = %version,
                fallback_version = %entry.version,
                fallback_date = ?entry.date,
                "Using build tag from same minor series as fallback"
            );
            return entry.date.clone();
        }

        None
    }

    fn resolve_vx_home() -> PathBuf {
        vx_paths::VxPaths::new()
            .map(|p| p.base_dir)
            .unwrap_or_else(|_| dirs::home_dir().unwrap_or_default().join(".vx"))
    }

    /// Parse metadata from a Starlark script
    ///
    /// Looks for:
    /// - `name()` function
    /// - `description()` function
    /// - `runtimes` variable
    pub(super) fn parse_metadata(content: &str) -> Result<(ProviderMeta, Vec<RuntimeMeta>)> {
        // Use the static metadata parser (StarMetadata) to extract name/description/etc.
        // This handles both `name = "..."` (top-level variable) and
        // `def name(): return "..."` (function return) formats.
        let star_meta = vx_star_metadata::StarMetadata::parse(content);

        let lint_name = star_meta
            .name
            .as_deref()
            .unwrap_or("provider.star")
            .to_string();

        let meta = ProviderMeta {
            name: star_meta
                .name
                .clone()
                .unwrap_or_else(|| "unknown".to_string()),
            description: star_meta.description.unwrap_or_default(),
            version: "1.0.0".to_string(),
            homepage: star_meta.homepage,
            repository: star_meta.repository,
            platforms: star_meta.platforms.map(|os_list| {
                let mut map = std::collections::HashMap::new();
                map.insert("os".to_string(), os_list);
                map
            }),
            package_alias: star_meta.package_alias.map(|(ecosystem, package)| {
                crate::provider::types::PackageAlias {
                    ecosystem,
                    package,
                    executable: None,
                }
            }),
            package_prefixes: star_meta.package_prefixes,
            vx_version_req: star_meta.vx_version,
        };

        let mut runtimes: Vec<RuntimeMeta> = Vec::new();

        // Try to parse the `runtimes` list variable via the Starlark engine.
        // Use the provider name for linting so warnings point to the right provider.
        let virtual_path = PathBuf::from(lint_name);
        let engine = StarlarkEngine::new();
        if let Ok(Some(runtimes_json)) = engine.get_variable(&virtual_path, content, "runtimes")
            && let Some(arr) = runtimes_json.as_array()
        {
            for item in arr {
                let name = item
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or(&meta.name)
                    .to_string();
                let description = item
                    .get("description")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let executable = item
                    .get("executable")
                    .and_then(|v| v.as_str())
                    .unwrap_or(&name)
                    .to_string();
                let aliases: Vec<String> = item
                    .get("aliases")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str().map(|s| s.to_string()))
                            .collect()
                    })
                    .unwrap_or_default();
                let priority = item.get("priority").and_then(|v| v.as_u64()).unwrap_or(100) as u32;
                let command_prefix: Vec<String> = item
                    .get("command_prefix")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str().map(|s| s.to_string()))
                            .collect()
                    })
                    .unwrap_or_default();
                let system_paths: Vec<String> = item
                    .get("system_paths")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str().map(|s| s.to_string()))
                            .collect()
                    })
                    .unwrap_or_default();

                // Parse test_commands from the runtimes list entry.
                // Each entry is a dict with at minimum a "command" key.
                // Example in provider.star:
                //   "test_commands": [
                //       {"command": "{executable} --version", "name": "version_check"},
                //       {"command": "where {executable}", "name": "where_check"},
                //   ]
                let test_commands: Vec<crate::provider::types::TestCommandMeta> = item
                    .get("test_commands")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|tc| {
                                let command = tc.get("command")?.as_str()?.to_string();
                                // Parse check_type (default: "command")
                                let check_type = tc
                                    .get("check_type")
                                    .and_then(|v| v.as_str())
                                    .map(|s| match s {
                                        "check_path" => {
                                            crate::provider::types::TestCheckType::CheckPath
                                        }
                                        "check_not_path" => {
                                            crate::provider::types::TestCheckType::CheckNotPath
                                        }
                                        "check_env" => {
                                            crate::provider::types::TestCheckType::CheckEnv
                                        }
                                        "check_not_env" => {
                                            crate::provider::types::TestCheckType::CheckNotEnv
                                        }
                                        "check_file" => {
                                            crate::provider::types::TestCheckType::CheckFile
                                        }
                                        _ => crate::provider::types::TestCheckType::Command,
                                    })
                                    .unwrap_or_default();
                                Some(crate::provider::types::TestCommandMeta {
                                    command,
                                    check_type,
                                    expect_success: tc
                                        .get("expect_success")
                                        .and_then(|v| v.as_bool())
                                        .unwrap_or(true),
                                    expected_output: tc
                                        .get("expected_output")
                                        .and_then(|v| v.as_str())
                                        .map(|s| s.to_string()),
                                    name: tc
                                        .get("name")
                                        .and_then(|v| v.as_str())
                                        .map(|s| s.to_string()),
                                    timeout_ms: tc.get("timeout_ms").and_then(|v| v.as_u64()),
                                })
                            })
                            .collect()
                    })
                    .unwrap_or_default();

                runtimes.push(RuntimeMeta {
                    name,
                    description,
                    executable,
                    aliases,
                    priority,
                    command_prefix,
                    system_paths,
                    test_commands,
                    install_deps: item
                        .get("install_deps")
                        .and_then(|v| v.as_array())
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                                .collect()
                        })
                        .unwrap_or_default(),
                    bundled_with: item
                        .get("bundled_with")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                });
            }
        }

        if runtimes.is_empty() {
            runtimes.push(RuntimeMeta {
                name: meta.name.clone(),
                description: meta.description.clone(),
                executable: meta.name.clone(),
                aliases: vec![],
                priority: 100,
                command_prefix: vec![],
                system_paths: vec![],
                test_commands: vec![],
                install_deps: vec![],
                bundled_with: None,
            });
        }

        Ok((meta, runtimes))
    }
}

/// Extract the minor version prefix from a version string.
///
/// For "3.12.13" returns "3.12.", for "3.11.0" returns "3.11.".
/// Returns `None` if the version doesn't have at least two dot-separated components.
fn extract_minor_prefix(version: &str) -> Option<String> {
    let mut parts = version.splitn(3, '.');
    let major = parts.next()?;
    let minor = parts.next()?;
    Some(format!("{}.{}.", major, minor))
}
