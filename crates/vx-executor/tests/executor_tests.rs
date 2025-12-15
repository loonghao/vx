//! Tests for the dynamic executor module

use rstest::rstest;
use vx_executor::{DynamicExecutor, ExecutorConfig};

#[rstest]
fn test_executor_creation() {
    let config = ExecutorConfig::default();
    let executor = DynamicExecutor::new(config);
    assert!(executor.is_ok());
}

#[rstest]
fn test_executor_config_access() {
    let config = ExecutorConfig::default().without_auto_install();
    let executor = DynamicExecutor::new(config).unwrap();

    assert!(!executor.config().auto_install);
}

#[rstest]
fn test_executor_resolver_access() {
    let config = ExecutorConfig::default();
    let executor = DynamicExecutor::new(config).unwrap();

    // Should be able to access the resolver
    let resolver = executor.resolver();
    assert!(resolver.is_known_tool("node"));
}

#[rstest]
fn test_config_system_only() {
    let config = ExecutorConfig::default().system_only();
    let executor = DynamicExecutor::new(config).unwrap();

    assert!(!executor.config().prefer_vx_managed);
    assert!(executor.config().fallback_to_system);
    assert!(!executor.config().auto_install);
}

#[rstest]
fn test_config_with_timeout() {
    use std::time::Duration;

    let config = ExecutorConfig::default().with_timeout(Duration::from_secs(30));
    let executor = DynamicExecutor::new(config).unwrap();

    assert_eq!(
        executor.config().execution_timeout,
        Some(Duration::from_secs(30))
    );
}

#[rstest]
fn test_config_quiet_mode() {
    let config = ExecutorConfig::default().quiet();
    let executor = DynamicExecutor::new(config).unwrap();

    assert!(!executor.config().show_progress);
}

#[rstest]
fn test_config_with_prompt() {
    let config = ExecutorConfig::default().with_prompt();
    let executor = DynamicExecutor::new(config).unwrap();

    assert!(executor.config().prompt_before_install);
}

#[tokio::test]
async fn test_execute_system_tool_not_found() {
    // Try to execute a non-existent tool
    let result =
        vx_executor::execute_system_tool("nonexistent-tool-xyz-12345", &["--version".to_string()])
            .await;

    // Should fail because tool doesn't exist
    assert!(result.is_err());
}
