//! Tests for venv command functionality

use super::{cleanup_test_env, create_test_venv_manager};
use crate::commands::venv_cmd::{handle, VenvCommand};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Mutex;
use vx_core::VenvManager;

static TEST_ENV_COUNTER: AtomicU32 = AtomicU32::new(0);
static TEST_LOCK: Mutex<()> = Mutex::new(());

fn unique_env_name(prefix: &str) -> String {
    let id = TEST_ENV_COUNTER.fetch_add(1, Ordering::SeqCst);
    let thread_id = std::thread::current().id();
    format!("{}-{}-{:?}", prefix, id, thread_id)
}

#[tokio::test]
async fn test_venv_list_empty() {
    let (_manager, _temp_dir) = create_test_venv_manager().expect("Failed to create test manager");

    let result = handle(VenvCommand::List).await;
    assert!(result.is_ok());

    cleanup_test_env();
}

#[tokio::test]
async fn test_venv_current_none() {
    let (_manager, _temp_dir) = create_test_venv_manager().expect("Failed to create test manager");

    let result = handle(VenvCommand::Current).await;
    assert!(result.is_ok());

    cleanup_test_env();
}

#[tokio::test]
async fn test_venv_create_empty() {
    let (_manager, _temp_dir) = create_test_venv_manager().expect("Failed to create test manager");

    let result = handle(VenvCommand::Create {
        name: unique_env_name("test-env-empty"),
        tools: vec![],
    })
    .await;
    assert!(result.is_ok());

    cleanup_test_env();
}

#[tokio::test]
async fn test_venv_create_with_tools() {
    let (_manager, _temp_dir) = create_test_venv_manager().expect("Failed to create test manager");

    let result = handle(VenvCommand::Create {
        name: unique_env_name("test-env-tools"),
        tools: vec!["node@18.0.0".to_string(), "uv@latest".to_string()],
    })
    .await;
    assert!(result.is_ok());

    cleanup_test_env();
}

#[tokio::test]
async fn test_venv_create_duplicate() {
    let _lock = TEST_LOCK.lock().unwrap();
    let (manager, _temp_dir) = create_test_venv_manager().expect("Failed to create test manager");

    let env_name = unique_env_name("duplicate-env");

    // Create first environment directly using manager
    let result1 = manager.create(&env_name, &[]);
    assert!(result1.is_ok(), "First environment creation should succeed");

    // Try to create duplicate - should fail
    let result2 = manager.create(&env_name, &[]);
    assert!(
        result2.is_err(),
        "Duplicate environment creation should fail"
    );

    cleanup_test_env();
}

#[tokio::test]
async fn test_venv_remove_nonexistent() {
    let (_manager, _temp_dir) = create_test_venv_manager().expect("Failed to create test manager");

    let result = handle(VenvCommand::Remove {
        name: unique_env_name("nonexistent-env"),
        force: true,
    })
    .await;
    assert!(result.is_err());

    cleanup_test_env();
}

#[tokio::test]
async fn test_venv_remove_without_force() {
    let (_manager, _temp_dir) = create_test_venv_manager().expect("Failed to create test manager");

    let env_name = unique_env_name("remove-test-env");

    // Create environment first
    let _create_result = handle(VenvCommand::Create {
        name: env_name.clone(),
        tools: vec![],
    })
    .await;

    // Try to remove without force - should not actually remove
    let result = handle(VenvCommand::Remove {
        name: env_name,
        force: false,
    })
    .await;
    assert!(result.is_ok());

    cleanup_test_env();
}

#[tokio::test]
async fn test_venv_activate_nonexistent() {
    let (_manager, _temp_dir) = create_test_venv_manager().expect("Failed to create test manager");

    let result = handle(VenvCommand::Activate {
        name: unique_env_name("nonexistent-env"),
    })
    .await;
    assert!(result.is_err());

    cleanup_test_env();
}

#[tokio::test]
async fn test_venv_deactivate_when_none_active() {
    let (_manager, _temp_dir) = create_test_venv_manager().expect("Failed to create test manager");

    let result = handle(VenvCommand::Deactivate).await;
    assert!(result.is_ok());

    cleanup_test_env();
}

#[test]
fn test_venv_manager_static_methods() {
    // Test static methods without environment variables
    assert!(!VenvManager::is_active());
    assert!(VenvManager::current().is_none());

    // Test with environment variable set
    std::env::set_var("VX_VENV", "test-env");
    assert!(VenvManager::is_active());
    assert_eq!(VenvManager::current(), Some("test-env".to_string()));

    // Clean up
    std::env::remove_var("VX_VENV");
}
