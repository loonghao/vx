//! Tests for extension dependency management

use rstest::rstest;
use vx_extension::ExtensionDependency;

#[rstest]
#[case("my-extension", "my-extension", None, false)]
#[case("my-ext", "my-ext", None, false)]
#[case("ext_with_underscore", "ext_with_underscore", None, false)]
fn test_parse_simple_dependency(
    #[case] input: &str,
    #[case] expected_name: &str,
    #[case] expected_version: Option<&str>,
    #[case] expected_optional: bool,
) {
    let dep = ExtensionDependency::parse(input);
    assert_eq!(dep.name, expected_name);
    assert_eq!(dep.version.as_deref(), expected_version);
    assert_eq!(dep.optional, expected_optional);
}

#[rstest]
#[case("my-ext >= 1.0.0", "my-ext", Some(">= 1.0.0"), false)]
#[case("my-ext < 2.0.0", "my-ext", Some("< 2.0.0"), false)]
#[case("my-ext == 1.5.0", "my-ext", Some("== 1.5.0"), false)]
#[case("my-ext ~= 1.4", "my-ext", Some("~= 1.4"), false)]
#[case("my-ext ^1.0.0", "my-ext", Some("^1.0.0"), false)]
fn test_parse_dependency_with_version(
    #[case] input: &str,
    #[case] expected_name: &str,
    #[case] expected_version: Option<&str>,
    #[case] expected_optional: bool,
) {
    let dep = ExtensionDependency::parse(input);
    assert_eq!(dep.name, expected_name);
    assert_eq!(dep.version.as_deref(), expected_version);
    assert_eq!(dep.optional, expected_optional);
}

#[rstest]
#[case("?optional-ext", "optional-ext", None, true)]
#[case("?opt-ext >= 1.0", "opt-ext", Some(">= 1.0"), true)]
fn test_parse_optional_dependency(
    #[case] input: &str,
    #[case] expected_name: &str,
    #[case] expected_version: Option<&str>,
    #[case] expected_optional: bool,
) {
    let dep = ExtensionDependency::parse(input);
    assert_eq!(dep.name, expected_name);
    assert_eq!(dep.version.as_deref(), expected_version);
    assert_eq!(dep.optional, expected_optional);
}

#[rstest]
#[case("1.0.0", "1.0.0", true)]
#[case("1.0.1", "1.0.0", true)]
#[case("2.0.0", "1.0.0", true)]
#[case("0.9.0", "1.0.0", false)]
#[case("0.9.9", "1.0.0", false)]
fn test_satisfies_gte(#[case] version: &str, #[case] required: &str, #[case] expected: bool) {
    let dep = ExtensionDependency::parse(&format!("test >= {}", required));
    assert_eq!(dep.satisfies(version), expected);
}

#[rstest]
#[case("1.0.0", "2.0.0", true)]
#[case("1.9.9", "2.0.0", true)]
#[case("2.0.0", "2.0.0", false)]
#[case("3.0.0", "2.0.0", false)]
fn test_satisfies_lt(#[case] version: &str, #[case] required: &str, #[case] expected: bool) {
    let dep = ExtensionDependency::parse(&format!("test < {}", required));
    assert_eq!(dep.satisfies(version), expected);
}

#[rstest]
#[case("1.0.0", "1.0.0", true)]
#[case("1.0.1", "1.0.0", false)]
#[case("0.9.9", "1.0.0", false)]
fn test_satisfies_exact(#[case] version: &str, #[case] required: &str, #[case] expected: bool) {
    let dep = ExtensionDependency::parse(&format!("test == {}", required));
    assert_eq!(dep.satisfies(version), expected);
}

#[rstest]
#[case("1.0.0", true)]
#[case("2.0.0", true)]
#[case("0.0.1", true)]
fn test_satisfies_no_constraint(#[case] version: &str, #[case] expected: bool) {
    let dep = ExtensionDependency::parse("test");
    assert_eq!(dep.satisfies(version), expected);
}

#[rstest]
#[case("1.4.0", "1.4", true)]
#[case("1.9.0", "1.4", true)]
#[case("1.3.0", "1.4", false)]
#[case("2.0.0", "1.4", false)]
fn test_satisfies_compatible_release(
    #[case] version: &str,
    #[case] required: &str,
    #[case] expected: bool,
) {
    let dep = ExtensionDependency::parse(&format!("test ~= {}", required));
    assert_eq!(dep.satisfies(version), expected);
}

#[rstest]
#[case("1.2.3", "1.2.3", true)]
#[case("1.9.9", "1.2.3", true)]
#[case("1.2.2", "1.2.3", false)]
#[case("2.0.0", "1.2.3", false)]
fn test_satisfies_caret(#[case] version: &str, #[case] required: &str, #[case] expected: bool) {
    let dep = ExtensionDependency::parse(&format!("test ^{}", required));
    assert_eq!(dep.satisfies(version), expected);
}

#[test]
fn test_dependency_resolution_is_satisfied_empty() {
    let resolution = vx_extension::DependencyResolution {
        target: "test".to_string(),
        resolved: std::collections::HashMap::new(),
        missing: vec![],
        conflicts: vec![],
        circular: vec![],
    };
    assert!(resolution.is_satisfied());
}

#[test]
fn test_dependency_resolution_is_satisfied_with_resolved() {
    let mut resolved = std::collections::HashMap::new();
    resolved.insert(
        "dep1".to_string(),
        std::path::PathBuf::from("/path/to/dep1"),
    );
    resolved.insert(
        "dep2".to_string(),
        std::path::PathBuf::from("/path/to/dep2"),
    );

    let resolution = vx_extension::DependencyResolution {
        target: "test".to_string(),
        resolved,
        missing: vec![],
        conflicts: vec![],
        circular: vec![],
    };
    assert!(resolution.is_satisfied());
}

#[test]
fn test_dependency_resolution_not_satisfied_missing() {
    let resolution = vx_extension::DependencyResolution {
        target: "test".to_string(),
        resolved: std::collections::HashMap::new(),
        missing: vec![vx_extension::dependencies::MissingDependency {
            name: "missing-dep".to_string(),
            version: Some(">= 1.0".to_string()),
        }],
        conflicts: vec![],
        circular: vec![],
    };
    assert!(!resolution.is_satisfied());
}

#[test]
fn test_dependency_resolution_summary() {
    let resolution = vx_extension::DependencyResolution {
        target: "test".to_string(),
        resolved: std::collections::HashMap::new(),
        missing: vec![vx_extension::dependencies::MissingDependency {
            name: "missing-dep".to_string(),
            version: Some(">= 1.0".to_string()),
        }],
        conflicts: vec![],
        circular: vec![],
    };

    let summary = resolution.summary();
    assert!(summary.contains("Missing dependencies"));
    assert!(summary.contains("missing-dep"));
    assert!(summary.contains(">= 1.0"));
}
