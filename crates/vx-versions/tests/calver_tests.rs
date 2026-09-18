//! Calendar version (`YYYY-MM-DD`) parsing
//!
//! Tools that release by date instead of by semver (vcpkg is the main one) must
//! not be read as a semver version carrying a prerelease tag: `2025-12-16` has
//! to resolve to `2025.12.16`, otherwise every release looks like a prerelease
//! and `latest` resolution finds no candidate at all.

use vx_versions::Version;

#[test]
fn calver_parses_year_month_day() {
    let version = Version::parse("2025-12-16").expect("2025-12-16 should parse");

    assert_eq!(version, Version::new(2025, 12, 16));
    assert_eq!(version.to_string(), "2025.12.16");
}

#[test]
fn calver_is_not_treated_as_a_prerelease() {
    let version = Version::parse("2025-12-16").expect("2025-12-16 should parse");

    assert!(!version.is_prerelease());
}

#[test]
fn calver_versions_sort_in_date_order() {
    let january = Version::parse("2025-01-31").unwrap();
    let december = Version::parse("2025-12-16").unwrap();
    let next_year = Version::parse("2026-02-01").unwrap();

    assert!(january < december);
    assert!(december < next_year);
}

#[test]
fn is_calver_only_accepts_valid_dates() {
    assert!(Version::is_calver("2025-12-16"));
    assert!(Version::is_calver("2025-1-2"));

    // Regular semver, prereleases and dotted dates are not calendar versions.
    assert!(!Version::is_calver("1.2.3"));
    assert!(!Version::is_calver("1.2.3-beta.1"));
    assert!(!Version::is_calver("2025.12.16"));

    // Out-of-range month/day
    assert!(!Version::is_calver("2025-13-01"));
    assert!(!Version::is_calver("2025-12-45"));
    // Not a 4-digit year
    assert!(!Version::is_calver("25-12-16"));
}

#[test]
fn regular_semver_parsing_is_unaffected() {
    assert_eq!(Version::parse("1.2.3").unwrap(), Version::new(1, 2, 3));
    assert_eq!(Version::parse("v20.11.1").unwrap(), Version::new(20, 11, 1));
    assert_eq!(Version::parse("go1.22.0").unwrap(), Version::new(1, 22, 0));

    let prerelease = Version::parse("1.2.3-beta.1").unwrap();
    assert!(prerelease.is_prerelease());
    assert_eq!(prerelease.prerelease.as_deref(), Some("beta.1"));
}
