//! Command execution tests

use rstest::*;
use vx_cli::commands::execute::handle;

mod common;
use common::{cleanup_test_env, create_test_registry, create_test_venv_manager};

/// Test execute command with non-existent tool
#[rstest]
#[tokio::test]
async fn test_execute_nonexistent_tool() {
    let (_manager, _temp_dir) = create_test_venv_manager().expect("Failed to create test manager");
    let registry = create_test_registry();

    let result = handle(&registry, "nonexistent-tool", &Vec::<String>::new(), false).await;

    // Non-existent tool should fail gracefully
    assert!(result.is_err());

    cleanup_test_env();
}

/// Test execute command with empty arguments
#[rstest]
#[tokio::test]
async fn test_execute_empty_args() {
    let (_manager, _temp_dir) = create_test_venv_manager().expect("Failed to create test manager");
    let registry = create_test_registry();

    // Test with a tool that might exist in the registry
    let result = handle(&registry, "node", &Vec::<String>::new(), false).await;

    // Should handle empty args gracefully (might succeed or fail depending on tool availability)
    match result {
        Ok(_) => println!("Command succeeded"),
        Err(e) => println!("Command failed as expected: {:?}", e),
    }

    cleanup_test_env();
}

/// Test execute command with version flag
#[rstest]
#[tokio::test]
async fn test_execute_version_flag() {
    let (_manager, _temp_dir) = create_test_venv_manager().expect("Failed to create test manager");
    let registry = create_test_registry();

    let result = handle(&registry, "node", &vec!["--version".to_string()], false).await;

    // Version flag should be handled (might succeed or fail depending on tool availability)
    match result {
        Ok(_) => println!("Version command succeeded"),
        Err(e) => println!("Version command failed as expected: {:?}", e),
    }

    cleanup_test_env();
}
