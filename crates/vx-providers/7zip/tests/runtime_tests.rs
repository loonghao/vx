//! 7zip provider tests

use rstest::rstest;
use vx_runtime::Runtime;

#[test]
fn test_provider_name() {
    let provider = create_provider();
    assert_eq!(provider.name(), "7zip");
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
    assert!(names.contains(&"7zip"));
}

#[rstest]
#[case("7zip", true)]
#[case("7z", true)]
#[case("7za", true)]
#[case("node", false)]
fn test_provider_supports(#[case] name: &str, #[case] expected: bool) {
    let provider = create_provider();
    assert_eq!(provider.supports(name), expected);
}

#[test]
fn test_provider_get_runtime() {
    let provider = create_provider();
    assert!(provider.get_runtime("7zip").is_some());
    assert!(provider.get_runtime("7z").is_some());
    assert!(provider.get_runtime("unknown").is_none());
}

#[test]
fn test_star_metadata() {
    let meta = vx_starlark::StarMetadata::parse(vx_provider_7zip::PROVIDER_STAR);
    assert!(meta.name.is_some());
    assert!(!meta.runtimes.is_empty());
}

#[test]
fn test_star_metadata_aliases_parsed() {
    // Test that aliases are correctly parsed from the actual provider.star
    let meta = vx_starlark::StarMetadata::parse(vx_provider_7zip::PROVIDER_STAR);
    assert_eq!(meta.runtimes.len(), 1, "Expected 1 runtime");
    let rt = &meta.runtimes[0];
    assert_eq!(rt.name, Some("7zip".to_string()));
    assert!(
        !rt.aliases.is_empty(),
        "Expected aliases to not be empty, got: {:?}",
        rt.aliases
    );
    assert!(
        rt.aliases.contains(&"7z".to_string()),
        "Expected '7z' in aliases"
    );
    assert!(
        rt.aliases.contains(&"7za".to_string()),
        "Expected '7za' in aliases"
    );
    assert!(
        rt.aliases.contains(&"7zz".to_string()),
        "Expected '7zz' in aliases"
    );
}
fn create_provider() -> std::sync::Arc<dyn vx_runtime::Provider> {
    let meta = vx_starlark::StarMetadata::parse(vx_provider_7zip::PROVIDER_STAR);
    let name: &'static str = Box::leak(
        meta.name
            .unwrap_or_else(|| "unknown".to_string())
            .into_boxed_str(),
    );
    vx_starlark::create_provider(name, vx_provider_7zip::PROVIDER_STAR)
}
