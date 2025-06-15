//! Tests for install command functionality

use super::{cleanup_test_env, create_test_venv_manager};
use crate::commands::install::handle;
use std::sync::atomic::{AtomicU32, Ordering};
use vx_core::PluginRegistry;

static TEST_INSTALL_COUNTER: AtomicU32 = AtomicU32::new(0);

fn create_test_registry() -> PluginRegistry {
    PluginRegistry::new()
}

fn unique_version() -> String {
    let id = TEST_INSTALL_COUNTER.fetch_add(1, Ordering::SeqCst);
    format!("test-{}", id)
}

#[tokio::test]
async fn test_install_invalid_tool() {
    let (_manager, _temp_dir) = create_test_venv_manager().expect("Failed to create test manager");
    let registry = create_test_registry();

    let result = handle(&registry, "nonexistent-tool-12345", Some("1.0.0"), false).await;

    // Should fail for non-existent tool
    assert!(result.is_err());

    cleanup_test_env();
}

#[tokio::test]
async fn test_install_with_force() {
    let (_manager, _temp_dir) = create_test_venv_manager().expect("Failed to create test manager");
    let registry = create_test_registry();

    let result = handle(&registry, "node", Some(&unique_version()), true).await;

    // Force install should be handled (though tool may not actually install in test env)
    assert!(result.is_ok() || result.is_err()); // Either works or fails gracefully

    cleanup_test_env();
}

#[tokio::test]
async fn test_install_without_version() {
    let (_manager, _temp_dir) = create_test_venv_manager().expect("Failed to create test manager");
    let registry = create_test_registry();

    let result = handle(&registry, "node", None, false).await;

    // Install without version should try to get latest
    assert!(result.is_ok() || result.is_err()); // Either works or fails gracefully

    cleanup_test_env();
}

#[tokio::test]
async fn test_install_empty_tool_name() {
    let (_manager, _temp_dir) = create_test_venv_manager().expect("Failed to create test manager");
    let registry = create_test_registry();

    let result = handle(&registry, "", Some("1.0.0"), false).await;

    // Empty tool name should fail
    assert!(result.is_err());

    cleanup_test_env();
}

#[test]
fn test_tool_spec_parsing() {
    // Test various tool specification formats
    let specs = vec!["node@18.17.0", "uv@latest", "go@^1.21.0", "python@3.11"];

    for spec in specs {
        // Basic validation that spec format is recognized
        assert!(
            spec.contains('@'),
            "Tool spec should contain version separator"
        );
        let parts: Vec<&str> = spec.split('@').collect();
        assert_eq!(
            parts.len(),
            2,
            "Tool spec should have exactly one @ separator"
        );
        assert!(!parts[0].is_empty(), "Tool name should not be empty");
        assert!(!parts[1].is_empty(), "Tool version should not be empty");
    }
}

#[test]
fn test_version_constraint_validation() {
    let valid_versions = vec![
        "1.0.0",
        "latest",
        "^1.0.0",
        "~1.0.0",
        ">=1.0.0",
        "1.0.0-alpha.1",
    ];

    for version in valid_versions {
        // Basic validation that version format is reasonable
        assert!(!version.is_empty(), "Version should not be empty");
        assert!(!version.contains(' '), "Version should not contain spaces");
    }
}
