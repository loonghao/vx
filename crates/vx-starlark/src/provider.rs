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

/// Resolved install layout from a Starlark `install_layout()` descriptor
///
/// The Starlark script returns a descriptor dict (e.g. from `msi_install()`,
/// `archive_install()`, or `binary_install()` in install.star). The Rust layer
/// resolves the descriptor into this typed struct and performs the actual I/O.
#[derive(Debug, Clone)]
pub enum InstallLayout {
    /// MSI package installation (Windows only)
    Msi {
        /// Download URL for the .msi file
        url: String,
        /// Relative paths to executables within the extracted MSI
        executable_paths: Vec<String>,
        /// Directory prefix to strip from extracted paths
        strip_prefix: Option<String>,
        /// Extra msiexec command-line properties
        extra_args: Vec<String>,
    },
    /// Archive installation (ZIP, TAR.GZ, TAR.XZ, etc.)
    Archive {
        /// Download URL for the archive
        url: String,
        /// Directory prefix to strip from extracted paths
        strip_prefix: Option<String>,
        /// Relative paths to executables within the extracted archive
        executable_paths: Vec<String>,
    },
    /// Single binary installation
    Binary {
        /// Download URL for the binary
        url: String,
        /// Target filename for the downloaded binary
        executable_name: Option<String>,
        /// Unix file permissions (e.g. "755")
        permissions: String,
    },
    /// System tool finder (for prepare_execution)
    ///
    /// Instructs the Rust runtime to search for an already-installed system tool
    /// via PATH lookup and optional known system paths, before falling back to
    /// the vx-managed installation.
    ///
    /// This is the Starlark equivalent of `prepare_execution()` in Rust runtimes.
    /// Follows the same descriptor pattern as Buck2's `ctx.actions.run()`:
    /// Starlark declares *what to find*, Rust performs the actual search.
    SystemFind {
        /// Executable name to search for (e.g. "7z", "git")
        executable: String,
        /// Additional absolute paths to check after PATH lookup
        system_paths: Vec<String>,
        /// Human-readable hint shown when the tool is not found
        hint: Option<String>,
    },
}

/// Actions returned by `post_extract()` hook in Starlark provider scripts
///
/// The `post_extract()` function returns a list of these action descriptors.
/// The Rust runtime executes them in order after archive extraction.
///
/// Equivalent to the `post_extract()` method in Rust `Runtime` trait.
#[derive(Debug, Clone)]
pub enum PostExtractAction {
    /// Create a shim script that wraps another executable
    ///
    /// Starlark: `create_shim("bunx", "bun", args=["x"])`
    CreateShim {
        /// Name of the shim to create (e.g. "bunx")
        name: String,
        /// Target executable the shim wraps (e.g. "bun")
        target: String,
        /// Arguments to prepend when the shim is invoked
        args: Vec<String>,
        /// Optional directory where the shim is created
        shim_dir: Option<String>,
    },
    /// Set Unix file permissions on an extracted file
    ///
    /// Starlark: `set_permissions("bin/mytool", "755")`
    SetPermissions {
        /// Relative path to the file within the install directory
        path: String,
        /// Unix permission mode string (e.g. "755")
        mode: String,
    },
    /// Run an arbitrary command as part of the post-extract hook
    ///
    /// Starlark: `run_command("install_name_tool", ["-add_rpath", "..."])`
    RunCommand {
        /// The executable to run
        executable: String,
        /// Arguments to pass to the executable
        args: Vec<String>,
        /// Optional working directory
        working_dir: Option<String>,
        /// Optional environment variables
        env: std::collections::HashMap<String, String>,
        /// How to handle command failure: "warn", "error", "ignore"
        on_failure: String,
    },
    /// Flatten a nested subdirectory into the install root
    ///
    /// Starlark: `flatten_dir(pattern = "jdk-*")`
    ///
    /// Many archives extract to a single top-level subdirectory
    /// (e.g. `jdk-21.0.1+12/`, `ffmpeg-7.1-essentials_build/`).
    /// This action moves all contents one level up and removes the
    /// now-empty subdirectory.
    FlattenDir {
        /// Optional glob pattern to match the subdirectory name (e.g. "jdk-*").
        /// If None, flattens the single subdirectory if exactly one exists.
        pattern: Option<String>,
        /// Optional list of subdirectory names to keep in place rather than
        /// flattening (e.g. ["bin", "lib"]).
        keep_subdirs: Vec<String>,
    },
}

/// Actions returned by `pre_run()` hook in Starlark provider scripts
///
/// The `pre_run()` function returns a list of these action descriptors.
/// The Rust runtime executes them in order before running the tool.
///
/// Equivalent to the `pre_run()` method in Rust `Runtime` trait.
#[derive(Debug, Clone)]
pub enum PreRunAction {
    /// Ensure project dependencies are installed before running
    ///
    /// Starlark: `ensure_dependencies("bun")`
    EnsureDependencies {
        /// The package manager executable to run (e.g. "bun", "npm")
        package_manager: String,
        /// File that must exist for this check to apply (e.g. "package.json")
        check_file: String,
        /// Optional lock file to check
        lock_file: Option<String>,
        /// Directory to check for existence (e.g. "node_modules")
        install_dir: String,
    },
    /// Run an arbitrary command before the tool executes
    ///
    /// Starlark: `run_command("git", ["submodule", "update"])`
    RunCommand {
        /// The executable to run
        executable: String,
        /// Arguments to pass to the executable
        args: Vec<String>,
        /// Optional working directory
        working_dir: Option<String>,
        /// Optional environment variables
        env: std::collections::HashMap<String, String>,
        /// How to handle command failure: "warn", "error", "ignore"
        on_failure: String,
    },
}

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

    /// Call the `download_url` function
    ///
    /// Returns the download URL for a specific version, or `None` if the
    /// platform is not supported.
    pub async fn download_url(&self, version: &str) -> Result<Option<String>> {
        let ctx = ProviderContext::new(&self.meta.name, self.vx_home.clone())
            .with_sandbox(self.sandbox.clone())
            .with_version(version);

        self.execute_download_url(&ctx, version).await
    }

    /// Call the `install_layout` function and resolve the returned descriptor
    ///
    /// Returns the resolved [`InstallLayout`] that describes how to install
    /// the tool, or `None` if the script returns `None` (e.g. unsupported
    /// platform) or the function is not defined.
    pub async fn install_layout(&self, version: &str) -> Result<Option<InstallLayout>> {
        let ctx = ProviderContext::new(&self.meta.name, self.vx_home.clone())
            .with_sandbox(self.sandbox.clone())
            .with_version(version);

        self.execute_install_layout(&ctx, version).await
    }

    /// Call the `post_extract` function and resolve the returned action list
    ///
    /// Returns a list of [`PostExtractAction`]s to execute after archive extraction,
    /// or an empty list if the function is not defined or returns an empty list.
    ///
    /// The `post_extract(ctx, version, install_dir)` function in Starlark should
    /// return a list of descriptors from `create_shim()`, `set_permissions()`,
    /// or `run_command()` in install.star.
    pub async fn post_extract(
        &self,
        version: &str,
        install_dir: &std::path::Path,
    ) -> Result<Vec<PostExtractAction>> {
        let ctx = ProviderContext::new(&self.meta.name, self.vx_home.clone())
            .with_sandbox(self.sandbox.clone())
            .with_version(version);

        self.execute_post_extract(&ctx, version, install_dir).await
    }

    /// Call the `pre_run` function and resolve the returned action list
    ///
    /// Returns a list of [`PreRunAction`]s to execute before running the tool,
    /// or an empty list if the function is not defined or returns an empty list.
    ///
    /// The `pre_run(ctx, args, executable)` function in Starlark should return
    /// a list of descriptors from `ensure_dependencies()` or `run_command()`
    /// in install.star.
    pub async fn pre_run(
        &self,
        args: &[String],
        executable: &std::path::Path,
    ) -> Result<Vec<PreRunAction>> {
        let ctx = ProviderContext::new(&self.meta.name, self.vx_home.clone())
            .with_sandbox(self.sandbox.clone());

        self.execute_pre_run(&ctx, args, executable).await
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

    /// Execute download_url function using the Starlark engine
    ///
    /// Returns the download URL string, or `None` if the script returns `None`
    /// (e.g. unsupported platform) or the function is not defined.
    async fn execute_download_url(
        &self,
        ctx: &ProviderContext,
        version: &str,
    ) -> Result<Option<String>> {
        let engine = self.engine();
        let result = engine.call_function(
            &self.script_path,
            &self.script_content,
            "download_url",
            ctx,
            &[serde_json::Value::String(version.to_string())],
        );

        match result {
            Ok(json) => {
                if json.is_null() {
                    Ok(None)
                } else if let Some(url) = json.as_str() {
                    Ok(Some(url.to_string()))
                } else {
                    Ok(None)
                }
            }
            Err(Error::FunctionNotFound { .. }) => {
                debug!(
                    provider = %self.meta.name,
                    "download_url() not found in provider script"
                );
                Ok(None)
            }
            Err(e) => Err(e),
        }
    }

    /// Execute install_layout function using the Starlark engine
    ///
    /// Handles three descriptor shapes returned by install.star helpers:
    ///
    /// 1. **`msi_install`** descriptor — Windows MSI package:
    ///    ```json
    ///    { "__type": "msi_install", "url": "...", "executable_paths": [...],
    ///      "strip_prefix": "...", "extra_args": [...] }
    ///    ```
    ///
    /// 2. **`archive_install`** descriptor — ZIP / TAR archive:
    ///    ```json
    ///    { "__type": "archive_install", "url": "...", "strip_prefix": "...",
    ///      "executable_paths": [...] }
    ///    ```
    ///
    /// 3. **`binary_install`** descriptor — single executable file:
    ///    ```json
    ///    { "__type": "binary_install", "url": "...", "executable_name": "...",
    ///      "permissions": "755" }
    ///    ```
    ///
    /// Returns `None` if the script returns `None` (unsupported platform) or
    /// the function is not defined.
    async fn execute_install_layout(
        &self,
        ctx: &ProviderContext,
        version: &str,
    ) -> Result<Option<InstallLayout>> {
        let engine = self.engine();
        let result = engine.call_function(
            &self.script_path,
            &self.script_content,
            "install_layout",
            ctx,
            &[serde_json::Value::String(version.to_string())],
        );

        match result {
            Ok(json) => {
                if json.is_null() {
                    return Ok(None);
                }

                let type_str = json.get("__type").and_then(|t| t.as_str()).unwrap_or("");

                match type_str {
                    "msi_install" => {
                        let url = json
                            .get("url")
                            .and_then(|u| u.as_str())
                            .ok_or_else(|| {
                                Error::EvalError("msi_install descriptor missing 'url'".into())
                            })?
                            .to_string();

                        let executable_paths = json
                            .get("executable_paths")
                            .and_then(|p| p.as_array())
                            .map(|arr| {
                                arr.iter()
                                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                                    .collect()
                            })
                            .unwrap_or_default();

                        let strip_prefix = json
                            .get("strip_prefix")
                            .and_then(|s| s.as_str())
                            .map(|s| s.to_string());

                        let extra_args = json
                            .get("extra_args")
                            .and_then(|a| a.as_array())
                            .map(|arr| {
                                arr.iter()
                                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                                    .collect()
                            })
                            .unwrap_or_default();

                        debug!(
                            provider = %self.meta.name,
                            url = %url,
                            "Resolved msi_install descriptor"
                        );

                        Ok(Some(InstallLayout::Msi {
                            url,
                            executable_paths,
                            strip_prefix,
                            extra_args,
                        }))
                    }

                    "archive_install" => {
                        let url = json
                            .get("url")
                            .and_then(|u| u.as_str())
                            .ok_or_else(|| {
                                Error::EvalError("archive_install descriptor missing 'url'".into())
                            })?
                            .to_string();

                        let strip_prefix = json
                            .get("strip_prefix")
                            .and_then(|s| s.as_str())
                            .map(|s| s.to_string());

                        let executable_paths = json
                            .get("executable_paths")
                            .and_then(|p| p.as_array())
                            .map(|arr| {
                                arr.iter()
                                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                                    .collect()
                            })
                            .unwrap_or_default();

                        debug!(
                            provider = %self.meta.name,
                            url = %url,
                            "Resolved archive_install descriptor"
                        );

                        Ok(Some(InstallLayout::Archive {
                            url,
                            strip_prefix,
                            executable_paths,
                        }))
                    }

                    "binary_install" => {
                        let url = json
                            .get("url")
                            .and_then(|u| u.as_str())
                            .ok_or_else(|| {
                                Error::EvalError("binary_install descriptor missing 'url'".into())
                            })?
                            .to_string();

                        let executable_name = json
                            .get("executable_name")
                            .and_then(|n| n.as_str())
                            .map(|s| s.to_string());

                        let permissions = json
                            .get("permissions")
                            .and_then(|p| p.as_str())
                            .unwrap_or("755")
                            .to_string();

                        debug!(
                            provider = %self.meta.name,
                            url = %url,
                            "Resolved binary_install descriptor"
                        );

                        Ok(Some(InstallLayout::Binary {
                            url,
                            executable_name,
                            permissions,
                        }))
                    }

                    "system_find" => {
                        let executable = json
                            .get("executable")
                            .and_then(|e| e.as_str())
                            .ok_or_else(|| {
                                Error::EvalError(
                                    "system_find descriptor missing 'executable'".into(),
                                )
                            })?
                            .to_string();

                        let system_paths = json
                            .get("system_paths")
                            .and_then(|p| p.as_array())
                            .map(|arr| {
                                arr.iter()
                                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                                    .collect()
                            })
                            .unwrap_or_default();

                        let hint = json
                            .get("hint")
                            .and_then(|h| h.as_str())
                            .map(|s| s.to_string());

                        debug!(
                            provider = %self.meta.name,
                            executable = %executable,
                            "Resolved system_find descriptor"
                        );

                        Ok(Some(InstallLayout::SystemFind {
                            executable,
                            system_paths,
                            hint,
                        }))
                    }

                    other => {
                        warn!(
                            provider = %self.meta.name,
                            type_ = %other,
                            "Unknown install_layout descriptor type, ignoring"
                        );
                        Ok(None)
                    }
                }
            }
            Err(Error::FunctionNotFound { .. }) => {
                debug!(
                    provider = %self.meta.name,
                    "install_layout() not found in provider script"
                );
                Ok(None)
            }
            Err(e) => Err(e),
        }
    }

    /// Execute post_extract function using the Starlark engine
    ///
    /// The function signature in Starlark is:
    /// ```python
    /// def post_extract(ctx, version, install_dir):
    ///     return [create_shim("bunx", "bun", args=["x"])]
    /// ```
    async fn execute_post_extract(
        &self,
        ctx: &ProviderContext,
        version: &str,
        install_dir: &std::path::Path,
    ) -> Result<Vec<PostExtractAction>> {
        let engine = self.engine();
        let result = engine.call_function(
            &self.script_path,
            &self.script_content,
            "post_extract",
            ctx,
            &[
                serde_json::Value::String(version.to_string()),
                serde_json::Value::String(install_dir.to_string_lossy().to_string()),
            ],
        );

        match result {
            Ok(json) => {
                let actions = self.parse_hook_actions(&json, "post_extract")?;
                Ok(actions
                    .into_iter()
                    .filter_map(|a| self.json_to_post_extract_action(&a))
                    .collect())
            }
            Err(Error::FunctionNotFound { .. }) => {
                debug!(
                    provider = %self.meta.name,
                    "post_extract() not found in provider script"
                );
                Ok(vec![])
            }
            Err(e) => Err(e),
        }
    }

    /// Execute pre_run function using the Starlark engine
    ///
    /// The function signature in Starlark is:
    /// ```python
    /// def pre_run(ctx, args, executable):
    ///     if len(args) > 0 and args[0] == "run":
    ///         return [ensure_dependencies("bun")]
    ///     return []
    /// ```
    async fn execute_pre_run(
        &self,
        ctx: &ProviderContext,
        args: &[String],
        executable: &std::path::Path,
    ) -> Result<Vec<PreRunAction>> {
        let engine = self.engine();
        let args_json: Vec<serde_json::Value> = args
            .iter()
            .map(|a| serde_json::Value::String(a.clone()))
            .collect();

        let result = engine.call_function(
            &self.script_path,
            &self.script_content,
            "pre_run",
            ctx,
            &[
                serde_json::Value::Array(args_json),
                serde_json::Value::String(executable.to_string_lossy().to_string()),
            ],
        );

        match result {
            Ok(json) => {
                let actions = self.parse_hook_actions(&json, "pre_run")?;
                Ok(actions
                    .into_iter()
                    .filter_map(|a| self.json_to_pre_run_action(&a))
                    .collect())
            }
            Err(Error::FunctionNotFound { .. }) => {
                debug!(
                    provider = %self.meta.name,
                    "pre_run() not found in provider script"
                );
                Ok(vec![])
            }
            Err(e) => Err(e),
        }
    }

    /// Parse hook function return value into a list of action JSON objects
    ///
    /// Hook functions must return either:
    /// - A list of descriptor dicts: `[create_shim(...), set_permissions(...)]`
    /// - An empty list: `[]`
    /// - `None`: treated as empty list
    fn parse_hook_actions(
        &self,
        json: &serde_json::Value,
        func_name: &str,
    ) -> Result<Vec<serde_json::Value>> {
        if json.is_null() {
            return Ok(vec![]);
        }
        if let Some(arr) = json.as_array() {
            return Ok(arr.clone());
        }
        warn!(
            provider = %self.meta.name,
            func = %func_name,
            "Hook function must return a list, got: {:?}",
            json
        );
        Ok(vec![])
    }

    /// Convert a JSON descriptor to a PostExtractAction
    fn json_to_post_extract_action(&self, json: &serde_json::Value) -> Option<PostExtractAction> {
        let type_str = json.get("__type").and_then(|t| t.as_str()).unwrap_or("");

        match type_str {
            "create_shim" => {
                let name = json.get("name").and_then(|n| n.as_str())?.to_string();
                let target = json.get("target").and_then(|t| t.as_str())?.to_string();
                let args = json
                    .get("args")
                    .and_then(|a| a.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str().map(|s| s.to_string()))
                            .collect()
                    })
                    .unwrap_or_default();
                let shim_dir = json
                    .get("shim_dir")
                    .and_then(|d| d.as_str())
                    .map(|s| s.to_string());

                debug!(
                    provider = %self.meta.name,
                    shim = %name,
                    target = %target,
                    "Resolved create_shim descriptor"
                );

                Some(PostExtractAction::CreateShim {
                    name,
                    target,
                    args,
                    shim_dir,
                })
            }

            "set_permissions" => {
                let path = json.get("path").and_then(|p| p.as_str())?.to_string();
                let mode = json
                    .get("mode")
                    .and_then(|m| m.as_str())
                    .unwrap_or("755")
                    .to_string();

                debug!(
                    provider = %self.meta.name,
                    path = %path,
                    mode = %mode,
                    "Resolved set_permissions descriptor"
                );

                Some(PostExtractAction::SetPermissions { path, mode })
            }

            "run_command" => {
                let executable = json.get("executable").and_then(|e| e.as_str())?.to_string();
                let args = json
                    .get("args")
                    .and_then(|a| a.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str().map(|s| s.to_string()))
                            .collect()
                    })
                    .unwrap_or_default();
                let working_dir = json
                    .get("working_dir")
                    .and_then(|d| d.as_str())
                    .map(|s| s.to_string());
                let env = json
                    .get("env")
                    .and_then(|e| e.as_object())
                    .map(|obj| {
                        obj.iter()
                            .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                            .collect()
                    })
                    .unwrap_or_default();
                let on_failure = json
                    .get("on_failure")
                    .and_then(|f| f.as_str())
                    .unwrap_or("warn")
                    .to_string();

                debug!(
                    provider = %self.meta.name,
                    executable = %executable,
                    "Resolved run_command descriptor (post_extract)"
                );

                Some(PostExtractAction::RunCommand {
                    executable,
                    args,
                    working_dir,
                    env,
                    on_failure,
                })
            }

            "flatten_dir" => {
                let pattern = json
                    .get("pattern")
                    .and_then(|p| p.as_str())
                    .map(|s| s.to_string());
                let keep_subdirs = json
                    .get("keep_subdirs")
                    .and_then(|k| k.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str().map(|s| s.to_string()))
                            .collect()
                    })
                    .unwrap_or_default();

                debug!(
                    provider = %self.meta.name,
                    pattern = ?pattern,
                    "Resolved flatten_dir descriptor"
                );

                Some(PostExtractAction::FlattenDir {
                    pattern,
                    keep_subdirs,
                })
            }

            other => {
                warn!(
                    provider = %self.meta.name,
                    type_ = %other,
                    "Unknown post_extract action type, ignoring"
                );
                None
            }
        }
    }

    /// Convert a JSON descriptor to a PreRunAction
    fn json_to_pre_run_action(&self, json: &serde_json::Value) -> Option<PreRunAction> {
        let type_str = json.get("__type").and_then(|t| t.as_str()).unwrap_or("");

        match type_str {
            "ensure_dependencies" => {
                let package_manager = json
                    .get("package_manager")
                    .and_then(|p| p.as_str())?
                    .to_string();
                let check_file = json
                    .get("check_file")
                    .and_then(|f| f.as_str())
                    .unwrap_or("package.json")
                    .to_string();
                let lock_file = json
                    .get("lock_file")
                    .and_then(|f| f.as_str())
                    .map(|s| s.to_string());
                let install_dir = json
                    .get("install_dir")
                    .and_then(|d| d.as_str())
                    .unwrap_or("node_modules")
                    .to_string();

                debug!(
                    provider = %self.meta.name,
                    pm = %package_manager,
                    "Resolved ensure_dependencies descriptor"
                );

                Some(PreRunAction::EnsureDependencies {
                    package_manager,
                    check_file,
                    lock_file,
                    install_dir,
                })
            }

            "run_command" => {
                let executable = json.get("executable").and_then(|e| e.as_str())?.to_string();
                let args = json
                    .get("args")
                    .and_then(|a| a.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str().map(|s| s.to_string()))
                            .collect()
                    })
                    .unwrap_or_default();
                let working_dir = json
                    .get("working_dir")
                    .and_then(|d| d.as_str())
                    .map(|s| s.to_string());
                let env = json
                    .get("env")
                    .and_then(|e| e.as_object())
                    .map(|obj| {
                        obj.iter()
                            .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                            .collect()
                    })
                    .unwrap_or_default();
                let on_failure = json
                    .get("on_failure")
                    .and_then(|f| f.as_str())
                    .unwrap_or("warn")
                    .to_string();

                debug!(
                    provider = %self.meta.name,
                    executable = %executable,
                    "Resolved run_command descriptor (pre_run)"
                );

                Some(PreRunAction::RunCommand {
                    executable,
                    args,
                    working_dir,
                    env,
                    on_failure,
                })
            }

            other => {
                warn!(
                    provider = %self.meta.name,
                    type_ = %other,
                    "Unknown pre_run action type, ignoring"
                );
                None
            }
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
