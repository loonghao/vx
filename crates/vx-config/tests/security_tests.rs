//! Security configuration tests
//!
//! Tests for security-related configuration parsing and validation.

use rstest::rstest;
use vx_config::parse_config_str;

// ============================================
// Security Config Parsing Tests
// ============================================

#[test]
fn test_parse_security_config_basic() {
    let content = r#"
[security]
enabled = true
fail_on = "high"
"#;
    let config = parse_config_str(content).unwrap();
    let security = config.security.unwrap();
    assert_eq!(security.enabled, Some(true));
    assert_eq!(security.fail_on, Some("high".to_string()));
}

#[test]
fn test_parse_security_config_disabled() {
    let content = r#"
[security]
enabled = false
"#;
    let config = parse_config_str(content).unwrap();
    let security = config.security.unwrap();
    assert_eq!(security.enabled, Some(false));
}

#[rstest]
#[case("critical")]
#[case("high")]
#[case("medium")]
#[case("low")]
fn test_parse_security_fail_on_levels(#[case] level: &str) {
    let content = format!(
        r#"
[security]
fail_on = "{}"
"#,
        level
    );
    let config = parse_config_str(&content).unwrap();
    let security = config.security.unwrap();
    assert_eq!(security.fail_on, Some(level.to_string()));
}

#[test]
fn test_parse_security_audit_config() {
    let content = r#"
[security.audit]
enabled = true
ignore = ["CVE-2021-1234", "GHSA-5678"]
on_install = true
on_ci = true
"#;
    let config = parse_config_str(content).unwrap();
    let security = config.security.unwrap();
    let audit = security.audit.unwrap();
    assert_eq!(audit.enabled, Some(true));
    assert_eq!(audit.on_install, Some(true));
    assert_eq!(audit.on_ci, Some(true));
    assert_eq!(
        audit.ignore,
        vec!["CVE-2021-1234".to_string(), "GHSA-5678".to_string()]
    );
}

#[test]
fn test_parse_security_secrets_config() {
    let content = r#"
[security.secrets]
enabled = true
baseline = ".secrets-baseline"
exclude = ["*.test.js", "fixtures/"]
pre_commit = true
"#;
    let config = parse_config_str(content).unwrap();
    let security = config.security.unwrap();
    let secrets = security.secrets.unwrap();
    assert_eq!(secrets.enabled, Some(true));
    assert_eq!(secrets.baseline, Some(".secrets-baseline".to_string()));
    assert_eq!(secrets.pre_commit, Some(true));
    assert_eq!(
        secrets.exclude,
        vec!["*.test.js".to_string(), "fixtures/".to_string()]
    );
}

#[test]
fn test_parse_security_sast_config() {
    let content = r#"
[security.sast]
enabled = true
tool = "semgrep"
ruleset = "p/default"
"#;
    let config = parse_config_str(content).unwrap();
    let security = config.security.unwrap();
    let sast = security.sast.unwrap();
    assert_eq!(sast.enabled, Some(true));
    assert_eq!(sast.tool, Some("semgrep".to_string()));
    assert_eq!(sast.ruleset, Some("p/default".to_string()));
}

#[test]
fn test_parse_security_license_config() {
    let content = r#"
[security]
enabled = true
allowed_licenses = ["MIT", "Apache-2.0", "BSD-3-Clause"]
denied_licenses = ["GPL-3.0", "AGPL-3.0"]
"#;
    let config = parse_config_str(content).unwrap();
    let security = config.security.unwrap();
    assert_eq!(
        security.allowed_licenses,
        vec![
            "MIT".to_string(),
            "Apache-2.0".to_string(),
            "BSD-3-Clause".to_string()
        ]
    );
    assert_eq!(
        security.denied_licenses,
        vec!["GPL-3.0".to_string(), "AGPL-3.0".to_string()]
    );
}

#[test]
fn test_parse_full_security_config() {
    let content = r#"
[security]
enabled = true
fail_on = "high"
allowed_licenses = ["MIT", "Apache-2.0"]

[security.audit]
enabled = true
on_install = true

[security.secrets]
enabled = true
baseline = ".secrets-baseline"

[security.sast]
enabled = false
"#;
    let config = parse_config_str(content).unwrap();
    let security = config.security.unwrap();

    assert_eq!(security.enabled, Some(true));
    assert_eq!(security.fail_on, Some("high".to_string()));
    assert!(security.audit.is_some());
    assert!(security.secrets.is_some());
    assert!(security.sast.is_some());
}

#[test]
fn test_security_config_defaults() {
    let content = r#"
[security]
"#;
    let config = parse_config_str(content).unwrap();
    let security = config.security.unwrap();

    // All fields should be None by default
    assert!(security.enabled.is_none());
    assert!(security.fail_on.is_none());
    assert!(security.audit.is_none());
    assert!(security.secrets.is_none());
}
