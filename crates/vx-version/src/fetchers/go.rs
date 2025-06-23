//! Go version fetcher implementation

use crate::error::{Result, VersionError};
use crate::fetcher::VersionFetcher;
use async_trait::async_trait;
use vx_plugin::types::VersionInfo;

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

        // Get URL from configuration
        let config_manager =
            vx_config::ConfigManager::new()
                .await
                .map_err(|e| VersionError::Other {
                    message: format!("Failed to load configuration: {}", e),
                })?;
        let config = config_manager.config();

        let url = vx_config::get_tool_fetcher_url(config, "go")
            .unwrap_or_else(|| "https://go.dev/dl/?mode=json".to_string());

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_go_fetcher_creation() {
        let fetcher = GoVersionFetcher::new();
        assert_eq!(fetcher.tool_name(), "go");
    }

    #[test]
    fn test_go_fetcher_default() {
        let fetcher = GoVersionFetcher::default();
        assert_eq!(fetcher.tool_name(), "go");
    }
}
