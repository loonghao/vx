//! Version fetcher trait definition

use crate::{Result, VersionError, VersionInfo};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Trait for fetching version information from external sources
#[async_trait]
pub trait VersionFetcher: Send + Sync {
    /// Get the name of the tool this fetcher supports
    fn tool_name(&self) -> &str;

    /// Fetch available versions for the tool
    ///
    /// # Arguments
    /// * `include_prerelease` - Whether to include prerelease versions
    ///
    /// # Returns
    /// A vector of version information, sorted by version (latest first)
    async fn fetch_versions(&self, include_prerelease: bool) -> Result<Vec<VersionInfo>>;

    /// Get the latest stable version
    async fn get_latest_version(&self) -> Result<Option<VersionInfo>> {
        let versions = self.fetch_versions(false).await?;
        Ok(versions.into_iter().next())
    }

    /// Get the latest version (including prereleases)
    async fn get_latest_version_including_prerelease(&self) -> Result<Option<VersionInfo>> {
        let versions = self.fetch_versions(true).await?;
        Ok(versions.into_iter().next())
    }

    /// Check if a specific version exists
    async fn version_exists(&self, version: &str) -> Result<bool> {
        let versions = self.fetch_versions(true).await?;
        Ok(versions.iter().any(|v| v.version == version))
    }
}

/// Version fetcher for GitHub releases
#[derive(Debug, Clone)]
pub struct GitHubVersionFetcher {
    owner: String,
    repo: String,
    tool_name: String,
}

impl GitHubVersionFetcher {
    /// Create a new GitHubVersionFetcher
    pub fn new(owner: &str, repo: &str) -> Self {
        Self {
            owner: owner.to_string(),
            repo: repo.to_string(),
            tool_name: repo.to_string(),
        }
    }

    /// Create a new GitHubVersionFetcher with custom tool name
    pub fn with_tool_name(owner: &str, repo: &str, tool_name: &str) -> Self {
        Self {
            owner: owner.to_string(),
            repo: repo.to_string(),
            tool_name: tool_name.to_string(),
        }
    }

    /// Get the API URL for releases
    pub fn releases_url(&self) -> String {
        format!(
            "https://api.github.com/repos/{}/{}/releases",
            self.owner, self.repo
        )
    }
}

#[async_trait]
impl VersionFetcher for GitHubVersionFetcher {
    fn tool_name(&self) -> &str {
        &self.tool_name
    }

    async fn fetch_versions(&self, include_prerelease: bool) -> Result<Vec<VersionInfo>> {
        let client = reqwest::Client::new();
        let url = self.releases_url();

        let response = client
            .get(&url)
            .header("User-Agent", "vx-version")
            .send()
            .await
            .map_err(|e| VersionError::NetworkError {
                url: url.clone(),
                source: e,
            })?;

        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| VersionError::NetworkError { url, source: e })?;

        crate::parser::GitHubVersionParser::parse_versions(&json, include_prerelease)
    }
}

/// Version fetcher using Turbo CDN
#[derive(Debug, Clone)]
pub struct TurboCdnVersionFetcher {
    tool_name: String,
    owner: String,
    repo: String,
}

impl TurboCdnVersionFetcher {
    /// Create a new TurboCdnVersionFetcher with default configuration
    pub async fn new(owner: &str, repo: &str) -> Result<Self> {
        Ok(Self {
            tool_name: repo.to_string(),
            owner: owner.to_string(),
            repo: repo.to_string(),
        })
    }

    /// Create a new TurboCdnVersionFetcher with custom tool name
    pub async fn with_tool_name(owner: &str, repo: &str, tool_name: &str) -> Result<Self> {
        let mut fetcher = Self::new(owner, repo).await?;
        fetcher.tool_name = tool_name.to_string();
        Ok(fetcher)
    }

    /// Get repository identifier for turbo-cdn
    pub fn repo_identifier(&self) -> String {
        format!("{}/{}", self.owner, self.repo)
    }
}

#[async_trait]
impl VersionFetcher for TurboCdnVersionFetcher {
    fn tool_name(&self) -> &str {
        &self.tool_name
    }

    async fn fetch_versions(&self, include_prerelease: bool) -> Result<Vec<VersionInfo>> {
        // For now, use GitHub API as fallback until turbo-cdn integration is complete
        let github_fetcher = GitHubVersionFetcher::new(&self.owner, &self.repo);
        let mut versions = github_fetcher.fetch_versions(include_prerelease).await?;

        // Initialize turbo-cdn client for URL optimization
        let turbo_cdn_client =
            turbo_cdn::TurboCdn::new()
                .await
                .map_err(|e| VersionError::Other {
                    message: format!("Failed to initialize TurboCdn: {}", e),
                })?;

        // Update download URLs to use optimized turbo-cdn URLs
        for version in &mut versions {
            if let Some(original_url) = &version.download_url {
                // Try to optimize the URL with turbo-cdn
                match turbo_cdn_client.get_optimal_url(original_url).await {
                    Ok(optimized_url) => {
                        version.download_url = Some(optimized_url);
                    }
                    Err(_) => {
                        // If optimization fails, keep the original URL
                        // This ensures we always have a working download URL
                    }
                }
            }
        }

        Ok(versions)
    }
}

/// Version fetcher for Go official API
#[derive(Debug, Clone)]
pub struct GoVersionFetcher {
    tool_name: String,
}

impl GoVersionFetcher {
    /// Create a new GoVersionFetcher
    pub fn new() -> Self {
        Self {
            tool_name: "go".to_string(),
        }
    }
}

impl Default for GoVersionFetcher {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl VersionFetcher for GoVersionFetcher {
    fn tool_name(&self) -> &str {
        &self.tool_name
    }

    async fn fetch_versions(&self, include_prerelease: bool) -> Result<Vec<VersionInfo>> {
        let client = reqwest::Client::new();
        let url = "https://go.dev/dl/?mode=json";

        let response = client
            .get(url)
            .header("User-Agent", "vx-version")
            .send()
            .await
            .map_err(|e| VersionError::NetworkError {
                url: url.to_string(),
                source: e,
            })?;

        let json: serde_json::Value =
            response
                .json()
                .await
                .map_err(|e| VersionError::NetworkError {
                    url: url.to_string(),
                    source: e,
                })?;

        crate::parser::GoVersionParser::parse_versions(&json, include_prerelease)
    }
}

/// Version fetcher for Node.js releases
#[derive(Debug, Clone)]
pub struct NodeVersionFetcher {
    tool_name: String,
}

impl NodeVersionFetcher {
    /// Create a new NodeVersionFetcher
    pub fn new() -> Self {
        Self {
            tool_name: "node".to_string(),
        }
    }
}

impl Default for NodeVersionFetcher {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl VersionFetcher for NodeVersionFetcher {
    fn tool_name(&self) -> &str {
        &self.tool_name
    }

    async fn fetch_versions(&self, include_prerelease: bool) -> Result<Vec<VersionInfo>> {
        let client = reqwest::Client::new();
        let url = "https://nodejs.org/dist/index.json";

        let response = client
            .get(url)
            .header("User-Agent", "vx-version")
            .send()
            .await
            .map_err(|e| VersionError::NetworkError {
                url: url.to_string(),
                source: e,
            })?;

        let json: serde_json::Value =
            response
                .json()
                .await
                .map_err(|e| VersionError::NetworkError {
                    url: url.to_string(),
                    source: e,
                })?;

        crate::parser::NodeVersionParser::parse_versions(&json, include_prerelease)
    }
}

/// Cached entry for version information
#[derive(Debug, Clone)]
struct CachedVersions {
    versions: Vec<VersionInfo>,
    cached_at: Instant,
    include_prerelease: bool,
}

impl CachedVersions {
    fn new(versions: Vec<VersionInfo>, include_prerelease: bool) -> Self {
        Self {
            versions,
            cached_at: Instant::now(),
            include_prerelease,
        }
    }

    fn is_expired(&self, ttl: Duration) -> bool {
        self.cached_at.elapsed() > ttl
    }

    fn matches(&self, include_prerelease: bool) -> bool {
        // If we cached with prerelease=true, we can serve both true and false requests
        // If we cached with prerelease=false, we can only serve false requests
        !include_prerelease || self.include_prerelease
    }
}

/// Cached version fetcher wrapper
pub struct CachedVersionFetcher {
    inner: Arc<dyn VersionFetcher>,
    cache: Arc<RwLock<HashMap<String, CachedVersions>>>,
    ttl: Duration,
}

impl std::fmt::Debug for CachedVersionFetcher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CachedVersionFetcher")
            .field("tool_name", &self.inner.tool_name())
            .field("ttl", &self.ttl)
            .finish()
    }
}

impl CachedVersionFetcher {
    /// Create a new cached version fetcher with default TTL (5 minutes)
    pub fn new(inner: Arc<dyn VersionFetcher>) -> Self {
        Self::with_ttl(inner, Duration::from_secs(300))
    }

    /// Create a new cached version fetcher with custom TTL
    pub fn with_ttl(inner: Arc<dyn VersionFetcher>, ttl: Duration) -> Self {
        Self {
            inner,
            cache: Arc::new(RwLock::new(HashMap::new())),
            ttl,
        }
    }

    /// Clear the cache
    pub async fn clear_cache(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
    }

    /// Get cache key for a request
    fn cache_key(&self, include_prerelease: bool) -> String {
        format!("{}:{}", self.inner.tool_name(), include_prerelease)
    }
}

#[async_trait]
impl VersionFetcher for CachedVersionFetcher {
    fn tool_name(&self) -> &str {
        self.inner.tool_name()
    }

    async fn fetch_versions(&self, include_prerelease: bool) -> Result<Vec<VersionInfo>> {
        let cache_key = self.cache_key(include_prerelease);

        // Check cache first
        {
            let cache = self.cache.read().await;
            if let Some(cached) = cache.get(&cache_key) {
                if !cached.is_expired(self.ttl) && cached.matches(include_prerelease) {
                    // Filter out prerelease versions if not requested
                    if include_prerelease || cached.include_prerelease {
                        let filtered_versions = if include_prerelease {
                            cached.versions.clone()
                        } else {
                            cached
                                .versions
                                .iter()
                                .filter(|v| !v.prerelease)
                                .cloned()
                                .collect()
                        };
                        return Ok(filtered_versions);
                    }
                }
            }
        }

        // Cache miss or expired, fetch from source
        let versions = self.inner.fetch_versions(include_prerelease).await?;

        // Update cache
        {
            let mut cache = self.cache.write().await;
            cache.insert(
                cache_key,
                CachedVersions::new(versions.clone(), include_prerelease),
            );
        }

        Ok(versions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_github_fetcher_creation() {
        let fetcher = GitHubVersionFetcher::new("astral-sh", "uv");
        assert_eq!(fetcher.tool_name(), "uv");
        assert_eq!(
            fetcher.releases_url(),
            "https://api.github.com/repos/astral-sh/uv/releases"
        );
    }

    #[test]
    fn test_github_fetcher_with_custom_name() {
        let fetcher = GitHubVersionFetcher::with_tool_name("astral-sh", "uv", "python-uv");
        assert_eq!(fetcher.tool_name(), "python-uv");
    }

    #[test]
    fn test_node_fetcher_creation() {
        let fetcher = NodeVersionFetcher::new();
        assert_eq!(fetcher.tool_name(), "node");
    }

    #[test]
    fn test_cached_version_fetcher_creation() {
        let inner = Arc::new(GitHubVersionFetcher::new("astral-sh", "uv"));
        let cached_fetcher = CachedVersionFetcher::new(inner);
        assert_eq!(cached_fetcher.tool_name(), "uv");
    }

    #[test]
    fn test_cached_versions_expiry() {
        use std::time::Duration;

        let versions = vec![];
        let cached = CachedVersions::new(versions, false);

        // Should not be expired immediately
        assert!(!cached.is_expired(Duration::from_secs(300)));

        // Should be expired with zero TTL
        assert!(cached.is_expired(Duration::from_secs(0)));
    }
}
