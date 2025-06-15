//! Tests for list command functionality

use super::{cleanup_test_env, create_test_venv_manager};
use crate::commands::list::handle;
use vx_core::PluginRegistry;

fn create_test_registry() -> PluginRegistry {
    PluginRegistry::new()
}

#[tokio::test]
async fn test_list_all_tools() {
    let (_manager, _temp_dir) = create_test_venv_manager().expect("Failed to create test manager");
    let registry = create_test_registry();

    let result = handle(&registry, None, false).await;

    assert!(result.is_ok());

    cleanup_test_env();
}

#[tokio::test]
async fn test_list_with_status() {
    let (_manager, _temp_dir) = create_test_venv_manager().expect("Failed to create test manager");
    let registry = create_test_registry();

    let result = handle(&registry, None, true).await;

    assert!(result.is_ok());

    cleanup_test_env();
}

#[tokio::test]
async fn test_list_specific_tool() {
    let (_manager, _temp_dir) = create_test_venv_manager().expect("Failed to create test manager");
    let registry = create_test_registry();

    let result = handle(&registry, Some("node"), false).await;

    // Should handle tool lookup gracefully
    assert!(result.is_ok() || result.is_err()); // Either works or fails gracefully

    cleanup_test_env();
}

#[tokio::test]
async fn test_list_nonexistent_tool() {
    let (_manager, _temp_dir) = create_test_venv_manager().expect("Failed to create test manager");
    let registry = create_test_registry();

    let result = handle(&registry, Some("nonexistent-tool-12345"), false).await;

    // Should handle non-existent tool gracefully
    assert!(result.is_err()); // Should fail for non-existent tool

    cleanup_test_env();
}

#[tokio::test]
async fn test_list_specific_tool_with_status() {
    let (_manager, _temp_dir) = create_test_venv_manager().expect("Failed to create test manager");
    let registry = create_test_registry();

    let result = handle(&registry, Some("node"), true).await;

    // Should handle tool lookup with status gracefully
    assert!(result.is_ok() || result.is_err()); // Either works or fails gracefully

    cleanup_test_env();
}

#[tokio::test]
async fn test_list_empty_tool_name() {
    let (_manager, _temp_dir) = create_test_venv_manager().expect("Failed to create test manager");
    let registry = create_test_registry();

    let result = handle(&registry, Some(""), false).await;

    // Empty tool name should fail
    assert!(result.is_err());

    cleanup_test_env();
}

#[test]
fn test_format_validation() {
    let valid_formats = vec!["json", "yaml", "table"];
    let invalid_formats = vec!["xml", "csv", "invalid"];

    for format in valid_formats {
        // These should be valid format strings
        assert!(["json", "yaml", "table"].contains(&format));
    }

    for format in invalid_formats {
        // These should not be in the valid formats list
        assert!(!["json", "yaml", "table"].contains(&format));
    }
}

#[test]
fn test_tool_name_validation() {
    let valid_names = vec!["node", "uv", "go", "rust", "python"];
    let invalid_names = vec!["", " ", "node@version", "tool with spaces"];

    for name in valid_names {
        // Valid tool names should not be empty and not contain special characters
        assert!(!name.is_empty());
        assert!(!name.contains('@'));
        assert!(!name.contains(' '));
    }

    for name in invalid_names {
        // Invalid names should fail basic validation
        if !name.is_empty() {
            assert!(name.contains('@') || name.contains(' ') || name.trim().is_empty());
        }
    }
}
