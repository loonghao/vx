//! java provider tests

use rstest::rstest;
use vx_provider_java::create_provider;
use vx_runtime::{Provider, Runtime};

#[test]
fn test_provider_name() {
    let provider = create_provider();
    assert_eq!(provider.name(), "java");
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
    assert!(names.contains(&"java"));
    assert!(names.contains(&"javac"));
    assert!(names.contains(&"jar"));
}

#[rstest]
#[case("java", true)]
#[case("jdk", true)]
#[case("temurin", true)]
#[case("javac", true)]
#[case("jar", true)]
#[case("node", false)]
fn test_provider_supports(#[case] name: &str, #[case] expected: bool) {
    let provider = create_provider();
    assert_eq!(provider.supports(name), expected);
}

#[test]
fn test_provider_get_runtime() {
    let provider = create_provider();
    assert!(provider.get_runtime("java").is_some());
    assert!(provider.get_runtime("jdk").is_some());
    assert!(provider.get_runtime("javac").is_some());
    assert!(provider.get_runtime("unknown").is_none());
}

#[test]
fn test_star_metadata() {
    let meta = vx_provider_java::star_metadata();
    assert!(meta.name.is_some());
    assert!(!meta.runtimes.is_empty());
}
