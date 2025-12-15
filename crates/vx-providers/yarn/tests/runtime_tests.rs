//! Yarn runtime tests

use rstest::rstest;
use vx_provider_yarn::{YarnProvider, YarnRuntime};
use vx_runtime::{Ecosystem, Provider, Runtime};

#[rstest]
fn test_yarn_runtime_name() {
    let runtime = YarnRuntime::new();
    assert_eq!(runtime.name(), "yarn");
}

#[rstest]
fn test_yarn_runtime_ecosystem() {
    let runtime = YarnRuntime::new();
    assert_eq!(runtime.ecosystem(), Ecosystem::NodeJs);
}

#[rstest]
fn test_yarn_runtime_description() {
    let runtime = YarnRuntime::new();
    assert_eq!(
        runtime.description(),
        "Fast, reliable, and secure dependency management"
    );
}

#[rstest]
fn test_yarn_provider_name() {
    let provider = YarnProvider::new();
    assert_eq!(provider.name(), "yarn");
}

#[rstest]
fn test_yarn_provider_runtimes() {
    let provider = YarnProvider::new();
    let runtimes = provider.runtimes();
    assert_eq!(runtimes.len(), 1);
    assert_eq!(runtimes[0].name(), "yarn");
}

#[rstest]
fn test_yarn_provider_supports() {
    let provider = YarnProvider::new();
    assert!(provider.supports("yarn"));
    assert!(!provider.supports("npm"));
}

#[rstest]
fn test_yarn_provider_get_runtime() {
    let provider = YarnProvider::new();

    let yarn = provider.get_runtime("yarn");
    assert!(yarn.is_some());
    assert_eq!(yarn.unwrap().name(), "yarn");

    let unknown = provider.get_runtime("unknown");
    assert!(unknown.is_none());
}
