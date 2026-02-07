//! Tests for the manifest sub-module (RFC 0029 Phase 2: ManifestRegistry split)

use std::fs;
use std::sync::Arc;

use async_trait::async_trait;
use tempfile::TempDir;
use vx_manifest::{PlatformConstraint, ProviderManifest};
use vx_runtime::manifest::builder::ProviderBuilder;
use vx_runtime::manifest::index::ManifestIndex;
use vx_runtime::manifest::loader::ManifestStore;
use vx_runtime::{Provider, Runtime, RuntimeContext, VersionInfo};

// ========== Test helpers ==========

struct DummyRuntime {
    name: &'static str,
}

#[async_trait]
impl Runtime for DummyRuntime {
    fn name(&self) -> &str {
        self.name
    }

    async fn fetch_versions(&self, _ctx: &RuntimeContext) -> anyhow::Result<Vec<VersionInfo>> {
        Ok(vec![VersionInfo::new("1.0.0")])
    }
}

struct DummyProvider {
    name: &'static str,
}

impl Provider for DummyProvider {
    fn name(&self) -> &str {
        self.name
    }

    fn description(&self) -> &str {
        "Dummy provider for testing"
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(DummyRuntime { name: self.name })]
    }
}

fn parse_manifest(toml_content: &str) -> ProviderManifest {
    ProviderManifest::parse(toml_content).expect("failed to parse manifest TOML")
}

fn write_manifest(dir: &std::path::Path, name: &str, content: &str) {
    let provider_dir = dir.join(name);
    fs::create_dir_all(&provider_dir).unwrap();
    fs::write(provider_dir.join("provider.toml"), content).unwrap();
}

// ========== ManifestStore tests ==========

#[test]
fn store_load_from_directory() {
    let temp = TempDir::new().unwrap();
    write_manifest(
        temp.path(),
        "test-tool",
        r#"
[provider]
name = "test-tool"

[[runtimes]]
name = "test-tool"
executable = "test-tool"
"#,
    );

    let mut store = ManifestStore::new();
    let count = store.load_from_directory(temp.path()).unwrap();

    assert_eq!(count, 1);
    assert!(!store.is_empty());
    assert_eq!(store.len(), 1);
    assert!(store.get("test-tool").is_some());
}

#[test]
fn store_load_from_manifests() {
    let manifest = parse_manifest(
        r#"
[provider]
name = "my-tool"

[[runtimes]]
name = "my-tool"
executable = "my-tool"
"#,
    );

    let mut store = ManifestStore::new();
    store.load_from_manifests(vec![manifest]);

    assert_eq!(store.len(), 1);
    assert_eq!(store.names(), vec!["my-tool"]);
}

#[test]
fn store_find_runtime_by_name_and_alias() {
    let manifest = parse_manifest(
        r#"
[provider]
name = "test"

[[runtimes]]
name = "test-runtime"
executable = "test-bin"
aliases = ["tr", "test-rt"]
"#,
    );

    let mut store = ManifestStore::new();
    store.load_from_manifests(vec![manifest]);

    assert!(store.find_runtime("test-runtime").is_some());
    assert!(store.find_runtime("tr").is_some());
    assert!(store.find_runtime("test-rt").is_some());
    assert!(store.find_runtime("nonexistent").is_none());
}

// ========== ManifestIndex tests ==========

#[test]
fn index_basic_lookup() {
    let manifest = parse_manifest(
        r#"
[provider]
name = "node"
ecosystem = "nodejs"

[[runtimes]]
name = "node"
executable = "node"
aliases = ["nodejs"]
"#,
    );

    let index = ManifestIndex::from_manifests(&[manifest]);

    assert!(index.has_runtime("node"));
    assert!(index.has_runtime("nodejs"));
    assert!(!index.has_runtime("deno"));

    let meta = index.get_runtime("node").unwrap();
    assert_eq!(meta.name, "node");
    assert_eq!(meta.executable, "node");
    assert_eq!(meta.provider_name, "node");
    assert_eq!(meta.ecosystem, Some(vx_manifest::Ecosystem::NodeJs));
}

#[test]
fn index_alias_resolution() {
    let manifest = parse_manifest(
        r#"
[provider]
name = "test"

[[runtimes]]
name = "my-runtime"
executable = "myrt"
aliases = ["mr", "myrt"]
"#,
    );

    let index = ManifestIndex::from_manifests(&[manifest]);

    assert_eq!(index.resolve_alias("mr"), "my-runtime");
    assert_eq!(index.resolve_alias("myrt"), "my-runtime");
    assert_eq!(index.resolve_alias("unknown"), "unknown"); // passthrough

    // Lookup via alias returns the same metadata
    let by_name = index.get_runtime("my-runtime").unwrap();
    let by_alias = index.get_runtime("mr").unwrap();
    assert_eq!(by_name.name, by_alias.name);
}

#[test]
fn index_platform_constraint_intersection() {
    let manifest = parse_manifest(
        r#"
[provider]
name = "tool"

[provider.platforms]
os = ["windows", "linux"]

[[runtimes]]
name = "win-tool"
executable = "win-tool"
platform_constraint = { os = ["windows"] }

[[runtimes]]
name = "any-tool"
executable = "any-tool"
"#,
    );

    let index = ManifestIndex::from_manifests(&[manifest]);

    // win-tool: provider(windows,linux) ∩ runtime(windows) = windows
    let win_constraint = index.get_platform_constraint("win-tool").unwrap();
    assert_eq!(win_constraint.os, vec![vx_manifest::Os::Windows]);

    // any-tool: provider(windows,linux) ∩ runtime(none) = windows,linux
    let any_constraint = index.get_platform_constraint("any-tool").unwrap();
    assert_eq!(any_constraint.os.len(), 2);
}

#[test]
fn index_no_platform_constraint() {
    let manifest = parse_manifest(
        r#"
[provider]
name = "tool"

[[runtimes]]
name = "cross-platform"
executable = "tool"
"#,
    );

    let index = ManifestIndex::from_manifests(&[manifest]);
    assert!(index.get_platform_constraint("cross-platform").is_none());
}

#[test]
fn index_multiple_providers() {
    let m1 = parse_manifest(
        r#"
[provider]
name = "node"

[[runtimes]]
name = "node"
executable = "node"
aliases = ["nodejs"]

[[runtimes]]
name = "npm"
executable = "npm"
"#,
    );

    let m2 = parse_manifest(
        r#"
[provider]
name = "go"

[[runtimes]]
name = "go"
executable = "go"
aliases = ["golang"]
"#,
    );

    let index = ManifestIndex::from_manifests(&[m1, m2]);

    assert_eq!(index.runtime_count(), 3);
    assert_eq!(index.provider_count(), 2);
    assert!(index.has_runtime("node"));
    assert!(index.has_runtime("npm"));
    assert!(index.has_runtime("golang"));
}

#[test]
fn index_get_supported_runtimes() {
    let manifest = parse_manifest(
        r#"
[provider]
name = "tool"

[[runtimes]]
name = "cross-tool"
executable = "cross-tool"

[[runtimes]]
name = "alien-tool"
executable = "alien-tool"
# This won't match any real platform
platform_constraint = { os = [], arch = [] }
"#,
    );

    let index = ManifestIndex::from_manifests(&[manifest]);
    let supported = index.get_supported_runtimes();
    let all = index.get_all_runtimes();

    // cross-tool has no constraint (supported everywhere)
    // alien-tool has empty constraint (also treated as "all supported")
    assert_eq!(supported.len(), all.len());
}

#[test]
fn index_provider_metadata() {
    let manifest = parse_manifest(
        r#"
[provider]
name = "my-provider"
description = "A test provider"
ecosystem = "python"

[[runtimes]]
name = "tool"
executable = "tool"
"#,
    );

    let index = ManifestIndex::from_manifests(&[manifest]);
    let provider = index.get_provider("my-provider").unwrap();
    assert_eq!(provider.description.as_deref(), Some("A test provider"));
    assert_eq!(provider.ecosystem, Some(vx_manifest::Ecosystem::Python));
}

// ========== ProviderBuilder tests ==========

#[test]
fn builder_build_with_factory() {
    let manifest = parse_manifest(
        r#"
[provider]
name = "test-tool"

[[runtimes]]
name = "test-tool"
executable = "test-tool"
"#,
    );

    let mut builder = ProviderBuilder::new();
    builder.register_factory("test-tool", || {
        Arc::new(DummyProvider { name: "test-tool" }) as Arc<dyn Provider>
    });

    let result = builder.build(&[manifest]);

    assert!(result.errors.is_empty());
    assert!(result.warnings.is_empty());
    assert!(result.registry.supports("test-tool"));
}

#[test]
fn builder_build_missing_factory() {
    let manifest = parse_manifest(
        r#"
[provider]
name = "no-factory"

[[runtimes]]
name = "no-factory"
executable = "no-factory"
"#,
    );

    let builder = ProviderBuilder::new();
    let result = builder.build(&[manifest]);

    assert_eq!(result.errors.len(), 1);
    assert_eq!(result.errors[0].provider, "no-factory");
    assert!(result.errors[0].reason.contains("No factory"));
}

#[test]
fn builder_build_mixed_factories() {
    let m1 = parse_manifest(
        r#"
[provider]
name = "has-factory"

[[runtimes]]
name = "has-factory"
executable = "has-factory"
"#,
    );
    let m2 = parse_manifest(
        r#"
[provider]
name = "missing"

[[runtimes]]
name = "missing"
executable = "missing"
"#,
    );

    let mut builder = ProviderBuilder::new();
    builder.register_factory("has-factory", || {
        Arc::new(DummyProvider {
            name: "has-factory",
        }) as Arc<dyn Provider>
    });

    let result = builder.build(&[m1, m2]);

    assert!(result.registry.supports("has-factory"));
    assert!(!result.registry.supports("missing"));
    assert_eq!(result.errors.len(), 1);
    assert_eq!(result.errors[0].provider, "missing");
}

#[test]
fn builder_build_from_factories_only() {
    let mut builder = ProviderBuilder::new();
    builder.register_factory("tool-a", || {
        Arc::new(DummyProvider { name: "tool-a" }) as Arc<dyn Provider>
    });
    builder.register_factory("tool-b", || {
        Arc::new(DummyProvider { name: "tool-b" }) as Arc<dyn Provider>
    });

    let registry = builder.build_from_factories();
    assert!(registry.supports("tool-a"));
    assert!(registry.supports("tool-b"));
}

#[test]
fn builder_factory_names() {
    let mut builder = ProviderBuilder::new();
    builder.register_factory("alpha", || {
        Arc::new(DummyProvider { name: "alpha" }) as Arc<dyn Provider>
    });
    builder.register_factory("beta", || {
        Arc::new(DummyProvider { name: "beta" }) as Arc<dyn Provider>
    });

    let mut names = builder.factory_names();
    names.sort();
    assert_eq!(names, vec!["alpha", "beta"]);
}

// ========== PlatformConstraint::intersect tests ==========

#[test]
fn platform_intersect_both_empty() {
    let a = PlatformConstraint::new();
    let b = PlatformConstraint::new();
    let result = a.intersect(&b);
    assert!(result.is_empty());
}

#[test]
fn platform_intersect_one_empty() {
    let a = PlatformConstraint::windows_only();
    let b = PlatformConstraint::new();
    let result = a.intersect(&b);
    assert_eq!(result.os, vec![vx_manifest::Os::Windows]);
}

#[test]
fn platform_intersect_overlapping_os() {
    use vx_manifest::Os;
    let a = PlatformConstraint {
        os: vec![Os::Windows, Os::Linux],
        ..Default::default()
    };
    let b = PlatformConstraint {
        os: vec![Os::Linux, Os::MacOS],
        ..Default::default()
    };
    let result = a.intersect(&b);
    assert_eq!(result.os, vec![Os::Linux]);
}

#[test]
fn platform_intersect_disjoint_os() {
    let a = PlatformConstraint::windows_only();
    let b = PlatformConstraint::linux_only();
    let result = a.intersect(&b);
    assert!(result.os.is_empty());
}

#[test]
fn platform_intersect_excludes_union() {
    use vx_manifest::{Os, PlatformExclusion};
    let a = PlatformConstraint {
        exclude: vec![PlatformExclusion {
            os: Some(Os::Windows),
            arch: None,
        }],
        ..Default::default()
    };
    let b = PlatformConstraint {
        exclude: vec![PlatformExclusion {
            os: Some(Os::Linux),
            arch: None,
        }],
        ..Default::default()
    };
    let result = a.intersect(&b);
    assert_eq!(result.exclude.len(), 2);
}
