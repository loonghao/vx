//! TurboCdn version fetcher implementation

use crate::error::{Result, VersionError};
use crate::fetcher::VersionFetcher;
use async_trait::async_trait;
use vx_plugin::types::VersionInfo;

/// Version fetcher using TurboCdn for optimized downloads
#[derive(Debug, Clone)]
pub struct TurboCdnVersionFetcher {
    owner: String,
    repo: String,
    tool_name: String,
    turbo_cdn_client: Option<turbo_cdn::TurboCdn>,
}

impl TurboCdnVersionFetcher {
    /// Create a new TurboCdnVersionFetcher (synchronous)
    pub fn new(owner: &str, repo: &str) -> Self {
        Self {
            owner: owner.to_string(),
            repo: repo.to_string(),
            tool_name: repo.to_string(),
            turbo_cdn_client: None,
        }
    }

    /// Create a new TurboCdnVersionFetcher with custom tool name
    pub fn with_tool_name(owner: &str, repo: &str, tool_name: &str) -> Self {
        Self {
            owner: owner.to_string(),
            repo: repo.to_string(),
            tool_name: tool_name.to_string(),
            turbo_cdn_client: None,
        }
    }

    /// Initialize with TurboCdn client (async)
    pub async fn init(owner: &str, repo: &str) -> Result<Self> {
        let turbo_cdn_client =
            turbo_cdn::TurboCdn::new()
                .await
                .map_err(|e| VersionError::Other {
                    message: format!("Failed to initialize TurboCdn: {}", e),
                })?;

        Ok(Self {
            owner: owner.to_string(),
            repo: repo.to_string(),
            tool_name: repo.to_string(),
            turbo_cdn_client: Some(turbo_cdn_client),
        })
    }

    /// Get or initialize the TurboCdn client
    async fn get_turbo_cdn_client(&self) -> Result<turbo_cdn::TurboCdn> {
        match &self.turbo_cdn_client {
            Some(client) => Ok(client.clone()),
            None => turbo_cdn::TurboCdn::new()
                .await
                .map_err(|e| VersionError::Other {
                    message: format!("Failed to initialize TurboCdn: {}", e),
                }),
        }
    }
}

#[async_trait]
impl VersionFetcher for TurboCdnVersionFetcher {
    fn tool_name(&self) -> &str {
        &self.tool_name
    }

    async fn fetch_versions(&self, include_prerelease: bool) -> Result<Vec<VersionInfo>> {
        // Get TurboCdn client
        let turbo_cdn_client = self.get_turbo_cdn_client().await?;

        // Try to use TurboCdn for version fetching first
        // For now, we'll use GitHub API as the primary source but optimize URLs with TurboCdn
        // TODO: Once turbo-cdn 0.1.1 supports direct GitHub releases API, we can use that
        let github_fetcher = super::github::GitHubVersionFetcher::new(&self.owner, &self.repo);
        let mut versions = github_fetcher.fetch_versions(include_prerelease).await?;

        // Optimize download URLs using turbo-cdn 0.1.1
        for version in &mut versions {
            // Look for download URLs in metadata and optimize them
            let mut optimized_urls = std::collections::HashMap::new();

            for (key, value) in &version.metadata {
                if key.contains("download_url") || key.contains("browser_download_url") {
                    match turbo_cdn_client.get_fastest_url(value).await {
                        Ok(optimized_url) => {
                            let optimized_key = format!("{}_optimized", key);
                            optimized_urls.insert(optimized_key, optimized_url);
                        }
                        Err(e) => {
                            // Log warning but don't fail - use original URL
                            eprintln!(
                                "Warning: Failed to optimize URL for {} {}: {}",
                                version.version, key, e
                            );
                            // Still add the original URL as "optimized" for consistency
                            let optimized_key = format!("{}_optimized", key);
                            optimized_urls.insert(optimized_key, value.clone());
                        }
                    }
                }
            }

            // Add optimized URLs to metadata
            for (key, value) in optimized_urls {
                version.metadata.insert(key, value);
            }
        }

        Ok(versions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_turbo_cdn_fetcher_creation() {
        let fetcher = TurboCdnVersionFetcher::new("astral-sh", "uv");
        assert_eq!(fetcher.tool_name(), "uv");
    }

    #[test]
    fn test_turbo_cdn_fetcher_with_custom_name() {
        let fetcher = TurboCdnVersionFetcher::with_tool_name("astral-sh", "uv", "python-uv");
        assert_eq!(fetcher.tool_name(), "python-uv");
    }
}
