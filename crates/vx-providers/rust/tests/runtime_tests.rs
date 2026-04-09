//! rust provider tests

use rstest::rstest;
use vx_runtime::Runtime;

#[test]
fn test_provider_name() {
    let provider = create_provider();
    assert_eq!(provider.name(), "rust");
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
    assert!(names.contains(&"cargo"));
    assert!(names.contains(&"rustc"));
    // Primary runtime is now "rust" (with rustup as alias)
    assert!(names.contains(&"rust"));
}

#[rstest]
#[case("cargo", true)]
#[case("rustc", true)]
#[case("rustup", true)]  // rustup is an alias for "rust"
#[case("rust", true)]
#[case("node", false)]
fn test_provider_supports(#[case] name: &str, #[case] expected: bool) {
    let provider = create_provider();
    assert_eq!(provider.supports(name), expected);
}

#[test]
fn test_provider_get_runtime() {
    let provider = create_provider();
    assert!(provider.get_runtime("cargo").is_some());
    assert!(provider.get_runtime("rustc").is_some());
    // "rust" is the primary runtime name; "rustup" is an alias
    assert!(provider.get_runtime("rust").is_some());
    assert!(provider.get_runtime("unknown").is_none());
}

#[test]
fn test_star_metadata() {
    let meta = vx_starlark::StarMetadata::parse(vx_provider_rust::PROVIDER_STAR);
    eprintln!("meta.name = {:?}", meta.name);
    eprintln!("meta.runtimes.len() = {}", meta.runtimes.len());
    for rt in &meta.runtimes {
        eprintln!("  runtime: name={:?} bundled_with={:?}", rt.name, rt.bundled_with);
    }
    assert!(meta.name.is_some());
    assert!(!meta.runtimes.is_empty(), "runtimes should not be empty");
}
fn create_provider() -> std::sync::Arc<dyn vx_runtime::Provider> {
    let meta = vx_starlark::StarMetadata::parse(vx_provider_rust::PROVIDER_STAR);
    let name = meta.name.unwrap_or_else(|| "unknown".to_string());
    vx_starlark::create_provider(name, vx_provider_rust::PROVIDER_STAR)
}
