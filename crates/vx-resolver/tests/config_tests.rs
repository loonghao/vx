//! Tests for resolver configuration

use rstest::rstest;
use std::time::Duration;
use vx_resolver::ResolverConfig;

#[rstest]
fn test_default_config() {
    let config = ResolverConfig::default();
    assert!(config.auto_install);
    assert!(config.auto_install_dependencies);
    assert!(config.prefer_vx_managed);
    assert!(config.fallback_to_system);
}

#[rstest]
fn test_config_builders() {
    let config = ResolverConfig::new()
        .without_auto_install()
        .with_prompt()
        .quiet();

    assert!(!config.auto_install);
    assert!(config.prompt_before_install);
    assert!(!config.show_progress);
}

#[rstest]
fn test_system_only_config() {
    let config = ResolverConfig::new().system_only();

    assert!(!config.prefer_vx_managed);
    assert!(config.fallback_to_system);
    assert!(!config.auto_install);
}

#[rstest]
fn test_timeout_config() {
    let config = ResolverConfig::new().with_timeout(Duration::from_secs(60));

    assert_eq!(config.execution_timeout, Some(Duration::from_secs(60)));
}
