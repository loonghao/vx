//! cmake provider tests

use rstest::rstest;
use vx_provider_cmake::create_provider;
use vx_runtime::{Provider, Runtime};

#[test]
fn test_provider_name() {
    let provider = create_provider();
    assert_eq!(provider.name(), "cmake");
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
    assert!(names.contains(&"cmake"));
    assert!(names.contains(&"ctest"));
    assert!(names.contains(&"cpack"));
}

#[rstest]
#[case("cmake", true)]
#[case("ctest", true)]
#[case("cpack", true)]
#[case("node", false)]
fn test_provider_supports(#[case] name: &str, #[case] expected: bool) {
    let provider = create_provider();
    assert_eq!(provider.supports(name), expected);
}

#[test]
fn test_provider_get_runtime() {
    let provider = create_provider();
    assert!(provider.get_runtime("cmake").is_some());
    assert!(provider.get_runtime("ctest").is_some());
    assert!(provider.get_runtime("cpack").is_some());
    assert!(provider.get_runtime("unknown").is_none());
}

#[test]
fn test_star_metadata() {
    let meta = vx_provider_cmake::star_metadata();
    assert!(meta.name.is_some());
    assert!(!meta.runtimes.is_empty());
}
