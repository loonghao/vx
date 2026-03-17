use vx_versions::VersionRequest;

#[test]
fn range_constraints_support_spaces_after_operators() {
    let req = VersionRequest::parse(">= 1.0.0 < 2.0.0");

    assert!(req.satisfies("1.0.0"));
    assert!(req.satisfies("1.9.9"));
    assert!(!req.satisfies("0.9.9"));
    assert!(!req.satisfies("2.0.0"));
}

#[test]
fn exact_constraints_support_double_equals_with_spaces() {
    let req = VersionRequest::parse("== 1.0.0");

    assert!(req.satisfies("1.0.0"));
    assert!(!req.satisfies("1.0.1"));
}

#[test]
fn mixed_comma_and_space_separated_ranges_are_supported() {
    let req = VersionRequest::parse(">= 1.2.3, < 2.0.0");

    assert!(req.satisfies("1.2.3"));
    assert!(req.satisfies("1.8.0"));
    assert!(!req.satisfies("1.2.2"));
    assert!(!req.satisfies("2.0.0"));
}
