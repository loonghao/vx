//! GitHub releases version fetcher implementation

use crate::error::{Result, VersionError};
use crate::fetcher::VersionFetcher;
use async_trait::async_trait;
use vx_plugin::types::VersionInfo;

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
        // Try to get URL from tool configuration first
        if let Ok(config_manager) = tokio::runtime::Handle::try_current()
            .and_then(|rt| rt.block_on(vx_config::ConfigManager::new()))
        {
            let config = config_manager.config();
            if let Some(url) = vx_config::get_tool_fetcher_url(config, &self.tool_name) {
                return url;
            }
        }

        // Fallback to default GitHub API URL
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
}
