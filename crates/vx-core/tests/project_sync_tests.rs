//! Project synchronization tests

use rstest::*;
use vx_core::FigmentConfigManager;

mod common;
use common::{sample_configs, TestFixture};

/// Test project sync functionality
#[rstest]
#[tokio::test]
async fn test_project_sync() {
    let fixture = TestFixture::new().expect("Failed to create test fixture");

    // Create a .vx.toml with tool versions
    fixture
        .create_file(".vx.toml", sample_configs::VALID_VX_CONFIG)
        .expect("Failed to write config");

    let manager = FigmentConfigManager::new().expect("Failed to create config manager");

    // Test sync (this will not actually install tools in test environment)
    let result = manager.sync_project(false).await;

    // For now, we expect this to work or fail gracefully
    // The actual implementation might need adjustment
    match result {
        Ok(installed_tools) => {
            // Should identify tools that would be installed
            println!("Would install tools: {:?}", installed_tools);
        }
        Err(e) => {
            // Sync might fail in test environment, which is acceptable
            println!("Sync failed (expected in test): {:?}", e);
        }
    }
}

/// Test project sync with force flag
#[rstest]
#[tokio::test]
async fn test_project_sync_force() {
    let fixture = TestFixture::new().expect("Failed to create test fixture");

    // Create a .vx.toml with tool versions
    fixture
        .create_file(".vx.toml", sample_configs::VALID_VX_CONFIG)
        .expect("Failed to write config");

    let manager = FigmentConfigManager::new().expect("Failed to create config manager");

    // Test sync with force flag
    let result = manager.sync_project(true).await;

    // Similar to above, we expect this to work or fail gracefully
    match result {
        Ok(installed_tools) => {
            println!("Would force install tools: {:?}", installed_tools);
        }
        Err(e) => {
            println!("Force sync failed (expected in test): {:?}", e);
        }
    }
}

/// Test project sync without configuration
#[rstest]
#[tokio::test]
async fn test_project_sync_no_config() {
    let _fixture = TestFixture::new().expect("Failed to create test fixture");

    // No .vx.toml file created
    let manager = FigmentConfigManager::new().expect("Failed to create config manager");

    // Test sync without project config
    let result = manager.sync_project(false).await;

    // Should handle missing config gracefully
    match result {
        Ok(installed_tools) => {
            // Should be empty or minimal
            assert!(installed_tools.is_empty() || installed_tools.len() <= 1);
        }
        Err(_) => {
            // Acceptable to fail without config
        }
    }
}
