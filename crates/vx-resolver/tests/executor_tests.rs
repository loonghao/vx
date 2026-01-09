//! Tests for executor

use rstest::rstest;
use vx_manifest::ProviderManifest;
use vx_resolver::{Executor, ResolverConfig, RuntimeMap};
use vx_runtime::{mock_context, registry::ProviderRegistry};

/// Create a test RuntimeMap from manifests
fn create_test_runtime_map() -> RuntimeMap {
    let toml = r#"
[provider]
name = "node"
ecosystem = "nodejs"

[[runtimes]]
name = "node"
description = "Node.js"
executable = "node"

[[runtimes]]
name = "npm"
description = "NPM"
executable = "npm"
bundled_with = "node"
"#;
    let manifest = ProviderManifest::parse(toml).expect("Failed to parse manifest");
    RuntimeMap::from_manifests(&[manifest])
}

#[tokio::test]
async fn test_executor_creation() {
    let config = ResolverConfig::default();
    let registry = ProviderRegistry::new();
    let context = mock_context();
    let runtime_map = create_test_runtime_map();
    let executor = Executor::new(config, &registry, &context, runtime_map);
    assert!(executor.is_ok());
}

#[tokio::test]
async fn test_executor_with_disabled_auto_install() {
    let config = ResolverConfig::default().without_auto_install();
    let registry = ProviderRegistry::new();
    let context = mock_context();
    let runtime_map = create_test_runtime_map();
    let executor = Executor::new(config, &registry, &context, runtime_map).unwrap();
    assert!(!executor.config().auto_install);
}

#[rstest]
fn test_executor_resolver_access() {
    let config = ResolverConfig::default();
    let registry = ProviderRegistry::new();
    let context = mock_context();
    let runtime_map = create_test_runtime_map();
    let executor = Executor::new(config, &registry, &context, runtime_map).unwrap();

    // Should be able to access the resolver
    let resolver = executor.resolver();
    assert!(resolver.is_known_runtime("node"));
}
