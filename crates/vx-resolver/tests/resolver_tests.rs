//! Tests for resolver

use rstest::rstest;
use std::path::PathBuf;
use vx_resolver::{Resolver, ResolverConfig, RuntimeStatus};

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
    let resolver = Resolver::new(config);
    assert!(resolver.is_ok());
}

#[rstest]
fn test_known_runtimes() {
    let config = ResolverConfig::default();
    let resolver = Resolver::new(config).unwrap();

    assert!(resolver.is_known_runtime("node"));
    assert!(resolver.is_known_runtime("npm"));
    assert!(resolver.is_known_runtime("uv"));
    assert!(resolver.is_known_runtime("cargo"));
}

#[rstest]
fn test_unknown_runtime() {
    let config = ResolverConfig::default();
    let resolver = Resolver::new(config).unwrap();

    assert!(!resolver.is_known_runtime("unknown-runtime"));
}

#[rstest]
fn test_get_spec() {
    let config = ResolverConfig::default();
    let resolver = Resolver::new(config).unwrap();

    let node_spec = resolver.get_spec("node");
    assert!(node_spec.is_some());
    assert_eq!(node_spec.unwrap().name, "node");

    let unknown_spec = resolver.get_spec("unknown");
    assert!(unknown_spec.is_none());
}
