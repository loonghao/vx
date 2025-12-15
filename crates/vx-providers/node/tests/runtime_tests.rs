//! Node.js runtime tests

use rstest::rstest;
use vx_provider_node::{NodeProvider, NodeRuntime, NpmRuntime, NpxRuntime};
use vx_runtime::{Ecosystem, Provider, Runtime};

#[test]
fn test_node_runtime_creation() {
    let runtime = NodeRuntime::new();
    assert_eq!(runtime.name(), "node");
    assert!(!runtime.description().is_empty());
    assert!(runtime.aliases().contains(&"nodejs"));
    assert_eq!(runtime.ecosystem(), Ecosystem::NodeJs);
}

#[test]
fn test_npm_runtime_creation() {
    let runtime = NpmRuntime::new();
    assert_eq!(runtime.name(), "npm");
    assert!(!runtime.description().is_empty());
    assert_eq!(runtime.ecosystem(), Ecosystem::NodeJs);
}

#[test]
fn test_npx_runtime_creation() {
    let runtime = NpxRuntime::new();
    assert_eq!(runtime.name(), "npx");
    assert!(!runtime.description().is_empty());
    assert_eq!(runtime.ecosystem(), Ecosystem::NodeJs);
}

#[test]
fn test_node_runtime_metadata() {
    let runtime = NodeRuntime::new();
    let metadata = runtime.metadata();

    assert!(metadata.contains_key("homepage"));
    assert!(metadata.contains_key("ecosystem"));
    assert_eq!(metadata.get("ecosystem"), Some(&"javascript".to_string()));
}

#[test]
fn test_node_provider_creation() {
    let provider = NodeProvider::new();
    assert_eq!(provider.name(), "node");
    assert!(!provider.description().is_empty());
}

#[test]
fn test_node_provider_runtimes() {
    let provider = NodeProvider::new();
    let runtimes = provider.runtimes();

    assert_eq!(runtimes.len(), 3);

    let names: Vec<&str> = runtimes.iter().map(|r| r.name()).collect();
    assert!(names.contains(&"node"));
    assert!(names.contains(&"npm"));
    assert!(names.contains(&"npx"));
}

#[rstest]
#[case("node", true)]
#[case("nodejs", true)]
#[case("npm", true)]
#[case("npx", true)]
#[case("go", false)]
#[case("python", false)]
fn test_node_provider_supports(#[case] name: &str, #[case] expected: bool) {
    let provider = NodeProvider::new();
    assert_eq!(provider.supports(name), expected);
}

#[test]
fn test_node_provider_get_runtime() {
    let provider = NodeProvider::new();

    let node = provider.get_runtime("node");
    assert!(node.is_some());
    assert_eq!(node.unwrap().name(), "node");

    let nodejs = provider.get_runtime("nodejs");
    assert!(nodejs.is_some());
    assert_eq!(nodejs.unwrap().name(), "node");

    let npm = provider.get_runtime("npm");
    assert!(npm.is_some());
    assert_eq!(npm.unwrap().name(), "npm");

    let unknown = provider.get_runtime("unknown");
    assert!(unknown.is_none());
}
