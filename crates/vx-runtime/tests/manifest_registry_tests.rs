use std::fs;
use std::path::Path;
use std::sync::Arc;

use async_trait::async_trait;
use tempfile::TempDir;
use vx_manifest::{Ecosystem, ProviderManifest, ProviderMeta, RuntimeDef};
use vx_runtime::{ManifestRegistry, Provider, Runtime, RuntimeContext, VersionInfo};

/// Create a minimal RuntimeDef for testing
fn make_runtime_def(name: &str) -> RuntimeDef {
    RuntimeDef {
        name: name.to_string(),
        description: None,
        executable: name.to_string(),
        aliases: vec![],
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
        },
        runtimes: vec![make_runtime_def(name)],
    }
}

fn create_test_manifest(dir: &Path, name: &str) {
    let provider_dir = dir.join(name);
    fs::create_dir_all(&provider_dir).unwrap();

    let manifest = format!(
        r#"
[provider]
name = "{name}"

[[runtimes]]
name = "{name}"
executable = "{name}"
"#
    );

    fs::write(provider_dir.join("provider.toml"), manifest).unwrap();
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
fn later_manifest_overrides_by_name() {
    let mut registry = ManifestRegistry::new();

    let base = make_manifest("tool", "base description");
    let overlay = make_manifest("tool", "override description");

    registry.load_from_manifests(vec![base, overlay]);

    let manifest = registry
        .get_manifest("tool")
        .expect("manifest should exist after load");
    assert_eq!(
        manifest.provider.description.as_deref(),
        Some("override description")
    );
}

#[test]
fn directory_override_replaces_embedded() {
    let mut registry = ManifestRegistry::new();

    // Load embedded manifest first via load_from_manifests
    let embedded = ProviderManifest {
        provider: ProviderMeta {
            name: "tool".to_string(),
            description: Some("embedded".to_string()),
            homepage: None,
            repository: None,
            ecosystem: None,
            platform_constraint: None,
        },
        runtimes: vec![make_runtime_def("tool")],
    };
    registry.load_from_manifests(vec![embedded]);

    // Project-level override
    let temp = TempDir::new().expect("temp dir");
    let provider_dir = temp.path().join("tool");
    fs::create_dir_all(&provider_dir).expect("create provider dir");
    let override_manifest = r#"
[provider]
name = "tool"
description = "project override"

[[runtimes]]
name = "tool"
executable = "tool"
"#;
    fs::write(provider_dir.join("provider.toml"), override_manifest).expect("write override");

    registry
        .load_from_directory(temp.path())
        .expect("load override dir");

    let manifest = registry
        .get_manifest("tool")
        .expect("manifest should exist after override");
    assert_eq!(
        manifest.provider.description.as_deref(),
        Some("project override")
    );
}

#[test]
fn manifest_registry_loads_directory() {
    let temp_dir = TempDir::new().unwrap();
    create_test_manifest(temp_dir.path(), "test-provider");

    let mut registry = ManifestRegistry::new();
    let count = registry.load_from_directory(temp_dir.path()).unwrap();

    assert_eq!(count, 1);
    assert!(registry.get_manifest("test-provider").is_some());
}

#[test]
fn runtime_metadata_resolves_aliases() {
    let mut registry = ManifestRegistry::new();

    let mut runtime_def = make_runtime_def("test-runtime");
    runtime_def.description = Some("A test runtime".to_string());
    runtime_def.executable = "test-bin".to_string();
    runtime_def.aliases = vec!["tr".to_string(), "test".to_string()];

    let manifest = ProviderManifest {
        provider: ProviderMeta {
            name: "test".to_string(),
            description: Some("desc".to_string()),
            homepage: None,
            repository: None,
            ecosystem: Some(Ecosystem::NodeJs),
            platform_constraint: None,
        },
        runtimes: vec![runtime_def],
    };

    registry.load_from_manifests(vec![manifest]);

    let metadata = registry
        .get_runtime_metadata("tr")
        .expect("metadata should resolve by alias");

    assert_eq!(metadata.name, "test-runtime");
    assert_eq!(metadata.executable, "test-bin");
    assert_eq!(metadata.aliases, vec!["tr", "test"]);
    assert_eq!(metadata.provider_name, "test");
    assert_eq!(metadata.ecosystem, Some(Ecosystem::NodeJs));
}

#[test]
fn build_registry_uses_registered_factories() {
    let mut registry = ManifestRegistry::new();

    registry.register_factory("tool", || {
        Arc::new(DummyProvider::new(
            "tool",
            vec![Arc::new(DummyRuntime {
                name: "tool",
                aliases: &["tool-alias"],
            })],
        )) as Arc<dyn Provider>
    });

    registry.load_from_manifests(vec![make_manifest("tool", "desc")]);

    let provider_registry = registry.build_registry();
    assert!(provider_registry.supports("tool"));
    assert!(provider_registry.supports("tool-alias"));
}
