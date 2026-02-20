//! imagemagick provider tests

use rstest::rstest;
use vx_provider_imagemagick::create_provider;
use vx_runtime::{Provider, Runtime};

#[test]
fn test_provider_name() {
    let provider = create_provider();
    assert_eq!(provider.name(), "imagemagick");
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
    assert!(names.contains(&"magick"));
    assert!(names.contains(&"convert"));
}

#[rstest]
#[case("magick", true)]
#[case("imagemagick", true)]
#[case("convert", true)]
#[case("node", false)]
fn test_provider_supports(#[case] name: &str, #[case] expected: bool) {
    let provider = create_provider();
    assert_eq!(provider.supports(name), expected);
}

#[test]
fn test_provider_get_runtime() {
    let provider = create_provider();
    assert!(provider.get_runtime("magick").is_some());
    assert!(provider.get_runtime("imagemagick").is_some());
    assert!(provider.get_runtime("convert").is_some());
    assert!(provider.get_runtime("unknown").is_none());
}

#[test]
fn test_star_metadata() {
    let meta = vx_provider_imagemagick::star_metadata();
    assert!(meta.name.is_some());
    assert!(!meta.runtimes.is_empty());
}
