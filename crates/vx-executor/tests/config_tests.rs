//! Tests for the executor configuration module

use rstest::rstest;
use std::time::Duration;
use vx_executor::ExecutorConfig;

#[rstest]
fn test_default_config() {
    let config = ExecutorConfig::default();

    assert!(config.auto_install);
    assert!(config.auto_install_dependencies);
    assert!(config.prefer_vx_managed);
    assert!(config.fallback_to_system);
    assert!(config.execution_timeout.is_none());
    assert_eq!(config.install_timeout, Duration::from_secs(300));
    assert!(config.show_progress);
    assert!(!config.prompt_before_install);
    assert_eq!(config.max_parallel_installs, 4);
    assert!(config.verify_after_install);
}

#[rstest]
fn test_without_auto_install() {
    let config = ExecutorConfig::new().without_auto_install();

    assert!(!config.auto_install);
    assert!(!config.auto_install_dependencies);
}

#[rstest]
fn test_with_prompt() {
    let config = ExecutorConfig::new().with_prompt();

    assert!(config.prompt_before_install);
}

#[rstest]
fn test_system_only() {
    let config = ExecutorConfig::new().system_only();

    assert!(!config.prefer_vx_managed);
    assert!(config.fallback_to_system);
    assert!(!config.auto_install);
}

#[rstest]
fn test_with_timeout() {
    let timeout = Duration::from_secs(60);
    let config = ExecutorConfig::new().with_timeout(timeout);

    assert_eq!(config.execution_timeout, Some(timeout));
}

#[rstest]
fn test_quiet_mode() {
    let config = ExecutorConfig::new().quiet();

    assert!(!config.show_progress);
}

#[rstest]
fn test_chained_builders() {
    let config = ExecutorConfig::new()
        .without_auto_install()
        .with_prompt()
        .quiet()
        .with_timeout(Duration::from_secs(120));

    assert!(!config.auto_install);
    assert!(config.prompt_before_install);
    assert!(!config.show_progress);
    assert_eq!(config.execution_timeout, Some(Duration::from_secs(120)));
}
