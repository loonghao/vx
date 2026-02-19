//! StarlarkProvider - The main interface for loading and executing Starlark provider scripts
//!
//! This module implements the core functionality for:
//! - Loading and parsing provider.star files
//! - Executing provider functions
//! - Incremental analysis caching (content-hash based, Buck2-inspired)
//! - Providing a trait-based interface compatible with vx's Provider system

use crate::context::{InstallResult, ProviderContext, VersionInfo};
use crate::engine::{FrozenProviderInfo, StarlarkEngine};
use crate::error::{Error, Result};
use crate::sandbox::SandboxConfig;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Provider metadata parsed from the script
#[derive(Debug, Clone, Deserialize)]
pub struct ProviderMeta {
    /// Provider name
    pub name: String,
    /// Provider description
    #[serde(default)]
    pub description: String,
    /// Provider version
    #[serde(default = "default_version")]
    pub version: String,
    /// Provider homepage
    #[serde(default)]
    pub homepage: Option<String>,
    /// Provider repository
    #[serde(default)]
    pub repository: Option<String>,
    /// Platform constraints (os: ["windows", "linux"])
    #[serde(default)]
    pub platforms: Option<HashMap<String, Vec<String>>>,
}

fn default_version() -> String {
    "1.0.0".to_string()
}

/// Runtime metadata parsed from the script
#[derive(Debug, Clone, Deserialize)]
pub struct RuntimeMeta {
    /// Runtime name
    pub name: String,
    /// Runtime description
    #[serde(default)]
    pub description: String,
    /// Executable name
    pub executable: String,
    /// Aliases
    #[serde(default)]
    pub aliases: Vec<String>,
    /// Priority
    #[serde(default = "default_priority")]
    pub priority: u32,
}

fn default_priority() -> u32 {
    100
}

/// A loaded Starlark provider
#[derive(Debug, Clone)]
pub struct StarlarkProvider {
    /// Path to the provider script
    script_path: PathBuf,

    /// Provider metadata
    meta: ProviderMeta,

    /// Runtime definitions
    runtimes: Vec<RuntimeMeta>,

    /// Sandbox configuration
    sandbox: SandboxConfig,

    /// VX home directory
    vx_home: PathBuf,

    /// Cached script content (for engine execution)
    script_content: Arc<String>,

    /// SHA256 hash of the script content (for incremental analysis cache)
    script_hash: [u8; 32],
}

/// Incremental analysis cache entry (Buck2-inspired content-hash cache)
///
/// Inspired by Buck2's incremental analysis: cache the frozen ProviderInfo
/// keyed by the SHA256 hash of the script content. If the script hasn't
/// changed (same hash), reuse the cached analysis result without re-executing.
#[derive(Debug, Clone)]
struct AnalysisCacheEntry {
    /// SHA256 hash of the provider.star content
    script_hash: [u8; 32],
    /// Frozen analysis result (immutable after analysis phase)
    /// NOTE: Used in Phase 2 when full Starlark execution engine is implemented
    #[allow(dead_code)]
    frozen_info: FrozenProviderInfo,
    /// When this entry was cached
    /// NOTE: Used in Phase 2 for TTL-based cache expiration
    #[allow(dead_code)]
    cached_at: SystemTime,
}

/// Cache for analysis results, keyed by content hash (not file path)
///
/// Using content hash instead of path means:
/// - Same script content → same cache entry (deduplication)
/// - Modified script → new hash → cache miss → re-analysis
/// - File rename/move → same hash → cache hit (no re-analysis needed)
type AnalysisCache = Arc<RwLock<HashMap<[u8; 32], AnalysisCacheEntry>>>;

/// Global incremental analysis cache (content-hash based, Buck2-inspired)
static ANALYSIS_CACHE: once_cell::sync::Lazy<AnalysisCache> =
    once_cell::sync::Lazy::new(|| Arc::new(RwLock::new(HashMap::new())));

/// Compute SHA256 hash of content bytes
fn sha256_bytes(content: &[u8]) -> [u8; 32] {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    // Use a simple but deterministic hash approach
    // In production, this would use sha2 crate for proper SHA256
    // For now, we use a 32-byte representation via multiple hash passes
    let mut result = [0u8; 32];

    // Pass 1: hash the full content
    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    let h1 = hasher.finish();

    // Pass 2: hash with length prefix for better distribution
    let mut hasher2 = DefaultHasher::new();
    (content.len() as u64).hash(&mut hasher2);
    content.hash(&mut hasher2);
    let h2 = hasher2.finish();

    // Pass 3 & 4: hash reversed content for additional entropy
    let mut hasher3 = DefaultHasher::new();
    content
        .iter()
        .rev()
        .cloned()
        .collect::<Vec<u8>>()
        .hash(&mut hasher3);
    let h3 = hasher3.finish();

    let mut hasher4 = DefaultHasher::new();
    h1.hash(&mut hasher4);
    h2.hash(&mut hasher4);
    h3.hash(&mut hasher4);
    let h4 = hasher4.finish();

    // Fill 32 bytes from 4 x u64 hashes
    result[0..8].copy_from_slice(&h1.to_le_bytes());
    result[8..16].copy_from_slice(&h2.to_le_bytes());
    result[16..24].copy_from_slice(&h3.to_le_bytes());
    result[24..32].copy_from_slice(&h4.to_le_bytes());

    result
}

impl StarlarkProvider {
    /// Load a Starlark provider from a file
    ///
    /// Uses content-hash-based incremental analysis cache (Buck2-inspired):
    /// 1. Read the script content and compute its SHA256 hash
    /// 2. Check the analysis cache by content hash (not file path)
    /// 3. On cache hit: reuse the frozen ProviderInfo without re-executing
    /// 4. On cache miss: parse metadata, cache the result
    pub async fn load(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().to_path_buf();

        // Check if file exists
        if !path.exists() {
            return Err(Error::ScriptNotFound(path));
        }

        // Read the script content
        let content = std::fs::read_to_string(&path)?;
        debug!("Loading Starlark provider from: {:?}", path);

        // Compute content hash for incremental analysis cache
        let script_hash = sha256_bytes(content.as_bytes());

        // Check analysis cache by content hash (not path)
        {
            let cache = ANALYSIS_CACHE.read().await;
            if let Some(entry) = cache.get(&script_hash) {
                debug!(
                    path = %path.display(),
                    "Using cached analysis result (content hash match)"
                );
                // Reconstruct provider from cached frozen info
                let vx_home = vx_paths::VxPaths::new()
                    .map(|p| p.base_dir)
                    .unwrap_or_else(|_| dirs::home_dir().unwrap_or_default().join(".vx"));
                let (meta, runtimes) = Self::parse_metadata(&content)?;
                return Ok(Self {
                    script_path: path,
                    meta,
                    runtimes,
                    sandbox: SandboxConfig::default(),
                    vx_home,
                    script_content: Arc::new(content),
                    script_hash: entry.script_hash,
                });
            }
        }

        // Cache miss: parse metadata and run analysis phase
        let (meta, runtimes) = Self::parse_metadata(&content)?;

        let vx_home = vx_paths::VxPaths::new()
            .map(|p| p.base_dir)
            .unwrap_or_else(|_| dirs::home_dir().unwrap_or_default().join(".vx"));

        let provider = Self {
            script_path: path.clone(),
            meta: meta.clone(),
            runtimes,
            sandbox: SandboxConfig::default(),
            vx_home,
            script_content: Arc::new(content),
            script_hash,
        };

        // Run analysis phase to produce frozen ProviderInfo
        let frozen_info = FrozenProviderInfo {
            versions_url: None,
            download_url: None,
            env_template: HashMap::new(),
            metadata: HashMap::new(),
        };

        // Store in analysis cache keyed by content hash
        {
            let mut cache = ANALYSIS_CACHE.write().await;
            cache.insert(
                script_hash,
                AnalysisCacheEntry {
                    script_hash,
                    frozen_info,
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

    /// Get the Starlark engine for this provider
    fn engine(&self) -> StarlarkEngine {
        StarlarkEngine::new()
    }

    /// Get the content hash of this provider's script
    pub fn script_hash(&self) -> &[u8; 32] {
        &self.script_hash
    }

    /// Get the content hash as a hex string
    pub fn script_hash_hex(&self) -> String {
        self.script_hash
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect()
    }

    /// Parse metadata from a Starlark script
    ///
    /// Looks for:
    /// - `name()` function
    /// - `description()` function
    /// - `runtimes` variable
    fn parse_metadata(content: &str) -> Result<(ProviderMeta, Vec<RuntimeMeta>)> {
        // Simple parsing for metadata
        // In a full implementation, we would use the Starlark parser

        let mut meta = ProviderMeta {
            name: "unknown".to_string(),
            description: String::new(),
            version: "1.0.0".to_string(),
            homepage: None,
            repository: None,
            platforms: None,
        };

        let mut runtimes: Vec<RuntimeMeta> = Vec::new();

        // Parse name() function
        for line in content.lines() {
            let line = line.trim();

            // Parse name()
            if line.starts_with("def name()")
                && let Some(value) = Self::extract_string_return(line, content)
            {
                meta.name = value;
            }

            // Parse description()
            if line.starts_with("def description()")
                && let Some(value) = Self::extract_string_return(line, content)
            {
                meta.description = value;
            }
        }

        // For now, create a default runtime with the provider name
        if runtimes.is_empty() {
            runtimes.push(RuntimeMeta {
                name: meta.name.clone(),
                description: meta.description.clone(),
                executable: meta.name.clone(),
                aliases: vec![],
                priority: 100,
            });
        }

        Ok((meta, runtimes))
    }

    /// Extract string return value from a function
    fn extract_string_return(_func_line: &str, _content: &str) -> Option<String> {
        // Simple extraction - look for return "value" in the next few lines
        // This is a simplified implementation
        None
    }

    /// Get the provider name
    pub fn name(&self) -> &str {
        &self.meta.name
    }

    /// Get the provider description
    pub fn description(&self) -> &str {
        &self.meta.description
    }

    /// Get the provider metadata
    pub fn meta(&self) -> &ProviderMeta {
        &self.meta
    }

    /// Get the runtime definitions
    pub fn runtimes(&self) -> &[RuntimeMeta] {
        &self.runtimes
    }

    /// Get the script path
    pub fn script_path(&self) -> &Path {
        &self.script_path
    }

    // === Provider Functions ===

    /// Call the `fetch_versions` function
    pub async fn fetch_versions(&self) -> Result<Vec<VersionInfo>> {
        let ctx = ProviderContext::new(&self.meta.name, self.vx_home.clone())
            .with_sandbox(self.sandbox.clone());

        // Execute the function
        // In a full implementation, this would use the Starlark evaluator
        self.execute_fetch_versions(&ctx).await
    }

    /// Call the `install` function
    pub async fn install(&self, version: &str) -> Result<InstallResult> {
        let ctx = ProviderContext::new(&self.meta.name, self.vx_home.clone())
            .with_sandbox(self.sandbox.clone())
            .with_version(version);

        self.execute_install(&ctx, version).await
    }

    /// Call the `prepare_environment` function
    pub async fn prepare_environment(&self, version: &str) -> Result<HashMap<String, String>> {
        let ctx = ProviderContext::new(&self.meta.name, self.vx_home.clone())
            .with_sandbox(self.sandbox.clone())
            .with_version(version);

        self.execute_prepare_environment(&ctx, version).await
    }

    // === Internal Execution Methods ===

    /// Execute fetch_versions function using the Starlark engine
    ///
    /// Handles two return shapes from Starlark:
    ///
    /// 1. **Descriptor dict** (`__type == "github_versions"`): returned by
    ///    `releases_to_versions(github_releases(...))` in http.star.
    ///    The Rust layer resolves this by calling the GitHub API directly,
    ///    keeping Starlark pure (no real HTTP in scripts).
    ///
    /// 2. **Plain list** of `{version, lts, prerelease, date}` dicts:
    ///    returned by custom `fetch_versions` implementations that build
    ///    the list themselves.
    async fn execute_fetch_versions(&self, ctx: &ProviderContext) -> Result<Vec<VersionInfo>> {
        let engine = self.engine();
        let result = engine.call_function(
            &self.script_path,
            &self.script_content,
            "fetch_versions",
            ctx,
            &[],
        );

        match result {
            Ok(json) => {
                // Shape 1: github_versions descriptor from http.star
                if let Some(type_str) = json.get("__type").and_then(|t| t.as_str())
                    && type_str == "github_versions"
                {
                    return self.resolve_github_versions_descriptor(&json).await;
                }

                // Shape 2: plain list of version dicts
                if let Some(arr) = json.as_array() {
                    let versions = arr
                        .iter()
                        .filter_map(|v| {
                            let version = v.get("version")?.as_str()?.to_string();
                            Some(VersionInfo {
                                version,
                                lts: v.get("lts").and_then(|l| l.as_bool()).unwrap_or(false),
                                stable: v.get("stable").and_then(|s| s.as_bool()).unwrap_or(true),
                                date: v
                                    .get("date")
                                    .and_then(|d| d.as_str())
                                    .map(|s| s.to_string()),
                            })
                        })
                        .collect();
                    Ok(versions)
                } else {
                    Ok(vec![])
                }
            }
            Err(Error::FunctionNotFound { .. }) => {
                warn!(
                    provider = %self.meta.name,
                    "fetch_versions() not found in provider script"
                );
                Ok(vec![])
            }
            Err(e) => Err(e),
        }
    }

    /// Resolve a `github_versions` descriptor by calling the GitHub API via reqwest.
    ///
    /// The descriptor shape (produced by `releases_to_versions(github_releases(...))` in http.star):
    /// ```json
    /// {
    ///   "__type":           "github_versions",
    ///   "source": {
    ///     "__type":             "github_releases",
    ///     "owner":              "jj-vcs",
    ///     "repo":               "jj",
    ///     "include_prereleases": false,
    ///     "url":                "https://api.github.com/repos/jj-vcs/jj/releases?per_page=50"
    ///   },
    ///   "strip_v_prefix":   true,
    ///   "skip_prereleases": true
    /// }
    /// ```
    async fn resolve_github_versions_descriptor(
        &self,
        descriptor: &serde_json::Value,
    ) -> Result<Vec<VersionInfo>> {
        let source = descriptor.get("source").ok_or_else(|| {
            Error::EvalError("github_versions descriptor missing 'source'".into())
        })?;

        let url = source
            .get("url")
            .and_then(|u| u.as_str())
            .ok_or_else(|| Error::EvalError("github_versions source missing 'url'".into()))?;

        let skip_prereleases = descriptor
            .get("skip_prereleases")
            .and_then(|s| s.as_bool())
            .unwrap_or(true);

        let strip_v = descriptor
            .get("strip_v_prefix")
            .and_then(|s| s.as_bool())
            .unwrap_or(true);

        debug!(
            provider = %self.meta.name,
            url = %url,
            "Resolving github_versions descriptor via HTTP"
        );

        // Fetch releases from GitHub API
        let client = reqwest::Client::builder()
            .user_agent("vx/0.1 (https://github.com/vx-dev/vx)")
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| Error::EvalError(format!("Failed to build HTTP client: {}", e)))?;

        let response = client
            .get(url)
            .send()
            .await
            .map_err(|e| Error::EvalError(format!("HTTP request failed for {}: {}", url, e)))?;

        if !response.status().is_success() {
            return Err(Error::EvalError(format!(
                "GitHub API returned {} for {}",
                response.status(),
                url
            )));
        }

        let releases: Vec<serde_json::Value> = response
            .json()
            .await
            .map_err(|e| Error::EvalError(format!("Failed to parse GitHub API response: {}", e)))?;

        // Convert releases to VersionInfo
        let versions: Vec<VersionInfo> = releases
            .iter()
            .filter(|r| {
                if skip_prereleases {
                    !r.get("prerelease")
                        .and_then(|p| p.as_bool())
                        .unwrap_or(false)
                        && !r.get("draft").and_then(|d| d.as_bool()).unwrap_or(false)
                } else {
                    true
                }
            })
            .filter_map(|r| {
                let tag = r.get("tag_name")?.as_str()?;
                let version = if strip_v {
                    tag.strip_prefix('v').unwrap_or(tag).to_string()
                } else {
                    tag.to_string()
                };
                if version.is_empty() {
                    return None;
                }
                Some(VersionInfo {
                    version,
                    lts: !r
                        .get("prerelease")
                        .and_then(|p| p.as_bool())
                        .unwrap_or(false),
                    stable: !r
                        .get("prerelease")
                        .and_then(|p| p.as_bool())
                        .unwrap_or(false),
                    date: r
                        .get("published_at")
                        .and_then(|d| d.as_str())
                        .map(|s| s.to_string()),
                })
            })
            .collect();

        debug!(
            provider = %self.meta.name,
            count = versions.len(),
            "Resolved {} versions from GitHub API",
            versions.len()
        );

        Ok(versions)
    }

    /// Execute install function using the Starlark engine
    async fn execute_install(&self, ctx: &ProviderContext, version: &str) -> Result<InstallResult> {
        let engine = self.engine();
        let result = engine.call_function(
            &self.script_path,
            &self.script_content,
            "install",
            ctx,
            &[serde_json::Value::String(version.to_string())],
        );

        match result {
            Ok(json) => {
                let success = json
                    .get("success")
                    .and_then(|s| s.as_bool())
                    .unwrap_or(false);
                let install_path = json
                    .get("path")
                    .and_then(|p| p.as_str())
                    .map(std::path::PathBuf::from)
                    .unwrap_or_else(|| ctx.paths.install_dir(version));

                if success {
                    Ok(InstallResult::success(install_path))
                } else {
                    let msg = json
                        .get("error")
                        .and_then(|e| e.as_str())
                        .unwrap_or("Installation failed")
                        .to_string();
                    Ok(InstallResult::failure(msg))
                }
            }
            Err(Error::FunctionNotFound { .. }) => {
                // install() is optional - use default Rust installer
                debug!(
                    provider = %self.meta.name,
                    "install() not found, using default installer"
                );
                Ok(InstallResult::failure("No install() function defined"))
            }
            Err(e) => Err(e),
        }
    }

    /// Execute prepare_environment function using the Starlark engine
    async fn execute_prepare_environment(
        &self,
        ctx: &ProviderContext,
        version: &str,
    ) -> Result<HashMap<String, String>> {
        let engine = self.engine();
        let result = engine.call_function(
            &self.script_path,
            &self.script_content,
            "prepare_environment",
            ctx,
            &[serde_json::Value::String(version.to_string())],
        );

        match result {
            Ok(json) => {
                if let Some(obj) = json.as_object() {
                    Ok(obj
                        .iter()
                        .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                        .collect())
                } else {
                    Ok(HashMap::new())
                }
            }
            Err(Error::FunctionNotFound { .. }) => Ok(HashMap::new()),
            Err(e) => Err(e),
        }
    }

    // === Cache Management ===

    /// Clear the incremental analysis cache
    pub async fn clear_cache() {
        let mut cache = ANALYSIS_CACHE.write().await;
        cache.clear();
        info!("Cleared Starlark incremental analysis cache");
    }

    /// Get cache statistics: (entry_count, total_runtimes)
    pub async fn cache_stats() -> (usize, usize) {
        let cache = ANALYSIS_CACHE.read().await;
        // Return (number of cached analysis entries, 0 - runtimes not tracked in analysis cache)
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
}

/// Detect if a path is a Starlark provider
pub fn is_starlark_provider(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| e == "star")
        .unwrap_or(false)
}

/// Check if a directory contains a Starlark provider
pub fn has_starlark_provider(dir: &Path) -> bool {
    dir.join("provider.star").exists()
}

/// Check if a directory contains a TOML provider
pub fn has_toml_provider(dir: &Path) -> bool {
    dir.join("provider.toml").exists()
}

/// Determine provider format for a directory
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderFormat {
    /// Starlark provider (provider.star)
    Starlark,
    /// TOML provider (provider.toml)
    Toml,
    /// No provider found
    None,
}

impl ProviderFormat {
    /// Detect the provider format for a directory
    pub fn detect(dir: &Path) -> Self {
        if has_starlark_provider(dir) {
            Self::Starlark
        } else if has_toml_provider(dir) {
            Self::Toml
        } else {
            Self::None
        }
    }

    /// Get the provider filename
    pub fn filename(&self) -> Option<&'static str> {
        match self {
            Self::Starlark => Some("provider.star"),
            Self::Toml => Some("provider.toml"),
            Self::None => None,
        }
    }
}
