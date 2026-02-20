//! kubectl provider tests

use rstest::rstest;
use vx_provider_kubectl::create_provider;
use vx_runtime::{Provider, Runtime};

#[test]
fn test_provider_name() {
    let provider = create_provider();
    assert_eq!(provider.name(), "kubectl");
}

#[test]
fn test_provider_description() {
    let provider = create_provider();
    assert!(!provider.description().is_empty());
}

#[test]
fn test_provider_runtimes() {
    let provider = create_provider();
    let runtimes = provider.runtimes();
    assert!(!runtimes.is_empty());
    let names: Vec<&str> = runtimes
        .iter()
        .map(|r: &std::sync::Arc<dyn Runtime>| r.name())
        .collect();
    assert!(names.contains(&"kubectl"));
}

#[rstest]
#[case("kubectl", true)]
#[case("kube", true)]
#[case("k", true)]
#[case("node", false)]
fn test_provider_supports(#[case] name: &str, #[case] expected: bool) {
    let provider = create_provider();
    assert_eq!(provider.supports(name), expected);
}

#[test]
fn test_provider_get_runtime() {
    let provider = create_provider();
    assert!(provider.get_runtime("kubectl").is_some());
    assert!(provider.get_runtime("kube").is_some());
    assert!(provider.get_runtime("k").is_some());
    assert!(provider.get_runtime("unknown").is_none());
}

#[test]
fn test_star_metadata() {
    let meta = vx_provider_kubectl::star_metadata();
    assert!(meta.name.is_some());
    assert!(!meta.runtimes.is_empty());
}
