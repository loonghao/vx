//! Tests for resolver

use rstest::rstest;
use std::path::PathBuf;
use vx_manifest::ProviderManifest;
use vx_resolver::{Resolver, ResolverConfig, RuntimeMap, RuntimeStatus};

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

[[runtimes]]
name = "uv"
description = "UV"
executable = "uv"

[[runtimes]]
name = "cargo"
description = "Cargo"
executable = "cargo"
"#;
    let manifest = ProviderManifest::parse(toml).expect("Failed to parse manifest");
    RuntimeMap::from_manifests(&[manifest])
}

#[rstest]
fn test_runtime_status_is_available() {
    assert!(RuntimeStatus::VxManaged {
        version: "1.0.0".into(),
        path: PathBuf::from("/usr/bin/node")
    }
    .is_available());

    assert!(RuntimeStatus::SystemAvailable {
        path: PathBuf::from("/usr/bin/node")
    }
    .is_available());

    assert!(!RuntimeStatus::NotInstalled.is_available());
    assert!(!RuntimeStatus::Unknown.is_available());
}

#[rstest]
fn test_resolver_creation() {
    let config = ResolverConfig::default();
    let runtime_map = create_test_runtime_map();
    let resolver = Resolver::new(config, runtime_map);
    assert!(resolver.is_ok());
}

#[rstest]
fn test_known_runtimes() {
    let config = ResolverConfig::default();
    let runtime_map = create_test_runtime_map();
    let resolver = Resolver::new(config, runtime_map).unwrap();

    assert!(resolver.is_known_runtime("node"));
    assert!(resolver.is_known_runtime("npm"));
    assert!(resolver.is_known_runtime("uv"));
    assert!(resolver.is_known_runtime("cargo"));
}

#[rstest]
fn test_unknown_runtime() {
    let config = ResolverConfig::default();
    let runtime_map = create_test_runtime_map();
    let resolver = Resolver::new(config, runtime_map).unwrap();

    assert!(!resolver.is_known_runtime("unknown-runtime"));
}

#[rstest]
fn test_get_spec() {
    let config = ResolverConfig::default();
    let runtime_map = create_test_runtime_map();
    let resolver = Resolver::new(config, runtime_map).unwrap();

    let node_spec = resolver.get_spec("node");
    assert!(node_spec.is_some());
    assert_eq!(node_spec.unwrap().name, "node");

    let unknown_spec = resolver.get_spec("unknown");
    assert!(unknown_spec.is_none());
}
