//! Tests for fetcher URL configuration system
//!
//! These tests ensure that the new fetcher_url configuration works correctly
//! and that all tools can properly retrieve their version information.

use tokio;
use vx_config::{get_tool_fetcher_url, ConfigManager};

#[cfg(test)]
mod fetcher_url_tests {
    use super::*;

    #[tokio::test]
    async fn test_get_tool_fetcher_url_existing_tool() {
        let config_manager = ConfigManager::new()
            .await
            .expect("Failed to create config manager");
        let config = config_manager.config();

        // Test that we can get fetcher URLs for configured tools
        assert!(get_tool_fetcher_url(config, "go").is_some());
        assert!(get_tool_fetcher_url(config, "node").is_some());
        assert!(get_tool_fetcher_url(config, "bun").is_some());
    }

    #[tokio::test]
    async fn test_get_tool_fetcher_url_nonexistent_tool() {
        let config_manager = ConfigManager::new()
            .await
            .expect("Failed to create config manager");
        let config = config_manager.config();

        // Test that we get None for non-configured tools
        assert!(get_tool_fetcher_url(config, "nonexistent-tool").is_none());
    }

    #[tokio::test]
    async fn test_fetcher_url_format_validation() {
        let config_manager = ConfigManager::new()
            .await
            .expect("Failed to create config manager");
        let config = config_manager.config();

        // Test that all configured fetcher URLs are valid HTTPS URLs
        for (tool_name, tool_config) in &config.tools {
            if let Some(fetcher_url) = &tool_config.fetcher_url {
                assert!(
                    fetcher_url.starts_with("https://"),
                    "Tool {} has non-HTTPS fetcher URL: {}",
                    tool_name,
                    fetcher_url
                );

                // Basic URL validation
                assert!(
                    fetcher_url.contains("."),
                    "Tool {} has invalid fetcher URL: {}",
                    tool_name,
                    fetcher_url
                );
            }
        }
    }

    #[tokio::test]
    async fn test_go_fetcher_url_specific() {
        let config_manager = ConfigManager::new()
            .await
            .expect("Failed to create config manager");
        let config = config_manager.config();

        let go_url = get_tool_fetcher_url(config, "go").unwrap();

        // Go should use the official Go API
        assert!(go_url.contains("go.dev"));
        assert!(go_url.contains("dl"));
        assert!(go_url.contains("mode=json"));
    }

    #[tokio::test]
    async fn test_node_fetcher_url_specific() {
        let config_manager = ConfigManager::new()
            .await
            .expect("Failed to create config manager");
        let config = config_manager.config();

        let node_url = get_tool_fetcher_url(config, "node").unwrap();

        // Node should use the official Node.js API
        assert!(node_url.contains("nodejs.org"));
        assert!(node_url.contains("dist"));
        assert!(node_url.contains("index.json"));
    }

    #[tokio::test]
    async fn test_bun_fetcher_url_specific() {
        let config_manager = ConfigManager::new()
            .await
            .expect("Failed to create config manager");
        let config = config_manager.config();

        let bun_url = get_tool_fetcher_url(config, "bun").unwrap();

        // Bun should use GitHub releases API
        assert!(bun_url.contains("api.github.com"));
        assert!(bun_url.contains("oven-sh/bun"));
        assert!(bun_url.contains("releases"));
    }

    #[tokio::test]
    async fn test_all_tools_have_fetcher_urls() {
        let config_manager = ConfigManager::new()
            .await
            .expect("Failed to create config manager");
        let config = config_manager.config();

        // Every configured tool should have a fetcher URL, except for dependency tools
        for (tool_name, tool_config) in &config.tools {
            // Skip tools that depend on other tools (they don't need their own fetcher URL)
            if tool_config.depends_on.is_some() {
                continue;
            }

            assert!(
                tool_config.fetcher_url.is_some(),
                "Tool {} is missing fetcher_url configuration",
                tool_name
            );
        }
    }

    fn extract_domain(url: &str) -> String {
        url.split("://")
            .nth(1)
            .and_then(|s| s.split('/').next())
            .unwrap_or("")
            .to_string()
    }
}

#[cfg(test)]
mod config_validation_tests {
    use super::*;

    #[tokio::test]
    async fn test_config_completeness() {
        let config_manager = ConfigManager::new()
            .await
            .expect("Failed to create config manager");
        let config = config_manager.config();

        // Every tool should have all required fields
        for (tool_name, tool_config) in &config.tools {
            assert!(
                tool_config.description.is_some(),
                "Tool {} missing description",
                tool_name
            );
            assert!(
                tool_config.homepage.is_some(),
                "Tool {} missing homepage",
                tool_name
            );

            // Only check fetcher_url and download_url_template for independent tools
            if tool_config.depends_on.is_none() {
                assert!(
                    tool_config.fetcher_url.is_some(),
                    "Tool {} missing fetcher_url",
                    tool_name
                );
                assert!(
                    tool_config.download_url_template.is_some(),
                    "Tool {} missing download_url_template",
                    tool_name
                );
            }
        }
    }

    #[tokio::test]
    async fn test_config_no_hardcoded_values() {
        let config_manager = ConfigManager::new()
            .await
            .expect("Failed to create config manager");
        let config = config_manager.config();

        // Ensure no hardcoded values remain in configuration
        for (tool_name, tool_config) in &config.tools {
            if let Some(fetcher_url) = &tool_config.fetcher_url {
                // Should not contain placeholder values
                assert!(
                    !fetcher_url.contains("TODO"),
                    "Tool {} has TODO in fetcher_url",
                    tool_name
                );
                assert!(
                    !fetcher_url.contains("FIXME"),
                    "Tool {} has FIXME in fetcher_url",
                    tool_name
                );
                assert!(
                    !fetcher_url.contains("localhost"),
                    "Tool {} has localhost in fetcher_url",
                    tool_name
                );
            }
        }
    }
}

// Integration tests with fetchers are in vx-version crate
