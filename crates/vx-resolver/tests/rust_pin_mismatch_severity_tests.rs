//! Contract tests: `[check] toolchain_pin_mismatch` is honoured on every path that
//! can report a pin mismatch, not only the `vx check` subcommand.
//!
//! `ToolchainPinMismatch::Ignore` is documented as "Do not report the mismatch at
//! all." That promise was only kept by `vx check`; the resolve path that backs
//! `vx run -- cargo …` and `vx cargo …` warned unconditionally, so the documented
//! escape hatch did not exist where the noise actually appeared.
//!
//! These tests pin the config-plumbing half of the fix: `ProjectToolsConfig` carries
//! the `[check]` section through to the resolve stage. They deliberately need no Rust
//! toolchain, so they hold even where the end-to-end warning cannot be produced. The
//! end-to-end half lives in `crates/vx-cli/tests/rust_pin_check_tests.rs`, which is
//! the only place the warning's stderr is observable.

use std::collections::HashMap;

use vx_resolver::{CheckConfig, ProjectToolsConfig, ToolchainPinMismatch};

/// Build a `[check]` section carrying only a pin-mismatch severity.
fn check_with(severity: ToolchainPinMismatch) -> CheckConfig {
    CheckConfig {
        toolchain_pin_mismatch: Some(severity),
    }
}

fn tools() -> HashMap<String, String> {
    HashMap::from([("rust".to_string(), "1.0.0".to_string())])
}

/// The regression: `"ignore"` must be observable from the resolve path.
#[test]
fn test_ignore_severity_is_ignored() {
    let config = ProjectToolsConfig::from_tools_with_check(
        tools(),
        Some(check_with(ToolchainPinMismatch::Ignore)),
    );

    assert!(
        config.pin_mismatch_severity().is_ignored(),
        "ignore must suppress the mismatch on the resolve path too"
    );
}

/// The default must stay `warn`, so upgrading vx does not change behaviour for a
/// repository that never configured the section.
#[test]
fn test_absent_check_section_defaults_to_warn() {
    let config = ProjectToolsConfig::from_tools(tools());

    assert_eq!(
        config.pin_mismatch_severity(),
        ToolchainPinMismatch::Warn,
        "a vx.toml without a [check] section must behave as warn"
    );
    assert!(
        !config.pin_mismatch_severity().is_ignored(),
        "the default must not silently suppress the warning"
    );
}

/// An explicit `warn` is indistinguishable from the default.
#[test]
fn test_explicit_warn_is_not_ignored() {
    let config = ProjectToolsConfig::from_tools_with_check(
        tools(),
        Some(check_with(ToolchainPinMismatch::Warn)),
    );

    assert!(!config.pin_mismatch_severity().is_ignored());
    assert!(!config.pin_mismatch_severity().is_error());
}

/// `error` must not be conflated with `ignore`: only the latter suppresses the
/// warning. The resolve path warns in both the `warn` and `error` cases — failing a
/// build there would make an already-drifting repository unbuildable.
#[test]
fn test_error_severity_is_not_ignored() {
    let config = ProjectToolsConfig::from_tools_with_check(
        tools(),
        Some(check_with(ToolchainPinMismatch::Error)),
    );

    assert!(config.pin_mismatch_severity().is_error());
    assert!(
        !config.pin_mismatch_severity().is_ignored(),
        "error must not silence the warning; it only fails `vx check`"
    );
}

/// A `[check]` section with the key absent resolves to the default rather than
/// panicking or suppressing.
#[test]
fn test_check_section_without_the_key_defaults_to_warn() {
    let config = ProjectToolsConfig::from_tools_with_check(tools(), Some(CheckConfig::default()));

    assert_eq!(config.pin_mismatch_severity(), ToolchainPinMismatch::Warn);
}

/// `None` for the whole section behaves exactly like an absent section.
#[test]
fn test_none_check_section_defaults_to_warn() {
    let config = ProjectToolsConfig::from_tools_with_check(tools(), None);

    assert_eq!(config.pin_mismatch_severity(), ToolchainPinMismatch::Warn);
}

/// The severity must survive alongside the pin it governs: carrying `[check]` must
/// not disturb tool-version lookup.
#[test]
fn test_check_section_does_not_disturb_tool_versions() {
    let config = ProjectToolsConfig::from_tools_with_check(
        tools(),
        Some(check_with(ToolchainPinMismatch::Ignore)),
    );

    assert_eq!(config.declared_version("rust"), Some("1.0.0"));
    assert_eq!(config.get_version_with_fallback("cargo"), Some("1.0.0"));
}
