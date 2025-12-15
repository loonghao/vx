//! Tests for provider system functionality

use crate::test_utils::*;
use std::sync::Arc;
use vx_runtime::{Provider, ProviderRegistry};

#[test]
fn test_provider_registry_creation() {
    let registry = ProviderRegistry::new();

    // Registry should be created successfully
    let providers = registry.providers();
    assert!(providers.is_empty()); // Should start empty
}

#[test]
fn test_provider_registration() {
    let registry = ProviderRegistry::new();

    // Create a mock provider
    let provider =
        MockProvider::new("test-provider").with_runtime(MockRuntime::new("test-runtime", "1.0.0"));

    // Register the provider
    registry.register(Arc::new(provider));

    // Check that provider is registered
    let providers = registry.providers();
    assert_eq!(providers.len(), 1);
    assert_eq!(providers[0].name(), "test-provider");
}

#[test]
fn test_multiple_provider_registration() {
    let registry = ProviderRegistry::new();

    // Register multiple providers
    let provider1 =
        MockProvider::new("provider1").with_runtime(MockRuntime::new("runtime1", "1.0.0"));
    let provider2 =
        MockProvider::new("provider2").with_runtime(MockRuntime::new("runtime2", "2.0.0"));

    registry.register(Arc::new(provider1));
    registry.register(Arc::new(provider2));

    let providers = registry.providers();
    assert_eq!(providers.len(), 2);
}

#[test]
fn test_runtime_discovery() {
    let registry = ProviderRegistry::new();

    // Create provider with multiple runtimes
    let provider = MockProvider::new("multi-runtime-provider")
        .with_runtime(MockRuntime::new("node", "18.0.0"))
        .with_runtime(MockRuntime::new("npm", "8.0.0"))
        .with_runtime(MockRuntime::new("yarn", "1.22.0"));

    registry.register(Arc::new(provider));

    // Test runtime discovery
    let runtimes = registry.runtime_names();
    assert_eq!(runtimes.len(), 3);
    assert!(runtimes.contains(&"node".to_string()));
    assert!(runtimes.contains(&"npm".to_string()));
    assert!(runtimes.contains(&"yarn".to_string()));
}

#[test]
fn test_get_runtime() {
    let registry = ProviderRegistry::new();

    let provider =
        MockProvider::new("test-provider").with_runtime(MockRuntime::new("test-runtime", "1.0.0"));

    registry.register(Arc::new(provider));

    // Test getting existing runtime
    let runtime = registry.get_runtime("test-runtime");
    assert!(runtime.is_some());

    let runtime = runtime.unwrap();
    assert_eq!(runtime.name(), "test-runtime");

    // Test getting non-existent runtime
    let missing_runtime = registry.get_runtime("missing-runtime");
    assert!(missing_runtime.is_none());
}

#[test]
fn test_mock_provider_creation() {
    let provider = MockProvider::new("test-provider");
    assert_eq!(provider.name(), "test-provider");
    assert_eq!(provider.description(), "Mock provider for testing");
    assert!(provider.runtimes().is_empty());
}

#[test]
fn test_mock_provider_with_runtimes() {
    let runtime1 = MockRuntime::new("runtime1", "1.0.0");
    let runtime2 = MockRuntime::new("runtime2", "2.0.0");

    let provider = MockProvider::new("test-provider")
        .with_runtime(runtime1)
        .with_runtime(runtime2);

    let runtimes = provider.runtimes();
    assert_eq!(runtimes.len(), 2);
    assert_eq!(runtimes[0].name(), "runtime1");
    assert_eq!(runtimes[1].name(), "runtime2");
}

#[test]
fn test_mock_runtime_properties() {
    let runtime = MockRuntime::new("test-runtime", "1.0.0");

    assert_eq!(runtime.name, "test-runtime");
    assert_eq!(runtime.version, "1.0.0");
    assert!(runtime.executable_path.is_none());
}

#[test]
fn test_mock_runtime_with_executable() {
    let path = std::path::PathBuf::from("/usr/bin/test-runtime");
    let runtime = MockRuntime::new("test-runtime", "1.0.0").with_executable(path.clone());

    assert_eq!(runtime.name, "test-runtime");
    assert_eq!(runtime.executable_path, Some(path));
}

#[test]
fn test_mock_runtime_failure_mode() {
    let runtime = MockRuntime::new("failing-runtime", "1.0.0").with_failure();

    assert!(runtime.should_fail);
}

#[test]
fn test_provider_registry_runtime_lookup_performance() {
    let registry = ProviderRegistry::new();

    // Register many runtimes to test lookup performance
    for i in 0..100 {
        let provider = MockProvider::new(&format!("provider-{}", i))
            .with_runtime(MockRuntime::new(&format!("runtime-{}", i), "1.0.0"));
        registry.register(Arc::new(provider));
    }

    // Test that lookup is still fast
    let start = std::time::Instant::now();
    let runtime = registry.get_runtime("runtime-50");
    let duration = start.elapsed();

    assert!(runtime.is_some());
    assert!(duration.as_millis() < 10); // Should be very fast
}

#[test]
fn test_provider_registry_duplicate_runtimes() {
    let registry = ProviderRegistry::new();

    // Register two providers with the same runtime name
    let provider1 =
        MockProvider::new("provider1").with_runtime(MockRuntime::new("common-runtime", "1.0.0"));
    let provider2 =
        MockProvider::new("provider2").with_runtime(MockRuntime::new("common-runtime", "2.0.0"));

    registry.register(Arc::new(provider1));
    registry.register(Arc::new(provider2));

    // Should handle duplicate runtime names gracefully
    let runtime = registry.get_runtime("common-runtime");
    assert!(runtime.is_some());

    // The behavior for duplicate runtimes depends on implementation
    // This test ensures it doesn't panic
}

#[test]
fn test_empty_provider_registration() {
    let registry = ProviderRegistry::new();

    // Register a provider with no runtimes
    let empty_provider = MockProvider::new("empty-provider");
    registry.register(Arc::new(empty_provider));

    let providers = registry.providers();
    assert_eq!(providers.len(), 1);
    assert_eq!(providers[0].name(), "empty-provider");

    let runtimes = registry.runtime_names();
    assert!(runtimes.is_empty());
}
