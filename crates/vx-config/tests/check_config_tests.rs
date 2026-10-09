//! Tests for the `[check]` section of `vx.toml`.

use rstest::rstest;
use vx_config::{CheckConfig, ToolchainPinMismatch, parse_config_str};

fn parse_check(content: &str) -> CheckConfig {
    parse_config_str(content)
        .expect("config should parse")
        .check
        .expect("[check] section should be present")
}

#[test]
fn test_check_section_is_absent_by_default() {
    let config = parse_config_str("[tools]\nnode = \"20\"\n").unwrap();
    assert!(config.check.is_none());
}

#[test]
fn test_check_section_defaults_to_warn() {
    let check = parse_check("[check]\n");

    assert_eq!(check.pin_mismatch_severity(), ToolchainPinMismatch::Warn);
    assert!(!check.pin_mismatch_severity().is_error());
}

#[rstest]
#[case("warn", ToolchainPinMismatch::Warn)]
#[case("error", ToolchainPinMismatch::Error)]
#[case("ignore", ToolchainPinMismatch::Ignore)]
fn test_toolchain_pin_mismatch_parses(
    #[case] literal: &str,
    #[case] expected: ToolchainPinMismatch,
) {
    let check = parse_check(&format!(
        "[check]\ntoolchain_pin_mismatch = \"{literal}\"\n"
    ));

    assert_eq!(check.pin_mismatch_severity(), expected);
}

#[test]
fn test_toolchain_pin_mismatch_error_is_actionable() {
    let check = parse_check("[check]\ntoolchain_pin_mismatch = \"error\"\n");

    // "error" is the only level that may fail `vx check`.
    assert!(check.pin_mismatch_severity().is_error());
    assert!(!check.pin_mismatch_severity().is_ignored());
}

#[test]
fn test_toolchain_pin_mismatch_ignore_is_silent() {
    let check = parse_check("[check]\ntoolchain_pin_mismatch = \"ignore\"\n");

    assert!(check.pin_mismatch_severity().is_ignored());
    assert!(!check.pin_mismatch_severity().is_error());
}

#[test]
fn test_check_section_does_not_disturb_tools() {
    let config = parse_config_str(
        "[tools]\nrust = \"1.93.1\"\n\n[check]\ntoolchain_pin_mismatch = \"error\"\n",
    )
    .unwrap();

    assert_eq!(config.get_tool_version("rust"), Some("1.93.1".to_string()));
    assert!(config.check.unwrap().pin_mismatch_severity().is_error());
}

#[test]
fn test_unknown_severity_is_rejected() {
    // A typo must fail loudly rather than silently falling back to "warn".
    let result = parse_config_str("[check]\ntoolchain_pin_mismatch = \"shout\"\n");

    assert!(
        result.is_err(),
        "an unknown severity must be a parse error, not a silent default"
    );
}

#[test]
fn test_severity_round_trips_through_toml() {
    let check = parse_check("[check]\ntoolchain_pin_mismatch = \"error\"\n");
    let serialized = toml::to_string(&check).unwrap();

    assert!(
        serialized.contains("error"),
        "serialized config should keep the severity: {serialized}"
    );
}
