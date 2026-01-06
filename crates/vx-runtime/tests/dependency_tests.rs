//! Tests for RuntimeDependency version constraints in vx-runtime

use rstest::rstest;
use vx_runtime::RuntimeDependency;

#[rstest]
#[case("20.0.0", None, None, true)]
#[case("18.0.0", Some("16.0.0"), None, true)]
#[case("14.0.0", Some("16.0.0"), None, false)]
#[case("20.0.0", None, Some("22.0.0"), true)]
#[case("23.0.0", None, Some("22.0.0"), false)]
#[case("20.0.0", Some("18.0.0"), Some("22.0.0"), true)]
#[case("17.0.0", Some("18.0.0"), Some("22.0.0"), false)]
#[case("23.0.0", Some("18.0.0"), Some("22.0.0"), false)]
fn test_version_compatibility(
    #[case] version: &str,
    #[case] min_version: Option<&str>,
    #[case] max_version: Option<&str>,
    #[case] expected: bool,
) {
    let mut dep = RuntimeDependency::required("node");
    if let Some(min) = min_version {
        dep = dep.with_min_version(min);
    }
    if let Some(max) = max_version {
        dep = dep.with_max_version(max);
    }
    assert_eq!(
        dep.is_version_compatible(version),
        expected,
        "Version {} should be {} with min={:?}, max={:?}",
        version,
        if expected {
            "compatible"
        } else {
            "incompatible"
        },
        min_version,
        max_version
    );
}

#[test]
fn test_yarn_node_compatibility() {
    // Yarn 1.x compatibility: Node.js 12+ but < 23
    let dep = RuntimeDependency::required("node")
        .with_min_version("12.0.0")
        .with_max_version("22.99.99")
        .with_recommended_version("20")
        .with_reason("yarn requires Node.js runtime");

    // Compatible versions
    assert!(dep.is_version_compatible("20.10.0"));
    assert!(dep.is_version_compatible("18.19.0"));
    assert!(dep.is_version_compatible("22.0.0"));
    assert!(dep.is_version_compatible("12.0.0"));

    // Incompatible versions
    assert!(!dep.is_version_compatible("23.0.0"));
    assert!(!dep.is_version_compatible("23.11.0"));
    assert!(!dep.is_version_compatible("25.2.1")); // Node.js 25 should be incompatible
    assert!(!dep.is_version_compatible("11.0.0"));

    // Check recommended version
    assert_eq!(dep.recommended_version, Some("20".to_string()));
}

#[test]
fn test_dependency_builder() {
    let dep = RuntimeDependency::required("node")
        .with_min_version("18.0.0")
        .with_max_version("22.0.0")
        .with_recommended_version("20.10.0")
        .with_reason("test reason");

    assert_eq!(dep.name, "node");
    assert_eq!(dep.reason, Some("test reason".to_string()));
    assert_eq!(dep.min_version, Some("18.0.0".to_string()));
    assert_eq!(dep.max_version, Some("22.0.0".to_string()));
    assert_eq!(dep.recommended_version, Some("20.10.0".to_string()));
    assert!(!dep.optional);
}

#[test]
fn test_optional_dependency() {
    let dep = RuntimeDependency::optional("python").with_reason("optional for scripts");
    assert!(dep.optional);
    assert_eq!(dep.name, "python");
}

#[rstest]
#[case("20", "20", true)]
#[case("20.10", "20", true)]
#[case("20.10.0", "20", true)]
#[case("19.0.0", "20", false)]
#[case("20.0.0", "20.10", false)]
#[case("20.10.0", "20.10", true)]
#[case("20.11.0", "20.10", true)]
fn test_partial_version_min(#[case] version: &str, #[case] min: &str, #[case] expected: bool) {
    let dep = RuntimeDependency::required("node").with_min_version(min);
    assert_eq!(
        dep.is_version_compatible(version),
        expected,
        "Version {} should be {} with min={}",
        version,
        if expected {
            "compatible"
        } else {
            "incompatible"
        },
        min
    );
}
