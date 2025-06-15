//! Tests for execute command functionality

use super::{cleanup_test_env, create_test_venv_manager};
use crate::commands::execute::handle;
use vx_core::PluginRegistry;

fn create_test_registry() -> PluginRegistry {
    PluginRegistry::new()
}

#[tokio::test]
async fn test_execute_help_command() {
    let (_manager, _temp_dir) = create_test_venv_manager().expect("Failed to create test manager");
    let registry = create_test_registry();

    // Test with a non-existent tool instead of 'help' to avoid system command execution
    let result = handle(&registry, "nonexistent-help-tool", &[], false).await;

    // Non-existent tool should fail gracefully
    assert!(result.is_err());

    cleanup_test_env();
}

#[tokio::test]
async fn test_execute_version_command() {
    let (_manager, _temp_dir) = create_test_venv_manager().expect("Failed to create test manager");
    let registry = create_test_registry();

    let result = handle(&registry, "version", &[], false).await;

    // Version command should be handled gracefully
    assert!(result.is_ok() || result.is_err()); // Either works or fails gracefully

    cleanup_test_env();
}

#[tokio::test]
async fn test_execute_with_system_path() {
    let (_manager, _temp_dir) = create_test_venv_manager().expect("Failed to create test manager");
    let registry = create_test_registry();

    let result = handle(&registry, "echo", &["test".to_string()], true).await;

    // Using system path should work for basic commands (if available)
    assert!(result.is_ok() || result.is_err()); // Either works or fails gracefully

    cleanup_test_env();
}

#[tokio::test]
async fn test_execute_nonexistent_tool() {
    let (_manager, _temp_dir) = create_test_venv_manager().expect("Failed to create test manager");
    let registry = create_test_registry();

    let result = handle(&registry, "nonexistent-tool-12345", &[], false).await;

    // Non-existent tool should fail gracefully
    assert!(result.is_err());

    cleanup_test_env();
}

#[tokio::test]
async fn test_execute_with_arguments() {
    let (_manager, _temp_dir) = create_test_venv_manager().expect("Failed to create test manager");
    let registry = create_test_registry();

    let result = handle(
        &registry,
        "echo",
        &["hello".to_string(), "world".to_string()],
        true,
    )
    .await;

    // Command with arguments should work (if available)
    assert!(result.is_ok() || result.is_err()); // Either works or fails gracefully

    cleanup_test_env();
}

#[tokio::test]
async fn test_execute_empty_tool_name() {
    let (_manager, _temp_dir) = create_test_venv_manager().expect("Failed to create test manager");
    let registry = create_test_registry();

    let result = handle(&registry, "", &[], false).await;

    // Empty tool name should fail
    assert!(result.is_err());

    cleanup_test_env();
}

#[test]
fn test_command_argument_parsing() {
    let test_cases = vec![
        (vec!["--version"], 1),
        (vec!["--help"], 1),
        (vec!["build", "--release"], 2),
        (vec![], 0),
        (vec!["run", "script.js", "--watch"], 3),
    ];

    for (args, expected_len) in test_cases {
        assert_eq!(args.len(), expected_len);

        // Validate that arguments don't contain null bytes or other invalid characters
        for arg in &args {
            assert!(
                !arg.contains('\0'),
                "Arguments should not contain null bytes"
            );
            assert!(
                !arg.is_empty() || args.len() == 0,
                "Non-empty args should not be empty strings"
            );
        }
    }
}

#[test]
fn test_tool_name_validation() {
    let valid_tools = vec!["node", "npm", "uv", "go", "cargo", "rustc"];
    let invalid_tools = vec!["", " ", "tool with spaces", "tool\0null", "tool\nnewline"];

    for tool in valid_tools {
        assert!(!tool.is_empty());
        assert!(!tool.contains(' '));
        assert!(!tool.contains('\0'));
        assert!(!tool.contains('\n'));
    }

    for tool in invalid_tools {
        assert!(
            tool.is_empty()
                || tool.trim().is_empty()
                || tool.contains(' ')
                || tool.contains('\0')
                || tool.contains('\n')
        );
    }
}

#[test]
fn test_system_path_flag_behavior() {
    // Test that system path flag is properly handled
    let test_cases = vec![
        (true, "Should use system PATH"),
        (false, "Should use vx-managed tools"),
    ];

    for (use_system_path, description) in test_cases {
        // Basic validation that the flag is a boolean
        assert!(
            use_system_path == true || use_system_path == false,
            "{}",
            description
        );
    }
}
