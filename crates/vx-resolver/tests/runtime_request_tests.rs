//! Tests for RuntimeRequest parsing

use rstest::rstest;
use vx_resolver::RuntimeRequest;

#[rstest]
#[case("yarn", "yarn", None)]
#[case("node", "node", None)]
#[case("npm", "npm", None)]
fn test_parse_name_only(
    #[case] input: &str,
    #[case] expected_name: &str,
    #[case] expected_version: Option<&str>,
) {
    let req = RuntimeRequest::parse(input);
    assert_eq!(req.name, expected_name);
    assert_eq!(req.version.as_deref(), expected_version);
}

#[rstest]
#[case("yarn@1.21.1", "yarn", Some("1.21.1"))]
#[case("node@20", "node", Some("20"))]
#[case("node@20.10.0", "node", Some("20.10.0"))]
#[case("go@1.22", "go", Some("1.22"))]
fn test_parse_with_version(
    #[case] input: &str,
    #[case] expected_name: &str,
    #[case] expected_version: Option<&str>,
) {
    let req = RuntimeRequest::parse(input);
    assert_eq!(req.name, expected_name);
    assert_eq!(req.version.as_deref(), expected_version);
}

#[rstest]
#[case("node@^18.0.0", "node", Some("^18.0.0"))]
#[case("node@~18.0.0", "node", Some("~18.0.0"))]
#[case("node@>=18", "node", Some(">=18"))]
fn test_parse_with_semver_constraint(
    #[case] input: &str,
    #[case] expected_name: &str,
    #[case] expected_version: Option<&str>,
) {
    let req = RuntimeRequest::parse(input);
    assert_eq!(req.name, expected_name);
    assert_eq!(req.version.as_deref(), expected_version);
}

#[test]
fn test_parse_empty_version() {
    let req = RuntimeRequest::parse("yarn@");
    assert_eq!(req.name, "yarn");
    assert_eq!(req.version, None);
}

#[test]
fn test_display() {
    let req = RuntimeRequest::with_version("yarn", "1.21.1");
    assert_eq!(format!("{}", req), "yarn@1.21.1");

    let req = RuntimeRequest::new("yarn");
    assert_eq!(format!("{}", req), "yarn");
}

#[test]
fn test_version_or_latest() {
    let req = RuntimeRequest::new("yarn");
    assert_eq!(req.version_or_latest(), "latest");

    let req = RuntimeRequest::with_version("yarn", "1.21.1");
    assert_eq!(req.version_or_latest(), "1.21.1");
}

#[test]
fn test_has_version() {
    let req = RuntimeRequest::new("yarn");
    assert!(!req.has_version());

    let req = RuntimeRequest::with_version("yarn", "1.21.1");
    assert!(req.has_version());
}

#[test]
fn test_from_str() {
    let req: RuntimeRequest = "yarn@1.21.1".into();
    assert_eq!(req.name, "yarn");
    assert_eq!(req.version, Some("1.21.1".to_string()));
}

#[test]
fn test_from_string() {
    let req: RuntimeRequest = String::from("node@20").into();
    assert_eq!(req.name, "node");
    assert_eq!(req.version, Some("20".to_string()));
}
