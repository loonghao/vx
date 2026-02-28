//! Tests for ProviderManifest struct construction and usage.
//!
//! Registry behavior tests (supports, aliases, multiple providers) are in registry_tests.rs.

use vx_manifest::{Ecosystem, ProviderManifest, ProviderMeta, RuntimeDef};

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
