//! Version fetching traits and implementations

use crate::{Result, VersionError, VersionInfo};
use async_trait::async_trait;

/// Trait for fetching version information from external sources
#[async_trait]
pub trait VersionFetcher: Send + Sync {
    /// Get the name of the tool this fetcher supports
    fn tool_name(&self) -> &str;

    /// Fetch available versions for the tool
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
}
