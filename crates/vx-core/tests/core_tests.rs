//! Tests for vx-core types and utilities

use rstest::rstest;
use vx_core::WithDependency;

// ============================================================================
// WithDependency tests
// ============================================================================

#[rstest]
#[case("bun@1.1.0", "bun", Some("1.1.0"))]
#[case("node@20.0.0", "node", Some("20.0.0"))]
#[case("deno", "deno", None)]
#[case("go", "go", None)]
fn test_with_dependency_parse(
    #[case] spec: &str,
    #[case] expected_runtime: &str,
    #[case] expected_version: Option<&str>,
) {
    let dep = WithDependency::parse(spec);
    assert_eq!(dep.runtime, expected_runtime);
    assert_eq!(dep.version.as_deref(), expected_version);
}

#[test]
fn test_with_dependency_display_with_version() {
    let dep = WithDependency::new("node", Some("20.0.0".to_string()));
    assert_eq!(dep.to_string(), "node@20.0.0");
}

#[test]
fn test_with_dependency_display_without_version() {
    let dep = WithDependency::new("bun", None);
    assert_eq!(dep.to_string(), "bun");
}

#[test]
fn test_with_dependency_parse_many() {
    let specs = vec![
        "bun@1.1".to_string(),
        "deno".to_string(),
        "node@20".to_string(),
    ];
    let deps = WithDependency::parse_many(&specs);
    assert_eq!(deps.len(), 3);
    assert_eq!(deps[0].runtime, "bun");
    assert_eq!(deps[0].version, Some("1.1".to_string()));
    assert_eq!(deps[1].runtime, "deno");
    assert_eq!(deps[1].version, None);
    assert_eq!(deps[2].runtime, "node");
    assert_eq!(deps[2].version, Some("20".to_string()));
}

#[test]
fn test_with_dependency_equality() {
    let dep1 = WithDependency::parse("bun@1.1.0");
    let dep2 = WithDependency::new("bun", Some("1.1.0".to_string()));
    assert_eq!(dep1, dep2);
}

// ============================================================================
// Version resolution utility tests
// ============================================================================

#[rstest]
#[case("latest", true)]
#[case("LATEST", true)]
#[case("Latest", true)]
#[case("1.0.0", false)]
#[case("", false)]
fn test_is_latest_version(#[case] version: &str, #[case] expected: bool) {
    assert_eq!(vx_core::is_latest_version(version), expected);
}

#[test]
fn test_resolve_latest_version() {
    let versions = vec![
        "1.0.0".to_string(),
        "2.0.0".to_string(),
        "1.5.0".to_string(),
    ];
    assert_eq!(
        vx_core::resolve_latest_version("latest", &versions),
        Some("2.0.0".to_string())
    );
    assert_eq!(
        vx_core::resolve_latest_version("1.5.0", &versions),
        Some("1.5.0".to_string())
    );
    assert_eq!(vx_core::resolve_latest_version("latest", &Vec::new()), None);
}
