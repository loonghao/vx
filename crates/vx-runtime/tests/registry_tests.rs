//! Provider registry tests

use async_trait::async_trait;
use std::sync::Arc;
use vx_runtime::{Provider, ProviderRegistry, Runtime, RuntimeContext, VersionInfo};

/// Test runtime
struct TestRuntime {
    name: &'static str,
    aliases: &'static [&'static str],
}

#[async_trait]
impl Runtime for TestRuntime {
    fn name(&self) -> &str {
        self.name
    }

    fn aliases(&self) -> &[&str] {
        self.aliases
    }

    async fn fetch_versions(&self, _ctx: &RuntimeContext) -> anyhow::Result<Vec<VersionInfo>> {
        Ok(vec![VersionInfo::new("1.0.0")])
    }
}

/// Test provider
struct TestProvider {
    name: &'static str,
    runtimes: Vec<Arc<dyn Runtime>>,
}

impl TestProvider {
    fn new(name: &'static str, runtimes: Vec<Arc<dyn Runtime>>) -> Self {
        Self { name, runtimes }
    }
}

impl Provider for TestProvider {
    fn name(&self) -> &str {
        self.name
    }

    fn description(&self) -> &str {
        "Test provider"
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        self.runtimes.clone()
    }
}

#[test]
fn test_registry_new() {
    let registry = ProviderRegistry::new();
    assert!(registry.providers().is_empty());
}

#[test]
fn test_registry_register() {
    let registry = ProviderRegistry::new();

    let provider = Arc::new(TestProvider::new(
        "test",
        vec![Arc::new(TestRuntime {
            name: "test-runtime",
            aliases: &[],
        })],
    ));

    registry.register(provider);

    assert_eq!(registry.providers().len(), 1);
}

#[test]
fn test_registry_get_runtime() {
    let registry = ProviderRegistry::new();

    let provider = Arc::new(TestProvider::new(
        "node",
        vec![
            Arc::new(TestRuntime {
                name: "node",
                aliases: &["nodejs"],
            }),
            Arc::new(TestRuntime {
                name: "npm",
                aliases: &[],
            }),
        ],
    ));

    registry.register(provider);

    // Get by name
    let runtime = registry.get_runtime("node");
    assert!(runtime.is_some());
    assert_eq!(runtime.unwrap().name(), "node");

    // Get by alias
    let runtime = registry.get_runtime("nodejs");
    assert!(runtime.is_some());
    assert_eq!(runtime.unwrap().name(), "node");

    // Get another runtime
    let runtime = registry.get_runtime("npm");
    assert!(runtime.is_some());
    assert_eq!(runtime.unwrap().name(), "npm");

    // Non-existent runtime
    let runtime = registry.get_runtime("nonexistent");
    assert!(runtime.is_none());
}

#[test]
fn test_registry_supports() {
    let registry = ProviderRegistry::new();

    let provider = Arc::new(TestProvider::new(
        "node",
        vec![Arc::new(TestRuntime {
            name: "node",
            aliases: &["nodejs"],
        })],
    ));

    registry.register(provider);

    assert!(registry.supports("node"));
    assert!(registry.supports("nodejs"));
    assert!(!registry.supports("go"));
}

#[test]
fn test_registry_runtime_names() {
    let registry = ProviderRegistry::new();

    let provider = Arc::new(TestProvider::new(
        "node",
        vec![
            Arc::new(TestRuntime {
                name: "node",
                aliases: &[],
            }),
            Arc::new(TestRuntime {
                name: "npm",
                aliases: &[],
            }),
        ],
    ));

    registry.register(provider);

    let names = registry.runtime_names();
    assert!(names.contains(&"node".to_string()));
    assert!(names.contains(&"npm".to_string()));
}

#[test]
fn test_registry_clear() {
    let registry = ProviderRegistry::new();

    let provider = Arc::new(TestProvider::new(
        "test",
        vec![Arc::new(TestRuntime {
            name: "test",
            aliases: &[],
        })],
    ));

    registry.register(provider);
    assert_eq!(registry.providers().len(), 1);

    registry.clear();
    assert!(registry.providers().is_empty());
}

#[test]
fn test_registry_multiple_providers() {
    let registry = ProviderRegistry::new();

    let node_provider = Arc::new(TestProvider::new(
        "node",
        vec![Arc::new(TestRuntime {
            name: "node",
            aliases: &[],
        })],
    ));

    let go_provider = Arc::new(TestProvider::new(
        "go",
        vec![Arc::new(TestRuntime {
            name: "go",
            aliases: &["golang"],
        })],
    ));

    registry.register(node_provider);
    registry.register(go_provider);

    assert_eq!(registry.providers().len(), 2);
    assert!(registry.supports("node"));
    assert!(registry.supports("go"));
    assert!(registry.supports("golang"));
}
