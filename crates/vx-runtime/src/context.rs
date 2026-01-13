//! Context types for dependency injection
//!
//! These contexts provide all external dependencies needed by runtimes,
//! allowing for easy testing through mock implementations.

use crate::traits::{CommandExecutor, FileSystem, HttpClient, Installer, PathProvider};
use crate::types::VersionInfo;
use crate::version_cache::{CacheMode, CompactVersion, VersionCache};
use chrono::DateTime;
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

    /// Alias for with_version_cache (for backward compatibility)
    #[deprecated(note = "Use with_version_cache instead")]
    pub fn with_version_cache_v2(mut self, cache: VersionCache) -> Self {
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

    /// Fetch versions from GitHub Releases API with caching
    ///
    /// This method uses the bincode-based cache for high performance:
    /// - 10-100x faster serialization/deserialization
    /// - 5-10x smaller cache files
    /// - Separate metadata file for quick validity checks
    /// - Automatic stale cache fallback on network errors
    ///
    /// # Arguments
    /// * `tool_name` - Name of the tool (used as cache key)
    /// * `owner` - GitHub repository owner
    /// * `repo` - GitHub repository name
    /// * `options` - Options for parsing releases
    ///
    /// # Example
    /// ```ignore
    /// let versions = ctx.fetch_github_releases(
    ///     "pnpm",
    ///     "pnpm",
    ///     "pnpm",
    ///     GitHubReleaseOptions::default().strip_v_prefix(true),
    /// ).await?;
    /// ```
    pub async fn fetch_github_releases(
        &self,
        tool_name: &str,
        owner: &str,
        repo: &str,
        options: GitHubReleaseOptions,
    ) -> anyhow::Result<Vec<VersionInfo>> {
        // Try cache first (fast path)
        if let Some(cache) = &self.version_cache {
            if let Some(versions) = cache.get(tool_name) {
                tracing::debug!(
                    "Using cached versions for {} ({} versions)",
                    tool_name,
                    versions.len()
                );
                return Ok(compact_to_version_info(versions));
            }

            // Check for offline mode
            if cache.mode() == CacheMode::Offline {
                return Err(anyhow::anyhow!(
                    "Offline mode: no cached versions available for {}. Run without --offline to fetch.",
                    tool_name
                ));
            }
        }

        // Get stale cache for fallback before fetching
        let stale_versions = self
            .version_cache
            .as_ref()
            .and_then(|c| c.get_stale(tool_name));

        // Fetch from GitHub API
        let url = format!(
            "https://api.github.com/repos/{}/{}/releases?per_page={}",
            owner, repo, options.per_page
        );

        tracing::debug!("Fetching versions for {} from {}", tool_name, url);
        let fetch_result = self.http.get_json_value(&url).await;

        match fetch_result {
            Ok(response) => {
                // Check for GitHub API error response
                if let Some(message) = response.get("message").and_then(|m| m.as_str()) {
                    // If we have stale cache, use it
                    if let Some(stale) = stale_versions {
                        tracing::warn!(
                            "GitHub API error for {}: {}, using stale cache",
                            tool_name,
                            message
                        );
                        return Ok(compact_to_version_info(stale));
                    }
                    return Err(anyhow::anyhow!(
                        "GitHub API error: {}. Set GITHUB_TOKEN or GH_TOKEN environment variable to avoid rate limits.",
                        message
                    ));
                }

                let releases = response
                    .as_array()
                    .ok_or_else(|| anyhow::anyhow!("Invalid response format from GitHub API"))?;

                // Convert to compact format and cache
                let compact_versions: Vec<CompactVersion> = releases
                    .iter()
                    .filter_map(|release| parse_github_release_to_compact(release, &options))
                    .collect();

                // Store in cache
                if let Some(cache) = &self.version_cache {
                    if let Err(e) = cache.set_with_options(
                        tool_name,
                        compact_versions.clone(),
                        Some(&url),
                        None,
                    ) {
                        tracing::warn!("Failed to cache versions for {}: {}", tool_name, e);
                    }
                }

                Ok(compact_to_version_info(compact_versions))
            }
            Err(fetch_error) => {
                // On network error, try stale cache
                if let Some(stale) = stale_versions {
                    tracing::warn!(
                        "Network error fetching versions for {}, using stale cache: {}",
                        tool_name,
                        fetch_error
                    );
                    return Ok(compact_to_version_info(stale));
                }

                // No stale cache, return helpful error
                let error_msg = fetch_error.to_string();
                if error_msg.contains("timeout") || error_msg.contains("timed out") {
                    Err(anyhow::anyhow!(
                        "Network timeout while fetching versions for {}.\n\n\
                        Possible solutions:\n\
                        1. Check your internet connection\n\
                        2. If behind a firewall/proxy, set HTTPS_PROXY environment variable\n\
                        3. Try again later (GitHub API may be temporarily slow)\n\
                        4. Use --offline flag if you have cached versions\n\n\
                        Original error: {}",
                        tool_name,
                        error_msg
                    ))
                } else {
                    Err(fetch_error)
                }
            }
        }
    }

    /// Fetch versions from a generic JSON API with caching
    ///
    /// This is a convenience method for fetching versions from any JSON API.
    /// The caller provides a parser function to convert the response into versions.
    ///
    /// # Arguments
    /// * `tool_name` - Name of the tool (used as cache key)
    /// * `url` - API URL to fetch
    /// * `parser` - Function to parse the JSON response into versions
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

            // Check for offline mode
            if cache.mode() == CacheMode::Offline {
                return Err(anyhow::anyhow!(
                    "Offline mode: no cached versions available for {}. Run without --offline to fetch.",
                    tool_name
                ));
            }
        }

        // Get stale cache for fallback
        let stale_json = self
            .version_cache
            .as_ref()
            .and_then(|c| c.get_stale_json(tool_name));

        // Fetch from API
        tracing::debug!("Fetching versions for {} from {}", tool_name, url);
        let fetch_result = self.http.get_json_value(url).await;

        match fetch_result {
            Ok(response) => {
                // Store in cache
                if let Some(cache) = &self.version_cache {
                    if let Err(e) =
                        cache.set_json_with_options(tool_name, response.clone(), Some(url), None)
                    {
                        tracing::warn!("Failed to cache versions for {}: {}", tool_name, e);
                    }
                }
                parser(response)
            }
            Err(fetch_error) => {
                // On network error, try stale cache
                if let Some(stale) = stale_json {
                    tracing::warn!(
                        "Network error fetching versions for {}, using stale cache: {}",
                        tool_name,
                        fetch_error
                    );
                    return parser(stale);
                }

                // No stale cache, return helpful error
                let error_msg = fetch_error.to_string();
                if error_msg.contains("timeout") || error_msg.contains("timed out") {
                    Err(anyhow::anyhow!(
                        "Network timeout while fetching versions for {} from {}.\n\n\
                        Possible solutions:\n\
                        1. Check your internet connection\n\
                        2. If behind a firewall/proxy, set HTTPS_PROXY environment variable\n\
                        3. Try again later (the API may be temporarily slow)\n\
                        4. Use --offline flag if you have cached versions\n\n\
                        Original error: {}",
                        tool_name,
                        url,
                        error_msg
                    ))
                } else {
                    Err(fetch_error)
                }
            }
        }
    }

    /// Fetch versions from GitHub tags API with caching
    ///
    /// This is useful for repositories that don't use GitHub Releases,
    /// only tags (like rustup).
    ///
    /// # Arguments
    /// * `tool_name` - Name of the tool (used as cache key)
    /// * `owner` - GitHub repository owner
    /// * `repo` - GitHub repository name
    /// * `options` - Options for parsing tags
    pub async fn fetch_github_tags(
        &self,
        tool_name: &str,
        owner: &str,
        repo: &str,
        options: GitHubReleaseOptions,
    ) -> anyhow::Result<Vec<VersionInfo>> {
        // Try cache first (fast path)
        if let Some(cache) = &self.version_cache {
            if let Some(versions) = cache.get(tool_name) {
                tracing::debug!(
                    "Using cached versions for {} ({} versions)",
                    tool_name,
                    versions.len()
                );
                return Ok(compact_to_version_info(versions));
            }

            // Check for offline mode
            if cache.mode() == CacheMode::Offline {
                return Err(anyhow::anyhow!(
                    "Offline mode: no cached versions available for {}. Run without --offline to fetch.",
                    tool_name
                ));
            }
        }

        // Get stale cache for fallback before fetching
        let stale_versions = self
            .version_cache
            .as_ref()
            .and_then(|c| c.get_stale(tool_name));

        // Fetch from GitHub Tags API
        let url = format!(
            "https://api.github.com/repos/{}/{}/tags?per_page={}",
            owner, repo, options.per_page
        );

        tracing::debug!("Fetching versions for {} from {}", tool_name, url);
        let fetch_result = self.http.get_json_value(&url).await;

        match fetch_result {
            Ok(response) => {
                // Check for GitHub API error response
                if let Some(message) = response.get("message").and_then(|m| m.as_str()) {
                    // If we have stale cache, use it
                    if let Some(stale) = stale_versions {
                        tracing::warn!(
                            "GitHub API error for {}: {}, using stale cache",
                            tool_name,
                            message
                        );
                        return Ok(compact_to_version_info(stale));
                    }
                    return Err(anyhow::anyhow!(
                        "GitHub API error: {}. Set GITHUB_TOKEN or GH_TOKEN environment variable to avoid rate limits.",
                        message
                    ));
                }

                let tags = response
                    .as_array()
                    .ok_or_else(|| anyhow::anyhow!("Invalid response format from GitHub API"))?;

                // Convert tags to compact format
                let compact_versions: Vec<CompactVersion> = tags
                    .iter()
                    .filter_map(|tag| parse_github_tag_to_compact(tag, &options))
                    .collect();

                // Store in cache
                if let Some(cache) = &self.version_cache {
                    if let Err(e) = cache.set_with_options(
                        tool_name,
                        compact_versions.clone(),
                        Some(&url),
                        None,
                    ) {
                        tracing::warn!("Failed to cache versions for {}: {}", tool_name, e);
                    }
                }

                Ok(compact_to_version_info(compact_versions))
            }
            Err(fetch_error) => {
                // On network error, try stale cache
                if let Some(stale) = stale_versions {
                    tracing::warn!(
                        "Network error fetching versions for {}, using stale cache: {}",
                        tool_name,
                        fetch_error
                    );
                    return Ok(compact_to_version_info(stale));
                }

                // No stale cache, return helpful error
                let error_msg = fetch_error.to_string();
                if error_msg.contains("timeout") || error_msg.contains("timed out") {
                    Err(anyhow::anyhow!(
                        "Network timeout while fetching versions for {}.\n\n\
                        Possible solutions:\n\
                        1. Check your internet connection\n\
                        2. If behind a firewall/proxy, set HTTPS_PROXY environment variable\n\
                        3. Try again later (GitHub API may be temporarily slow)\n\
                        4. Use --offline flag if you have cached versions\n\n\
                        Original error: {}",
                        tool_name,
                        error_msg
                    ))
                } else {
                    Err(fetch_error)
                }
            }
        }
    }
}

/// Convert CompactVersion list to VersionInfo list
fn compact_to_version_info(versions: Vec<CompactVersion>) -> Vec<VersionInfo> {
    versions
        .into_iter()
        .map(|v| {
            let released_at = if v.published_at > 0 {
                DateTime::from_timestamp(v.published_at as i64, 0)
            } else {
                None
            };

            VersionInfo {
                version: v.version,
                released_at,
                prerelease: v.prerelease,
                lts: false, // LTS info not stored in compact format
                download_url: None,
                checksum: None,
                metadata: HashMap::new(),
            }
        })
        .collect()
}

/// Parse a GitHub release JSON to CompactVersion
fn parse_github_release_to_compact(
    release: &serde_json::Value,
    options: &GitHubReleaseOptions,
) -> Option<CompactVersion> {
    // Skip drafts if configured
    if options.skip_drafts
        && release
            .get("draft")
            .and_then(|d| d.as_bool())
            .unwrap_or(false)
    {
        return None;
    }

    let tag = release.get("tag_name")?.as_str()?;

    // Apply tag prefix stripping
    let version = if let Some(prefix) = &options.tag_prefix {
        tag.strip_prefix(prefix).unwrap_or(tag)
    } else if options.strip_v_prefix {
        tag.strip_prefix('v').unwrap_or(tag)
    } else {
        tag
    };

    let prerelease = release
        .get("prerelease")
        .and_then(|p| p.as_bool())
        .unwrap_or(false);

    // Skip prereleases if configured
    if options.skip_prereleases && prerelease {
        return None;
    }

    let published_at = release
        .get("published_at")
        .and_then(|v| v.as_str())
        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.timestamp() as u64)
        .unwrap_or(0);

    Some(
        CompactVersion::new(version)
            .with_prerelease(prerelease)
            .with_published_at(published_at),
    )
}

/// Parse a GitHub tag JSON to CompactVersion
fn parse_github_tag_to_compact(
    tag: &serde_json::Value,
    options: &GitHubReleaseOptions,
) -> Option<CompactVersion> {
    let tag_name = tag.get("name")?.as_str()?;

    // Apply tag prefix stripping
    let version = if let Some(prefix) = &options.tag_prefix {
        tag_name.strip_prefix(prefix).unwrap_or(tag_name)
    } else if options.strip_v_prefix {
        tag_name.strip_prefix('v').unwrap_or(tag_name)
    } else {
        tag_name
    };

    // Skip prereleases if configured (detect by version string)
    let prerelease = version.contains("alpha")
        || version.contains("beta")
        || version.contains("rc")
        || version.contains("dev")
        || version.contains("pre");

    if options.skip_prereleases && prerelease {
        return None;
    }

    // Tags don't have published_at, so we use 0
    Some(CompactVersion::new(version).with_prerelease(prerelease))
}

/// Options for parsing GitHub releases
#[allow(clippy::type_complexity)]
pub struct GitHubReleaseOptions {
    /// Number of releases to fetch per page (max 100)
    pub per_page: u32,
    /// Whether to strip 'v' prefix from tags (e.g., "v1.0.0" -> "1.0.0")
    pub strip_v_prefix: bool,
    /// Custom tag prefix to strip (takes precedence over strip_v_prefix)
    pub tag_prefix: Option<String>,
    /// Whether to skip draft releases
    pub skip_drafts: bool,
    /// Whether to skip prerelease versions
    pub skip_prereleases: bool,
    /// Custom function to detect LTS versions
    lts_detector: Option<Box<dyn Fn(&str) -> bool + Send + Sync>>,
}

impl std::fmt::Debug for GitHubReleaseOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GitHubReleaseOptions")
            .field("per_page", &self.per_page)
            .field("strip_v_prefix", &self.strip_v_prefix)
            .field("tag_prefix", &self.tag_prefix)
            .field("skip_drafts", &self.skip_drafts)
            .field("skip_prereleases", &self.skip_prereleases)
            .field("lts_detector", &self.lts_detector.is_some())
            .finish()
    }
}

impl Default for GitHubReleaseOptions {
    fn default() -> Self {
        Self {
            per_page: 50,
            strip_v_prefix: true,
            tag_prefix: None,
            skip_drafts: true,
            skip_prereleases: false,
            lts_detector: None,
        }
    }
}

impl GitHubReleaseOptions {
    /// Create new options with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the number of releases to fetch per page
    pub fn per_page(mut self, count: u32) -> Self {
        self.per_page = count.min(100);
        self
    }

    /// Whether to strip 'v' prefix from tags
    pub fn strip_v_prefix(mut self, strip: bool) -> Self {
        self.strip_v_prefix = strip;
        self
    }

    /// Set a custom tag prefix to strip
    pub fn tag_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.tag_prefix = Some(prefix.into());
        self
    }

    /// Whether to skip draft releases
    pub fn skip_drafts(mut self, skip: bool) -> Self {
        self.skip_drafts = skip;
        self
    }

    /// Whether to skip prerelease versions
    pub fn skip_prereleases(mut self, skip: bool) -> Self {
        self.skip_prereleases = skip;
        self
    }

    /// Set a custom LTS detector function
    pub fn lts_detector<F>(mut self, detector: F) -> Self
    where
        F: Fn(&str) -> bool + Send + Sync + 'static,
    {
        self.lts_detector = Some(Box::new(detector));
        self
    }
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
