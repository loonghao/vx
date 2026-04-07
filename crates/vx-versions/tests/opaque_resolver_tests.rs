//! Tests for the opaque/system ecosystem version resolver.
//!
//! These tests verify that the opaque resolver correctly handles non-semver
//! version strings like "system" and falls back gracefully when users specify
//! numeric version hints (e.g. `msvc = "14.42"` in vx.toml).

use rstest::rstest;
use vx_versions::{Ecosystem, VersionInfo, VersionResolver};

fn system_only_versions() -> Vec<VersionInfo> {
    vec![VersionInfo::new("system").with_lts(true)]
}

fn mixed_versions() -> Vec<VersionInfo> {
    vec![
        VersionInfo::new("1.2.0").with_lts(true),
        VersionInfo::new("1.1.0"),
        VersionInfo::new("system"),
    ]
}

// ── Basic opaque resolution ──────────────────────────────────────────────────

#[rstest]
#[case("latest", "system")]
#[case("*", "system")]
#[case("any", "system")]
#[case("stable", "system")]
#[case("system", "system")]
fn test_opaque_resolver_standard_aliases(#[case] input: &str, #[case] expected: &str) {
    let resolver = VersionResolver::new();
    let result = resolver.resolve(input, &system_only_versions(), &Ecosystem::System);
    assert_eq!(result, Some(expected.to_string()));
}

// ── MSVC version hint fallback (the bug fix) ────────────────────────────────

#[rstest]
#[case("14.42", "system")]
#[case("14.42.34433", "system")]
#[case("17.0", "system")]
#[case("2022", "system")]
fn test_opaque_resolver_numeric_hint_falls_back_to_system(
    #[case] input: &str,
    #[case] expected: &str,
) {
    let resolver = VersionResolver::new();
    let available = system_only_versions();
    let result = resolver.resolve(input, &available, &Ecosystem::System);
    assert_eq!(
        result,
        Some(expected.to_string()),
        "version hint '{}' should fall back to '{}' for pure opaque providers",
        input,
        expected
    );
}

// ── Mixed lists should still do semver resolution ────────────────────────────

#[test]
fn test_opaque_resolver_mixed_list_does_semver_resolution() {
    let resolver = VersionResolver::new();
    let available = mixed_versions();
    // "1.2" should match "1.2.0" via semver partial matching
    let result = resolver.resolve("1.2", &available, &Ecosystem::System);
    assert_eq!(result, Some("1.2.0".to_string()));
}

// ── Case-insensitive matching ────────────────────────────────────────────────

#[test]
fn test_opaque_resolver_case_insensitive() {
    let resolver = VersionResolver::new();
    let available = vec![VersionInfo::new("System").with_lts(true)];
    let result = resolver.resolve("system", &available, &Ecosystem::System);
    assert_eq!(result, Some("System".to_string()));
}

// ── Empty available list ─────────────────────────────────────────────────────

#[test]
fn test_opaque_resolver_empty_available() {
    let resolver = VersionResolver::new();
    let result = resolver.resolve("14.42", &[], &Ecosystem::System);
    assert_eq!(result, None);
}
