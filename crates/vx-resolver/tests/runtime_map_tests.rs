//! Tests for runtime map

use rstest::rstest;
use vx_resolver::{Ecosystem, RuntimeMap};

#[rstest]
fn test_runtime_map_creation() {
    let map = RuntimeMap::new();

    assert!(map.contains("node"));
    assert!(map.contains("nodejs")); // alias
    assert!(map.contains("npm"));
    assert!(map.contains("uv"));
    assert!(map.contains("cargo"));
}

#[rstest]
fn test_alias_resolution() {
    let map = RuntimeMap::new();

    assert_eq!(map.resolve_name("nodejs"), Some("node"));
    assert_eq!(map.resolve_name("node"), Some("node"));
    assert_eq!(map.resolve_name("pip3"), Some("pip"));
}

#[rstest]
fn test_dependency_lookup() {
    let map = RuntimeMap::new();

    let npm = map.get("npm").unwrap();
    assert_eq!(npm.required_dependencies().len(), 1);
    assert_eq!(npm.required_dependencies()[0].runtime_name, "node");
}

#[rstest]
fn test_ecosystem_filtering() {
    let map = RuntimeMap::new();

    let node_runtimes = map.by_ecosystem(Ecosystem::Node);
    assert!(node_runtimes.iter().any(|t| t.name == "node"));
    assert!(node_runtimes.iter().any(|t| t.name == "npm"));
    assert!(node_runtimes.iter().any(|t| t.name == "yarn"));
}

#[rstest]
fn test_install_order() {
    let map = RuntimeMap::new();

    // npm depends on node, so node should come first
    let order = map.get_install_order("npm");
    let node_pos = order.iter().position(|&t| t == "node");
    let npm_pos = order.iter().position(|&t| t == "npm");

    assert!(node_pos.is_some());
    assert!(npm_pos.is_some());
    assert!(node_pos.unwrap() < npm_pos.unwrap());
}

#[rstest]
fn test_uvx_command_prefix() {
    let map = RuntimeMap::new();

    let uvx = map.get("uvx").unwrap();
    assert_eq!(uvx.get_executable(), "uv");
    assert_eq!(uvx.command_prefix, vec!["tool", "run"]);
}

#[rstest]
fn test_standalone_runtimes() {
    let map = RuntimeMap::new();

    // uv has no dependencies
    let uv = map.get("uv").unwrap();
    assert!(uv.required_dependencies().is_empty());

    // bun has no dependencies
    let bun = map.get("bun").unwrap();
    assert!(bun.required_dependencies().is_empty());
}

#[rstest]
fn test_empty_runtime_map() {
    let map = RuntimeMap::empty();
    assert!(!map.contains("node"));
    assert!(map.runtime_names().is_empty());
}
