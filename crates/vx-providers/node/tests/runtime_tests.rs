//! Node.js provider tests

use rstest::rstest;
use vx_provider_node::create_provider;
use vx_runtime::{Ecosystem, Provider, Runtime};

#[test]
fn test_node_provider_creation() {
    let provider = create_provider();
    assert_eq!(provider.name(), "node");
    assert!(!provider.description().is_empty());
}

#[test]
fn test_node_provider_runtimes() {
    let provider = create_provider();
    let runtimes = provider.runtimes();

    assert!(!runtimes.is_empty());

    let names: Vec<&str> = runtimes
        .iter()
        .map(|r: &std::sync::Arc<dyn Runtime>| r.name())
        .collect();
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
    let provider = create_provider();
    assert_eq!(provider.supports(name), expected);
}

#[test]
fn test_node_runtime_basic() {
    let provider = create_provider();
    let runtimes = provider.runtimes();
    let node = runtimes.iter().find(|r| r.name() == "node");
    assert!(node.is_some());
    let node = node.unwrap();
    assert_eq!(node.name(), "node");
    assert!(!node.description().is_empty());
    assert_eq!(node.ecosystem(), Ecosystem::NodeJs);
}

#[test]
fn test_npm_runtime_basic() {
    let provider = create_provider();
    let runtimes = provider.runtimes();
    let npm = runtimes.iter().find(|r| r.name() == "npm");
    assert!(npm.is_some());
    let npm = npm.unwrap();
    assert_eq!(npm.name(), "npm");
    assert!(!npm.description().is_empty());
}

#[test]
fn test_npx_runtime_basic() {
    let provider = create_provider();
    let runtimes = provider.runtimes();
    let npx = runtimes.iter().find(|r| r.name() == "npx");
    assert!(npx.is_some());
    let npx = npx.unwrap();
    assert_eq!(npx.name(), "npx");
    assert!(!npx.description().is_empty());
}

#[test]
fn test_node_provider_get_runtime() {
    let provider = create_provider();

    let node = provider.get_runtime("node");
    assert!(node.is_some());
    assert_eq!(node.unwrap().name(), "node");

    let nodejs = provider.get_runtime("nodejs");
    assert!(nodejs.is_some());

    let npm = provider.get_runtime("npm");
    assert!(npm.is_some());
    assert_eq!(npm.unwrap().name(), "npm");

    let unknown = provider.get_runtime("unknown");
    assert!(unknown.is_none());
}

#[test]
fn test_star_metadata() {
    let meta = vx_provider_node::star_metadata();
    assert!(meta.name.is_some());
    assert!(!meta.runtimes.is_empty());
}
