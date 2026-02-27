//! ollama provider tests

use rstest::rstest;
use vx_runtime::Runtime;

fn create_provider() -> std::sync::Arc<dyn vx_runtime::Provider> {
    let meta = vx_starlark::StarMetadata::parse(vx_provider_ollama::PROVIDER_STAR);
    let name: &'static str = Box::leak(
        meta.name
            .unwrap_or_else(|| "unknown".to_string())
            .into_boxed_str(),
    );
    vx_starlark::create_provider(name, vx_provider_ollama::PROVIDER_STAR)
}

#[test]
fn test_provider_name() {
    let provider = create_provider();
    assert_eq!(provider.name(), "ollama");
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
    assert!(names.contains(&"ollama"));
}

#[rstest]
#[case("ollama", true)]
#[case("node", false)]
fn test_provider_supports(#[case] name: &str, #[case] expected: bool) {
    let provider = create_provider();
    assert_eq!(provider.supports(name), expected);
}

#[test]
fn test_provider_get_runtime() {
    let provider = create_provider();
    assert!(provider.get_runtime("ollama").is_some());
    assert!(provider.get_runtime("unknown").is_none());
}

#[test]
fn test_star_metadata() {
    let meta = vx_starlark::StarMetadata::parse(vx_provider_ollama::PROVIDER_STAR);
    assert!(meta.name.is_some());
    assert!(!meta.runtimes.is_empty());
}
