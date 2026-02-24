//! Tests for ProviderRegistry (formerly ManifestRegistry)
//!
//! These tests verify the provider registry functionality using the current API.

use std::sync::Arc;

use async_trait::async_trait;
use vx_manifest::{Ecosystem, ProviderManifest, ProviderMeta, RuntimeDef};
use vx_runtime::{Provider, ProviderRegistry, Runtime, RuntimeContext, VersionInfo};

/// Create a minimal RuntimeDef for testing
fn make_runtime_def(name: &str) -> RuntimeDef {
    RuntimeDef {
        name: name.to_string(),
        description: None,
        executable: name.to_string(),
        aliases: vec![],
        bundled: None,
        bundled_with: None,
        managed_by: None,
        command_prefix: vec![],
        constraints: vec![],
        hooks: None,
        platforms: None,
        platform_constraint: None,
        versions: None,
        executable_config: None,
        layout: None,
        download: None,
        priority: None,
        auto_installable: None,
        env_config: None,
        detection: None,
        health: None,
        cache: None,
        mirrors: vec![],
        mirror_strategy: None,
        commands: vec![],
        output: None,
        shell: None,
        test: None,
        system_deps: None,
        system_install: None,
        normalize: None,
        version_ranges: None,
    }
}

fn make_manifest(name: &str, description: &str) -> ProviderManifest {
    ProviderManifest {
        provider: ProviderMeta {
            name: name.to_string(),
            description: Some(description.to_string()),
            homepage: None,
            repository: None,
            ecosystem: None,
            platform_constraint: None,
            package_alias: None,
        },
        runtimes: vec![make_runtime_def(name)],
    }
}

struct DummyRuntime {
    name: &'static str,
    aliases: &'static [&'static str],
}

#[async_trait]
impl Runtime for DummyRuntime {
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

struct DummyProvider {
    name: &'static str,
    runtimes: Vec<Arc<dyn Runtime>>,
}

impl DummyProvider {
    fn new(name: &'static str, runtimes: Vec<Arc<dyn Runtime>>) -> Self {
        Self { name, runtimes }
    }
}

impl Provider for DummyProvider {
    fn name(&self) -> &str {
        self.name
    }

    fn description(&self) -> &str {
        "Dummy provider"
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        self.runtimes.clone()
    }
}

#[test]
fn provider_registry_supports_registered_provider() {
    let mut registry = ProviderRegistry::new();
    registry.register(Arc::new(DummyProvider::new(
        "tool",
        vec![Arc::new(DummyRuntime {
            name: "tool",
            aliases: &[],
        })],
    )));

    assert!(registry.supports("tool"));
    assert!(!registry.supports("other-tool"));
}

#[test]
fn provider_registry_supports_alias() {
    let mut registry = ProviderRegistry::new();
    registry.register(Arc::new(DummyProvider::new(
        "tool",
        vec![Arc::new(DummyRuntime {
            name: "tool",
            aliases: &["tool-alias", "t"],
        })],
    )));

    assert!(registry.supports("tool"));
    assert!(registry.supports("tool-alias"));
    assert!(registry.supports("t"));
}

#[test]
fn provider_registry_multiple_providers() {
    let mut registry = ProviderRegistry::new();
    registry.register(Arc::new(DummyProvider::new(
        "node",
        vec![
            Arc::new(DummyRuntime {
                name: "node",
                aliases: &["nodejs"],
            }),
            Arc::new(DummyRuntime {
                name: "npm",
                aliases: &[],
            }),
        ],
    )));
    registry.register(Arc::new(DummyProvider::new(
        "go",
        vec![Arc::new(DummyRuntime {
            name: "go",
            aliases: &["golang"],
        })],
    )));

    assert!(registry.supports("node"));
    assert!(registry.supports("nodejs"));
    assert!(registry.supports("npm"));
    assert!(registry.supports("go"));
    assert!(registry.supports("golang"));
}

#[test]
fn manifest_parse_and_use() {
    // Verify that ProviderManifest can be parsed and used
    let manifest = make_manifest("test-tool", "A test tool");
    assert_eq!(manifest.provider.name, "test-tool");
    assert_eq!(
        manifest.provider.description.as_deref(),
        Some("A test tool")
    );
    assert_eq!(manifest.runtimes.len(), 1);
    assert_eq!(manifest.runtimes[0].name, "test-tool");
}

#[test]
fn manifest_with_ecosystem() {
    let manifest = ProviderManifest {
        provider: ProviderMeta {
            name: "node".to_string(),
            description: Some("Node.js".to_string()),
            homepage: None,
            repository: None,
            ecosystem: Some(Ecosystem::NodeJs),
            platform_constraint: None,
            package_alias: None,
        },
        runtimes: vec![make_runtime_def("node")],
    };

    assert_eq!(manifest.provider.ecosystem, Some(Ecosystem::NodeJs));
}

#[test]
fn manifest_with_aliases() {
    let mut runtime_def = make_runtime_def("node");
    runtime_def.aliases = vec!["nodejs".to_string()];

    let manifest = ProviderManifest {
        provider: ProviderMeta {
            name: "node".to_string(),
            description: None,
            homepage: None,
            repository: None,
            ecosystem: None,
            platform_constraint: None,
            package_alias: None,
        },
        runtimes: vec![runtime_def],
    };

    assert_eq!(manifest.runtimes[0].aliases, vec!["nodejs"]);
}
