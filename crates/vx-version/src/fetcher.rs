//! Version fetching traits and implementations
//!
//! This module provides version fetching capabilities with support for:
//! - GitHub API with optional token authentication (to avoid rate limits)
//! - Node.js official release API
//! - Extensible trait for custom version sources

use crate::{Result, VersionError, VersionInfo};
use async_trait::async_trait;

/// Environment variable name for GitHub token
pub const GITHUB_TOKEN_ENV: &str = "GITHUB_TOKEN";

/// Alternative environment variable name for GitHub token (used by GitHub CLI)
pub const GH_TOKEN_ENV: &str = "GH_TOKEN";

/// Get GitHub token from environment variables
///
/// Checks in order: GITHUB_TOKEN, GH_TOKEN
pub fn get_github_token() -> Option<String> {
    std::env::var(GITHUB_TOKEN_ENV)
        .ok()
        .or_else(|| std::env::var(GH_TOKEN_ENV).ok())
        .filter(|t| !t.is_empty())
}

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
///
/// Supports optional GitHub token authentication to avoid API rate limits.
/// The token is read from environment variables: `GITHUB_TOKEN` or `GH_TOKEN`.
///
/// # Rate Limits
///
/// - Without token: 60 requests/hour per IP
/// - With token: 5,000 requests/hour per user
///
/// # Example
///
/// ```rust
/// use vx_version::GitHubVersionFetcher;
///
/// // Token will be automatically read from GITHUB_TOKEN or GH_TOKEN env var
/// let fetcher = GitHubVersionFetcher::new("astral-sh", "uv");
///
/// // Or explicitly set a token
/// let fetcher_with_token = GitHubVersionFetcher::new("astral-sh", "uv")
///     .with_token("ghp_xxxxxxxxxxxx".to_string());
/// ```
#[derive(Debug, Clone)]
pub struct GitHubVersionFetcher {
    owner: String,
    repo: String,
    tool_name: String,
    /// Optional GitHub token for authenticated requests
    token: Option<String>,
}

impl GitHubVersionFetcher {
    /// Create a new GitHubVersionFetcher
    ///
    /// Automatically reads token from GITHUB_TOKEN or GH_TOKEN environment variables.
    pub fn new(owner: &str, repo: &str) -> Self {
        Self {
            owner: owner.to_string(),
            repo: repo.to_string(),
            tool_name: repo.to_string(),
            token: get_github_token(),
        }
    }

    /// Create a new GitHubVersionFetcher with custom tool name
    pub fn with_tool_name(owner: &str, repo: &str, tool_name: &str) -> Self {
        Self {
            owner: owner.to_string(),
            repo: repo.to_string(),
            tool_name: tool_name.to_string(),
            token: get_github_token(),
        }
    }

    /// Set a custom GitHub token for this fetcher
    ///
    /// This overrides any token from environment variables.
    pub fn with_token(mut self, token: String) -> Self {
        self.token = Some(token);
        self
    }

    /// Check if this fetcher has a token configured
    pub fn has_token(&self) -> bool {
        self.token.is_some()
    }

    /// Get the API URL for releases
    pub fn releases_url(&self) -> String {
        format!(
            "https://api.github.com/repos/{}/{}/releases",
            self.owner, self.repo
        )
    }

    /// Build HTTP client with appropriate headers
    fn build_request(&self, client: &reqwest::Client, url: &str) -> reqwest::RequestBuilder {
        let mut request = client
            .get(url)
            .header("User-Agent", "vx-version")
            .header("Accept", "application/vnd.github+json")
            .header("X-GitHub-Api-Version", "2022-11-28");

        if let Some(token) = &self.token {
            request = request.header("Authorization", format!("Bearer {}", token));
        }

        request
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

        let response = self
            .build_request(&client, &url)
            .send()
            .await
            .map_err(|e| VersionError::NetworkError {
                url: url.clone(),
                source: e,
            })?;

        // Check for rate limit errors
        if response.status() == reqwest::StatusCode::FORBIDDEN {
            let remaining = response
                .headers()
                .get("x-ratelimit-remaining")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.parse::<u32>().ok())
                .unwrap_or(0);

            if remaining == 0 {
                return Err(VersionError::RateLimited {
                    message: format!(
                        "GitHub API rate limit exceeded. Set {} or {} environment variable to increase limit.",
                        GITHUB_TOKEN_ENV, GH_TOKEN_ENV
                    ),
                });
            }
        }

        // Check for other HTTP errors
        if !response.status().is_success() {
            return Err(VersionError::HttpError {
                url: url.clone(),
                status: response.status().as_u16(),
                message: format!("GitHub API returned status {}", response.status()),
            });
        }

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
