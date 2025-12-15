//! UV runtime tests

use rstest::rstest;
use vx_provider_uv::{UvProvider, UvRuntime, UvxRuntime};
use vx_runtime::{Ecosystem, Provider, Runtime};

#[test]
fn test_uv_runtime_creation() {
    let runtime = UvRuntime::new();
    assert_eq!(runtime.name(), "uv");
    assert!(!runtime.description().is_empty());
    assert_eq!(runtime.ecosystem(), Ecosystem::Python);
}

#[test]
fn test_uvx_runtime_creation() {
    let runtime = UvxRuntime::new();
    assert_eq!(runtime.name(), "uvx");
    assert!(!runtime.description().is_empty());
    assert_eq!(runtime.ecosystem(), Ecosystem::Python);
}

#[test]
fn test_uv_runtime_metadata() {
    let runtime = UvRuntime::new();
    let metadata = runtime.metadata();

    assert!(metadata.contains_key("homepage"));
    assert!(metadata.contains_key("ecosystem"));
    assert_eq!(metadata.get("ecosystem"), Some(&"python".to_string()));
}

#[test]
fn test_uv_provider_creation() {
    let provider = UvProvider::new();
    assert_eq!(provider.name(), "uv");
    assert!(!provider.description().is_empty());
}

#[test]
fn test_uv_provider_runtimes() {
    let provider = UvProvider::new();
    let runtimes = provider.runtimes();

    assert_eq!(runtimes.len(), 2);

    let names: Vec<&str> = runtimes.iter().map(|r| r.name()).collect();
    assert!(names.contains(&"uv"));
    assert!(names.contains(&"uvx"));
}

#[rstest]
#[case("uv", true)]
#[case("uvx", true)]
#[case("node", false)]
#[case("python", false)]
fn test_uv_provider_supports(#[case] name: &str, #[case] expected: bool) {
    let provider = UvProvider::new();
    assert_eq!(provider.supports(name), expected);
}

#[test]
fn test_uv_provider_get_runtime() {
    let provider = UvProvider::new();

    let uv = provider.get_runtime("uv");
    assert!(uv.is_some());
    assert_eq!(uv.unwrap().name(), "uv");

    let uvx = provider.get_runtime("uvx");
    assert!(uvx.is_some());
    assert_eq!(uvx.unwrap().name(), "uvx");

    let unknown = provider.get_runtime("unknown");
    assert!(unknown.is_none());
}
