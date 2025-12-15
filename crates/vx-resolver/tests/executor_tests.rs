//! Tests for executor

use rstest::rstest;
use vx_resolver::{Executor, ResolverConfig};

#[tokio::test]
async fn test_executor_creation() {
    let config = ResolverConfig::default();
    let executor = Executor::new(config);
    assert!(executor.is_ok());
}

#[tokio::test]
async fn test_executor_with_disabled_auto_install() {
    let config = ResolverConfig::default().without_auto_install();
    let executor = Executor::new(config).unwrap();
    assert!(!executor.config().auto_install);
}

#[rstest]
fn test_executor_resolver_access() {
    let config = ResolverConfig::default();
    let executor = Executor::new(config).unwrap();

    // Should be able to access the resolver
    let resolver = executor.resolver();
    assert!(resolver.is_known_runtime("node"));
}
