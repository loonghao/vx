//! Context types for dependency injection
//!
//! These contexts provide all external dependencies needed by runtimes,
//! allowing for easy testing through mock implementations.

use crate::github::{GitHubFetcher, GitHubReleaseOptions};
use crate::traits::{CommandExecutor, FileSystem, HttpClient, Installer, PathProvider};
use crate::types::VersionInfo;
use crate::version_cache::{CacheMode, VersionCache};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

/// Configuration for runtime operations
#[derive(Debug, Clone)]
pub struct RuntimeConfig {
    /// Whether to automatically install missing runtimes
    pub auto_install: bool,
    /// Whether to include prerelease versions
    pub include_prerelease: bool,
    /// Installation timeout
    pub install_timeout: Duration,
    /// Whether to verify checksums
    pub verify_checksum: bool,
    /// Whether to use verbose output
    pub verbose: bool,
    /// Cache mode for version fetching
    pub cache_mode: CacheMode,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            auto_install: true,
            include_prerelease: false,
            install_timeout: Duration::from_secs(300), // 5 minutes
            verify_checksum: true,
            verbose: false,
            cache_mode: CacheMode::Normal,
        }
    }
}

/// Context for runtime operations (install, fetch versions, etc.)
///
/// This context provides all dependencies needed for runtime operations,
/// allowing for easy mocking in tests.
#[derive(Clone)]
pub struct RuntimeContext {
    /// Path provider for directory management
    pub paths: Arc<dyn PathProvider>,
    /// HTTP client for network requests
    pub http: Arc<dyn HttpClient>,
    /// File system operations
    pub fs: Arc<dyn FileSystem>,
    /// Archive installer
    pub installer: Arc<dyn Installer>,
    /// Configuration
    pub config: RuntimeConfig,
    /// High-performance version cache (bincode format)
    pub version_cache: Option<VersionCache>,
    /// Pre-resolved download URLs from lock file (tool_name -> download_url)
    ///
    /// When a tool is being installed and its download URL is already known
    /// from vx.lock, this cache can be used to avoid re-fetching the URL.
    /// This improves performance and ensures reproducibility.
    pub download_url_cache: Option<HashMap<String, String>>,

    /// Tool-specific installation options from vx.toml.
    ///
    /// These are key-value pairs extracted from a tool's detailed configuration
    /// in vx.toml. For example, MSVC's `components = ["spectre"]` is passed as
    /// `{"VX_MSVC_COMPONENTS": "spectre"}`.
    ///
    /// This provides an explicit, testable way to pass tool configuration through
    /// the `RuntimeContext` (instead of relying on process-level environment variables).
    /// The environment variable fallback is still supported for backward compatibility
    /// (e.g., `VX_MSVC_COMPONENTS=spectre vx install msvc`).
    pub install_options: HashMap<String, String>,
}

impl RuntimeContext {
    /// Create a new runtime context
    pub fn new(
        paths: Arc<dyn PathProvider>,
        http: Arc<dyn HttpClient>,
        fs: Arc<dyn FileSystem>,
        installer: Arc<dyn Installer>,
    ) -> Self {
        Self {
            paths,
            http,
            fs,
            installer,
            config: RuntimeConfig::default(),
            version_cache: None,
            download_url_cache: None,
            install_options: HashMap::new(),
        }
    }

    /// Create a new runtime context with custom config
    pub fn with_config(mut self, config: RuntimeConfig) -> Self {
        self.config = config;
        self
    }

    /// Set version cache (high-performance bincode format)
    pub fn with_version_cache(mut self, cache: VersionCache) -> Self {
        self.version_cache = Some(cache);
        self
    }

    /// Set cache mode
    pub fn with_cache_mode(mut self, mode: CacheMode) -> Self {
        self.config.cache_mode = mode;
        if let Some(cache) = self.version_cache.take() {
            self.version_cache = Some(cache.with_mode(mode));
        }
        self
    }

    /// Set download URL cache from lock file
    ///
    /// This allows runtimes to use pre-resolved download URLs instead of
    /// re-fetching them during installation.
    pub fn with_download_url_cache(mut self, cache: HashMap<String, String>) -> Self {
        self.download_url_cache = Some(cache);
        self
    }

    /// Set tool-specific installation options.
    ///
    /// These are key-value pairs that runtimes can read during installation.
    /// For example, MSVC reads `VX_MSVC_COMPONENTS` to install Spectre libraries.
    pub fn with_install_options(mut self, options: HashMap<String, String>) -> Self {
        self.install_options = options;
        self
    }

    /// Set tool-specific installation options (mutating version).
    pub fn set_install_options(&mut self, options: HashMap<String, String>) {
        self.install_options = options;
    }

    /// Get an installation option by key.
    ///
    /// Returns the value from `install_options` if present.
    /// This does NOT fall back to environment variables — callers should
    /// handle that fallback themselves if needed.
    pub fn get_install_option(&self, key: &str) -> Option<&str> {
        self.install_options.get(key).map(|s| s.as_str())
    }

    /// Set download URL cache from lock file (mutating version)
    pub fn set_download_url_cache(&mut self, cache: HashMap<String, String>) {
        self.download_url_cache = Some(cache);
    }

    /// Get cached download URL for a tool
    ///
    /// Returns the cached URL if available, otherwise None.
    pub fn get_cached_download_url(&self, tool_name: &str) -> Option<String> {
        self.download_url_cache
            .as_ref()
            .and_then(|cache| cache.get(tool_name))
            .cloned()
    }

    /// Get cached data or fetch with a custom fetcher function
    ///
    /// This method provides caching for any JSON data source.
    /// It will use the cache if available, or call the fetcher function and cache the result.
    ///
    /// # Arguments
    /// * `cache_key` - Key for caching (usually the tool name)
    /// * `fetcher` - Async function that fetches the data
    ///
    /// # Example
    /// ```ignore
    /// let response = ctx
    ///     .get_cached_or_fetch("node", || async { ctx.http.get_json_value(url).await })
    ///     .await?;
    /// ```
    pub async fn get_cached_or_fetch<F, Fut>(
        &self,
        cache_key: &str,
        fetcher: F,
    ) -> anyhow::Result<serde_json::Value>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = anyhow::Result<serde_json::Value>>,
    {
        self.get_cached_or_fetch_with_url(cache_key, "", fetcher)
            .await
    }

    /// Get cached data or fetch with a custom fetcher function, storing the URL for reference
    ///
    /// # Arguments
    /// * `cache_key` - Key for caching (usually the tool name)
    /// * `url` - URL to store in cache metadata (for debugging/reference)
    /// * `fetcher` - Async function that fetches the data
    pub async fn get_cached_or_fetch_with_url<F, Fut>(
        &self,
        cache_key: &str,
        url: &str,
        fetcher: F,
    ) -> anyhow::Result<serde_json::Value>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = anyhow::Result<serde_json::Value>>,
    {
        // Try cache first
        if let Some(cache) = &self.version_cache {
            if let Some(cached) = cache.get_json(cache_key) {
                tracing::debug!("Using cached data for {}", cache_key);
                return Ok(cached);
            }

            // Check for offline mode
            if cache.mode() == CacheMode::Offline {
                return Err(anyhow::anyhow!(
                    "Offline mode: no cached data available for {}. Run without --offline to fetch.",
                    cache_key
                ));
            }
        }

        // Get stale cache for fallback
        let stale_json = self
            .version_cache
            .as_ref()
            .and_then(|c| c.get_stale_json(cache_key));

        // Fetch from source
        tracing::debug!("Fetching data for {}", cache_key);
        let fetch_result = fetcher().await;

        match fetch_result {
            Ok(response) => {
                // Store in cache
                if let Some(cache) = &self.version_cache {
                    let url_opt = if url.is_empty() { None } else { Some(url) };
                    if let Err(e) =
                        cache.set_json_with_options(cache_key, response.clone(), url_opt, None)
                    {
                        tracing::warn!("Failed to cache data for {}: {}", cache_key, e);
                    }
                }
                Ok(response)
            }
            Err(fetch_error) => {
                // On network error, try stale cache
                if let Some(stale) = stale_json {
                    tracing::warn!(
                        "Error fetching data for {}, using stale cache: {}",
                        cache_key,
                        fetch_error
                    );
                    return Ok(stale);
                }

                // No stale cache, return the error
                Err(fetch_error)
            }
        }
    }

    /// Build a [`GitHubFetcher`] borrowing this context's HTTP client and version cache.
    ///
    /// # Deprecated
    ///
    /// Use [`vx_version_fetcher::GitHubReleasesFetcher`] implementing the unified
    /// [`vx_version_fetcher::VersionFetcher`] trait instead.
    #[deprecated(
        since = "0.8.3",
        note = "Use `vx_version_fetcher::GitHubReleasesFetcher` instead, \
                which implements the unified `VersionFetcher` trait."
    )]
    pub fn github_fetcher(&self) -> GitHubFetcher<'_> {
        GitHubFetcher {
            http: Arc::clone(&self.http),
            cache: self.version_cache.as_ref(),
        }
    }

    /// Fetch versions from GitHub Releases API with caching and jsDelivr fallback.
    ///
    /// # Deprecated
    ///
    /// Use `vx_version_fetcher::GitHubReleasesFetcher::fetch(ctx)` instead.
    #[deprecated(
        since = "0.8.3",
        note = "Use `vx_version_fetcher::GitHubReleasesFetcher` instead."
    )]
    #[allow(deprecated)]
    pub async fn fetch_github_releases(
        &self,
        tool_name: &str,
        owner: &str,
        repo: &str,
        options: GitHubReleaseOptions,
    ) -> anyhow::Result<Vec<VersionInfo>> {
        self.github_fetcher()
            .fetch_releases(tool_name, owner, repo, options)
            .await
    }

    /// Fetch versions from GitHub Tags API with caching.
    ///
    /// # Deprecated
    ///
    /// Use `vx_version_fetcher::GitHubReleasesFetcher` (releases) or implement
    /// custom tag fetching via `vx_version_fetcher::CustomApiFetcher` instead.
    #[deprecated(
        since = "0.8.3",
        note = "Use `vx_version_fetcher::GitHubReleasesFetcher` or `CustomApiFetcher` instead."
    )]
    #[allow(deprecated)]
    pub async fn fetch_github_tags(
        &self,
        tool_name: &str,
        owner: &str,
        repo: &str,
        options: GitHubReleaseOptions,
    ) -> anyhow::Result<Vec<VersionInfo>> {
        self.github_fetcher()
            .fetch_tags(tool_name, owner, repo, options)
            .await
    }

    /// Fetch versions from any JSON API endpoint with caching.
    ///
    /// The caller provides a parser that converts the JSON response into versions.
    pub async fn fetch_json_versions<F>(
        &self,
        tool_name: &str,
        url: &str,
        parser: F,
    ) -> anyhow::Result<Vec<VersionInfo>>
    where
        F: FnOnce(serde_json::Value) -> anyhow::Result<Vec<VersionInfo>>,
    {
        // Try cache first
        if let Some(cache) = &self.version_cache {
            if let Some(cached) = cache.get_json(tool_name) {
                tracing::debug!("Using cached JSON versions for {}", tool_name);
                return parser(cached);
            }
            if cache.mode() == CacheMode::Offline {
                return Err(anyhow::anyhow!(
                    "Offline mode: no cached versions for {}. Run without --offline to fetch.",
                    tool_name
                ));
            }
        }

        let stale_json = self
            .version_cache
            .as_ref()
            .and_then(|c| c.get_stale_json(tool_name));

        tracing::debug!("Fetching versions for {} from {}", tool_name, url);
        match self.http.get_json_value(url).await {
            Ok(response) => {
                if let Some(cache) = &self.version_cache
                    && let Err(e) =
                        cache.set_json_with_options(tool_name, response.clone(), Some(url), None)
                {
                    tracing::warn!("Failed to cache versions for {}: {}", tool_name, e);
                }
                parser(response)
            }
            Err(e) => {
                if let Some(stale) = stale_json {
                    tracing::warn!(
                        "Network error fetching versions for {}, using stale cache: {}",
                        tool_name,
                        e
                    );
                    return parser(stale);
                }
                Err(e)
            }
        }
    }

    // End of GitHub fetch delegation methods.
}

/// Context for command execution
///
/// This context provides all dependencies needed for executing commands,
/// allowing for easy mocking in tests.
pub struct ExecutionContext {
    /// Working directory for the command
    pub working_dir: Option<PathBuf>,
    /// Environment variables to set
    pub env: HashMap<String, String>,
    /// Whether to capture stdout/stderr
    pub capture_output: bool,
    /// Command timeout
    pub timeout: Option<Duration>,
    /// Command executor
    pub executor: Arc<dyn CommandExecutor>,
}

impl ExecutionContext {
    /// Create a new execution context with an executor
    pub fn new(executor: Arc<dyn CommandExecutor>) -> Self {
        Self {
            working_dir: None,
            env: HashMap::new(),
            capture_output: false,
            timeout: None,
            executor,
        }
    }

    /// Set working directory
    pub fn with_working_dir(mut self, dir: PathBuf) -> Self {
        self.working_dir = Some(dir);
        self
    }

    /// Add an environment variable
    pub fn with_env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env.insert(key.into(), value.into());
        self
    }

    /// Set multiple environment variables
    pub fn with_envs(mut self, envs: HashMap<String, String>) -> Self {
        self.env.extend(envs);
        self
    }

    /// Enable output capture
    pub fn with_capture_output(mut self, capture: bool) -> Self {
        self.capture_output = capture;
        self
    }

    /// Set timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }
}
