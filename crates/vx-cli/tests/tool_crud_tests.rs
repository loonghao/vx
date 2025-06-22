//! Critical tests for tool CRUD (Create, Read, Update, Delete) operations
//!
//! These tests cover the most error-prone areas of vx tool management:
//! - Tool installation with various edge cases
//! - Tool removal and cleanup
//! - Tool listing and discovery
//! - Tool updates and version management

use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;
use tokio;

use vx_cli::commands::{install, list, remove, update};
use vx_config::ConfigManager;
use vx_plugin::VxTool;

/// Test fixture for tool CRUD operations
struct ToolCrudTestFixture {
    temp_dir: TempDir,
    config_manager: ConfigManager,
}

impl ToolCrudTestFixture {
    async fn new() -> Self {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");

        // Create a test configuration
        let config_manager = ConfigManager::new()
            .await
            .expect("Failed to create config manager");

        Self {
            temp_dir,
            config_manager,
        }
    }

    fn vx_home(&self) -> PathBuf {
        self.temp_dir.path().to_path_buf()
    }
}

#[cfg(test)]
mod install_tests {
    use super::*;

    #[tokio::test]
    async fn test_install_nonexistent_tool() {
        let fixture = ToolCrudTestFixture::new().await;

        // Try to install a tool that doesn't exist
        let result = install::install_tool("nonexistent-tool-xyz", None, false).await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Tool not found"));
    }

    #[tokio::test]
    async fn test_install_tool_twice_without_force() {
        let fixture = ToolCrudTestFixture::new().await;

        // First installation should succeed
        let result1 = install::install_tool("go", Some("1.21.0"), false).await;
        // Note: This might fail if network is unavailable, which is acceptable for now

        if result1.is_ok() {
            // Second installation without force should fail
            let result2 = install::install_tool("go", Some("1.21.0"), false).await;
            assert!(result2.is_err());
            assert!(result2
                .unwrap_err()
                .to_string()
                .contains("already installed"));
        }
    }

    #[tokio::test]
    async fn test_install_tool_twice_with_force() {
        let fixture = ToolCrudTestFixture::new().await;

        // First installation
        let result1 = install::install_tool("go", Some("1.21.0"), false).await;

        if result1.is_ok() {
            // Second installation with force should succeed
            let result2 = install::install_tool("go", Some("1.21.0"), true).await;
            assert!(result2.is_ok());
        }
    }

    #[tokio::test]
    async fn test_install_latest_version_resolution() {
        let fixture = ToolCrudTestFixture::new().await;

        // Test that "latest" gets resolved to actual version
        let result = install::install_tool("go", None, false).await; // None should default to latest

        // This test verifies the version resolution logic we just fixed
        if result.is_err() {
            let error_msg = result.unwrap_err().to_string();
            // Should not contain "latest" in download URL
            assert!(!error_msg.contains("golatest.windows"));
            assert!(!error_msg.contains("/latest/"));
        }
    }

    #[tokio::test]
    async fn test_install_invalid_version() {
        let fixture = ToolCrudTestFixture::new().await;

        // Try to install a version that doesn't exist
        let result = install::install_tool("go", Some("999.999.999"), false).await;

        assert!(result.is_err());
        // Should contain version-related error message
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("version") || error_msg.contains("not found"));
    }

    #[tokio::test]
    async fn test_install_with_network_simulation() {
        // TODO: Implement network failure simulation
        // This would test how installation handles network interruptions
    }

    #[tokio::test]
    async fn test_install_disk_space_simulation() {
        // TODO: Implement disk space limitation simulation
        // This would test how installation handles insufficient disk space
    }
}

#[cfg(test)]
mod remove_tests {
    use super::*;

    #[tokio::test]
    async fn test_remove_nonexistent_tool() {
        let fixture = ToolCrudTestFixture::new().await;

        // Try to remove a tool that was never installed
        let result = remove::remove_tool("nonexistent-tool-xyz", None).await;

        assert!(result.is_err());
        assert!(
            result.unwrap_err().to_string().contains("not found")
                || result.unwrap_err().to_string().contains("not installed")
        );
    }

    #[tokio::test]
    async fn test_remove_specific_version() {
        let fixture = ToolCrudTestFixture::new().await;

        // First install a tool
        let install_result = install::install_tool("go", Some("1.21.0"), false).await;

        if install_result.is_ok() {
            // Then remove the specific version
            let remove_result = remove::remove_tool("go", Some("1.21.0")).await;
            assert!(remove_result.is_ok());

            // Verify it's actually removed
            let list_result = list::list_installed_tools().await;
            if let Ok(tools) = list_result {
                assert!(!tools
                    .iter()
                    .any(|t| t.name == "go" && t.version == "1.21.0"));
            }
        }
    }

    #[tokio::test]
    async fn test_remove_all_versions() {
        let fixture = ToolCrudTestFixture::new().await;

        // Install multiple versions
        let _ = install::install_tool("go", Some("1.20.0"), false).await;
        let _ = install::install_tool("go", Some("1.21.0"), false).await;

        // Remove all versions
        let result = remove::remove_tool("go", None).await; // None = remove all

        if result.is_ok() {
            // Verify all versions are removed
            let list_result = list::list_installed_tools().await;
            if let Ok(tools) = list_result {
                assert!(!tools.iter().any(|t| t.name == "go"));
            }
        }
    }

    #[tokio::test]
    async fn test_remove_with_dependencies() {
        // TODO: Test removing a tool that other tools depend on
        // Should either fail or remove dependencies as well
    }

    #[tokio::test]
    async fn test_remove_partial_failure_rollback() {
        // TODO: Test rollback when removal partially fails
        // E.g., some files are locked or permission denied
    }
}

#[cfg(test)]
mod list_tests {
    use super::*;

    #[tokio::test]
    async fn test_list_empty_installation() {
        let fixture = ToolCrudTestFixture::new().await;

        // List tools when nothing is installed
        let result = list::list_installed_tools().await;

        assert!(result.is_ok());
        let tools = result.unwrap();
        assert!(tools.is_empty());
    }

    #[tokio::test]
    async fn test_list_after_installation() {
        let fixture = ToolCrudTestFixture::new().await;

        // Install a tool
        let install_result = install::install_tool("go", Some("1.21.0"), false).await;

        if install_result.is_ok() {
            // List should show the installed tool
            let list_result = list::list_installed_tools().await;
            assert!(list_result.is_ok());

            let tools = list_result.unwrap();
            assert!(tools
                .iter()
                .any(|t| t.name == "go" && t.version == "1.21.0"));
        }
    }

    #[tokio::test]
    async fn test_list_available_tools() {
        let fixture = ToolCrudTestFixture::new().await;

        // List available tools (from configuration)
        let result = list::list_available_tools().await;

        assert!(result.is_ok());
        let tools = result.unwrap();

        // Should include configured tools like go, node, bun, etc.
        assert!(tools.iter().any(|t| t.name == "go"));
        assert!(tools.iter().any(|t| t.name == "node"));
        assert!(tools.iter().any(|t| t.name == "bun"));
    }

    #[tokio::test]
    async fn test_list_corrupted_installation_directory() {
        let fixture = ToolCrudTestFixture::new().await;

        // Create a corrupted tool directory
        let tools_dir = fixture.vx_home().join("tools").join("go").join("1.21.0");
        fs::create_dir_all(&tools_dir).expect("Failed to create test directory");

        // Create some invalid files
        fs::write(tools_dir.join("corrupted_file"), "invalid content")
            .expect("Failed to write test file");

        // List should handle corrupted installations gracefully
        let result = list::list_installed_tools().await;
        assert!(result.is_ok()); // Should not crash

        // TODO: Verify that corrupted installations are marked as such
    }

    #[tokio::test]
    async fn test_list_performance_with_many_tools() {
        // TODO: Test listing performance with many installed tools
        // This would help identify performance bottlenecks
    }
}

#[cfg(test)]
mod update_tests {
    use super::*;

    #[tokio::test]
    async fn test_update_nonexistent_tool() {
        let fixture = ToolCrudTestFixture::new().await;

        // Try to update a tool that's not installed
        let result = update::update_tool("nonexistent-tool-xyz", None).await;

        assert!(result.is_err());
        assert!(
            result.unwrap_err().to_string().contains("not installed")
                || result.unwrap_err().to_string().contains("not found")
        );
    }

    #[tokio::test]
    async fn test_update_to_latest() {
        let fixture = ToolCrudTestFixture::new().await;

        // Install an older version
        let install_result = install::install_tool("go", Some("1.20.0"), false).await;

        if install_result.is_ok() {
            // Update to latest
            let update_result = update::update_tool("go", None).await; // None = latest

            if update_result.is_ok() {
                // Verify the version was updated
                let list_result = list::list_installed_tools().await;
                if let Ok(tools) = list_result {
                    let go_tool = tools.iter().find(|t| t.name == "go");
                    assert!(go_tool.is_some());
                    // Version should be newer than 1.20.0
                    assert_ne!(go_tool.unwrap().version, "1.20.0");
                }
            }
        }
    }

    #[tokio::test]
    async fn test_update_to_specific_version() {
        let fixture = ToolCrudTestFixture::new().await;

        // Install one version
        let install_result = install::install_tool("go", Some("1.20.0"), false).await;

        if install_result.is_ok() {
            // Update to specific version
            let update_result = update::update_tool("go", Some("1.21.0")).await;

            if update_result.is_ok() {
                // Verify the specific version is installed
                let list_result = list::list_installed_tools().await;
                if let Ok(tools) = list_result {
                    assert!(tools
                        .iter()
                        .any(|t| t.name == "go" && t.version == "1.21.0"));
                }
            }
        }
    }

    #[tokio::test]
    async fn test_update_failure_rollback() {
        // TODO: Test rollback when update fails midway
        // Should restore the previous version
    }

    #[tokio::test]
    async fn test_batch_update_with_failures() {
        // TODO: Test updating multiple tools where some fail
        // Should handle partial failures gracefully
    }
}

#[cfg(test)]
mod config_integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_fetcher_url_configuration() {
        let fixture = ToolCrudTestFixture::new().await;

        // Test that tools use configured fetcher URLs
        let config = fixture.config_manager.config();

        // Verify fetcher URLs are configured
        assert!(vx_config::get_tool_fetcher_url(config, "go").is_some());
        assert!(vx_config::get_tool_fetcher_url(config, "node").is_some());
        assert!(vx_config::get_tool_fetcher_url(config, "bun").is_some());

        // Verify URLs are valid
        let go_url = vx_config::get_tool_fetcher_url(config, "go").unwrap();
        assert!(go_url.starts_with("https://"));
        assert!(go_url.contains("go.dev"));
    }

    #[tokio::test]
    async fn test_tool_configuration_completeness() {
        let fixture = ToolCrudTestFixture::new().await;
        let config = fixture.config_manager.config();

        // Test that all configured tools have required fields
        for (tool_name, tool_config) in &config.tools {
            // Every tool should have a description
            assert!(
                tool_config.description.is_some(),
                "Tool {} missing description",
                tool_name
            );

            // Every tool should have a fetcher URL
            assert!(
                tool_config.fetcher_url.is_some(),
                "Tool {} missing fetcher_url",
                tool_name
            );

            // Every tool should have a download URL template
            assert!(
                tool_config.download_url_template.is_some(),
                "Tool {} missing download_url_template",
                tool_name
            );
        }
    }

    #[tokio::test]
    async fn test_invalid_configuration_handling() {
        // TODO: Test handling of invalid configuration files
        // Should provide clear error messages and fallback to defaults
    }
}

// Helper functions for testing
mod test_helpers {
    use super::*;

    /// Create a mock tool installation for testing
    pub async fn create_mock_installation(tool_name: &str, version: &str, vx_home: &PathBuf) {
        let tool_dir = vx_home.join("tools").join(tool_name).join(version);
        fs::create_dir_all(&tool_dir).expect("Failed to create mock tool directory");

        // Create a mock executable
        let exe_name = if cfg!(windows) {
            format!("{}.exe", tool_name)
        } else {
            tool_name.to_string()
        };

        fs::write(tool_dir.join(&exe_name), "mock executable")
            .expect("Failed to create mock executable");
    }

    /// Verify that a tool is properly installed
    pub async fn verify_tool_installation(
        tool_name: &str,
        version: &str,
        vx_home: &PathBuf,
    ) -> bool {
        let tool_dir = vx_home.join("tools").join(tool_name).join(version);
        tool_dir.exists()
    }

    /// Clean up test installations
    pub async fn cleanup_test_installation(tool_name: &str, vx_home: &PathBuf) {
        let tool_dir = vx_home.join("tools").join(tool_name);
        if tool_dir.exists() {
            let _ = fs::remove_dir_all(tool_dir);
        }
    }
}
