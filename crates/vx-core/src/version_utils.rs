//! Version parsing and comparison utilities
//!
//! This module provides shared version parsing logic for consistent
//! version handling across the vx ecosystem.
//!
//! ## Supported Version Formats
//!
//! - Standard semver: `1.2.3`, `0.6.27`
//! - With 'v' prefix: `v1.2.3`
//! - With 'vx-v' prefix: `vx-v1.2.3` (release-please format)
//! - With 'x-v' prefix: `x-v1.2.3`
//! - With prerelease: `1.2.3-beta.1`, `1.2.3-rc.1`
//!
//! ## Example
//!
//! ```rust
//! use vx_core::version_utils::{parse_version, compare_versions, is_prerelease};
//!
//! let v1 = parse_version("vx-v0.6.27").unwrap();
//! let v2 = parse_version("0.6.26").unwrap();
//! assert!(compare_versions(&v1, &v2).is_gt());
//!
//! assert!(!is_prerelease("vx-v0.6.27"));
//! assert!(is_prerelease("0.6.27-beta.1"));
//! ```

use std::cmp::Ordering;

/// Parsed semantic version with major, minor, patch components
/// and optional prerelease identifier
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedVersion {
    pub major: u64,
    pub minor: u64,
    pub patch: u64,
    /// Optional prerelease identifier (e.g., "beta.1", "rc.2")
    pub prerelease: Option<String>,
}

impl ParsedVersion {
    /// Create a new ParsedVersion
    pub fn new(major: u64, minor: u64, patch: u64) -> Self {
        Self {
            major,
            minor,
            patch,
            prerelease: None,
        }
    }

    /// Create a new ParsedVersion with prerelease
    pub fn with_prerelease(
        major: u64,
        minor: u64,
        patch: u64,
        prerelease: impl Into<String>,
    ) -> Self {
        Self {
            major,
            minor,
            patch,
            prerelease: Some(prerelease.into()),
        }
    }

    /// Check if this is a prerelease version
    pub fn is_prerelease(&self) -> bool {
        self.prerelease.is_some()
    }
}

impl Ord for ParsedVersion {
    fn cmp(&self, other: &Self) -> Ordering {
        // Compare major.minor.patch first
        match self.major.cmp(&other.major) {
            Ordering::Equal => {}
            ord => return ord,
        }
        match self.minor.cmp(&other.minor) {
            Ordering::Equal => {}
            ord => return ord,
        }
        match self.patch.cmp(&other.patch) {
            Ordering::Equal => {}
            ord => return ord,
        }

        // When major.minor.patch are equal:
        // - No prerelease > prerelease (stable > prerelease)
        // - Compare prerelease identifiers alphabetically
        match (&self.prerelease, &other.prerelease) {
            (None, None) => Ordering::Equal,
            (None, Some(_)) => Ordering::Greater, // stable > prerelease
            (Some(_), None) => Ordering::Less,    // prerelease < stable
            (Some(a), Some(b)) => a.cmp(b),       // compare prerelease strings
        }
    }
}

impl PartialOrd for ParsedVersion {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl std::fmt::Display for ParsedVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(ref pre) = self.prerelease {
            write!(f, "{}.{}.{}-{}", self.major, self.minor, self.patch, pre)
        } else {
            write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
        }
    }
}

/// Normalize a version string by removing common prefixes
///
/// Supported prefixes:
/// - `vx-v` (release-please format)
/// - `x-v`
/// - `v`
///
/// # Example
///
/// ```rust
/// use vx_core::version_utils::normalize_version;
///
/// assert_eq!(normalize_version("vx-v0.6.27"), "0.6.27");
/// assert_eq!(normalize_version("v1.0.0"), "1.0.0");
/// assert_eq!(normalize_version("0.6.27"), "0.6.27");
/// ```
pub fn normalize_version(version: &str) -> &str {
    version
        .trim_start_matches("vx-v")
        .trim_start_matches("x-v")
        .trim_start_matches('v')
}

/// Parse a version string into a ParsedVersion
///
/// Supports formats:
/// - `1.2.3` (standard semver)
/// - `v1.2.3` (with v prefix)
/// - `vx-v1.2.3` (release-please format)
/// - `1.2.3-beta.1` (with prerelease)
/// - `1.2` (two-part, patch defaults to 0)
///
/// # Returns
///
/// `Some(ParsedVersion)` if parsing succeeds, `None` otherwise
///
/// # Example
///
/// ```rust
/// use vx_core::version_utils::parse_version;
///
/// let v = parse_version("vx-v0.6.27").unwrap();
/// assert_eq!(v.major, 0);
/// assert_eq!(v.minor, 6);
/// assert_eq!(v.patch, 27);
/// assert!(v.prerelease.is_none());
///
/// let v2 = parse_version("1.0.0-beta.1").unwrap();
/// assert_eq!(v2.prerelease, Some("beta.1".to_string()));
/// ```
pub fn parse_version(version: &str) -> Option<ParsedVersion> {
    let normalized = normalize_version(version);

    if normalized.is_empty() {
        return None;
    }

    // Split by '-' to separate version from prerelease
    let (version_part, prerelease) = if let Some(idx) = normalized.find('-') {
        let pre = normalized[idx + 1..].to_string();
        (&normalized[..idx], Some(pre))
    } else {
        (normalized, None)
    };

    let parts: Vec<&str> = version_part.split('.').collect();

    if parts.len() < 2 {
        return None;
    }

    let major = parts[0].parse::<u64>().ok()?;
    let minor = parts[1].parse::<u64>().ok()?;
    let patch = parts
        .get(2)
        .and_then(|p| p.parse::<u64>().ok())
        .unwrap_or(0);

    Some(ParsedVersion {
        major,
        minor,
        patch,
        prerelease,
    })
}

/// Compare two version strings
///
/// Returns the ordering between the two versions.
/// Prerelease versions are considered less than their stable counterparts.
///
/// # Example
///
/// ```rust
/// use vx_core::version_utils::{compare_versions, parse_version};
/// use std::cmp::Ordering;
///
/// let v1 = parse_version("1.0.0").unwrap();
/// let v0 = parse_version("0.9.9").unwrap();
/// assert_eq!(compare_versions(&v1, &v0), Ordering::Greater);
/// ```
pub fn compare_versions(a: &ParsedVersion, b: &ParsedVersion) -> Ordering {
    a.cmp(b)
}

/// Compare two version strings directly
///
/// Convenience function that parses and compares version strings.
///
/// # Returns
///
/// `Some(Ordering)` if both versions can be parsed, `None` otherwise
pub fn compare_versions_str(a: &str, b: &str) -> Option<Ordering> {
    let va = parse_version(a)?;
    let vb = parse_version(b)?;
    Some(va.cmp(&vb))
}

/// Check if version_a is newer than version_b
///
/// This is a convenience function for checking if an update is available.
///
/// # Example
///
/// ```rust
/// use vx_core::version_utils::is_newer_version;
///
/// assert!(is_newer_version("1.0.0", "0.9.9"));
/// assert!(is_newer_version("0.6.27", "0.6.26"));
/// assert!(!is_newer_version("0.6.27", "0.6.27"));
/// assert!(!is_newer_version("0.6.26", "0.6.27"));
///
/// // Prerelease handling: stable is newer than prerelease of same version
/// assert!(is_newer_version("0.6.27", "0.6.27-beta.1"));
/// ```
pub fn is_newer_version(version_a: &str, version_b: &str) -> bool {
    compare_versions_str(version_a, version_b) == Some(Ordering::Greater)
}

/// Check if a version string represents a prerelease
///
/// Considers:
/// - Versions with `-alpha`, `-beta`, `-rc`, `-dev`, `-pre` suffixes
/// - Versions with `vx-v` or `x-v` prefixes are considered stable releases
///
/// # Example
///
/// ```rust
/// use vx_core::version_utils::is_prerelease;
///
/// assert!(!is_prerelease("vx-v0.6.27"));
/// assert!(!is_prerelease("0.6.27"));
/// assert!(is_prerelease("0.6.27-beta.1"));
/// assert!(is_prerelease("0.6.27-rc.1"));
/// ```
pub fn is_prerelease(version: &str) -> bool {
    // Normalize first to remove prefixes (vx-v, x-v, v)
    let normalized = normalize_version(version);

    // Check for prerelease suffixes in the normalized version
    normalized.contains("-alpha")
        || normalized.contains("-beta")
        || normalized.contains("-rc")
        || normalized.contains("-dev")
        || normalized.contains("-pre")
}

/// Sort a list of version strings in semver order (descending - newest first)
///
/// Invalid versions are placed at the end.
///
/// # Example
///
/// ```rust
/// use vx_core::version_utils::sort_versions_desc;
///
/// let mut versions = vec!["0.6.25", "0.6.27", "0.6.26", "invalid"];
/// sort_versions_desc(&mut versions);
/// assert_eq!(versions, vec!["0.6.27", "0.6.26", "0.6.25", "invalid"]);
/// ```
pub fn sort_versions_desc(versions: &mut [impl AsRef<str>]) {
    versions.sort_by(|a, b| {
        let va = parse_version(a.as_ref());
        let vb = parse_version(b.as_ref());
        match (va, vb) {
            (Some(a), Some(b)) => b.cmp(&a),   // Descending order
            (Some(_), None) => Ordering::Less, // Valid versions first
            (None, Some(_)) => Ordering::Greater,
            (None, None) => Ordering::Equal,
        }
    });
}

/// Find the latest version from a list of version strings
///
/// Optionally excludes prerelease versions.
///
/// # Example
///
/// ```rust
/// use vx_core::version_utils::find_latest_version;
///
/// let versions = vec!["0.6.25", "0.6.27", "0.6.26-beta.1", "0.6.26"];
/// assert_eq!(find_latest_version(&versions, false), Some("0.6.27"));
/// assert_eq!(find_latest_version(&versions, true), Some("0.6.27")); // excludes beta
/// ```
pub fn find_latest_version(
    versions: &[impl AsRef<str>],
    exclude_prerelease: bool,
) -> Option<&str> {
    versions
        .iter()
        .filter(|v| {
            if exclude_prerelease && is_prerelease(v.as_ref()) {
                return false;
            }
            parse_version(v.as_ref()).is_some()
        })
        .max_by(|a, b| {
            let va = parse_version(a.as_ref()).unwrap();
            let vb = parse_version(b.as_ref()).unwrap();
            va.cmp(&vb)
        })
        .map(|v| v.as_ref())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_version() {
        assert_eq!(normalize_version("vx-v0.6.27"), "0.6.27");
        assert_eq!(normalize_version("x-v0.6.27"), "0.6.27");
        assert_eq!(normalize_version("v0.6.27"), "0.6.27");
        assert_eq!(normalize_version("0.6.27"), "0.6.27");
        assert_eq!(normalize_version("vx-v1.0.0-beta.1"), "1.0.0-beta.1");
    }

    #[test]
    fn test_parse_version() {
        let v = parse_version("0.6.27").unwrap();
        assert_eq!(v.major, 0);
        assert_eq!(v.minor, 6);
        assert_eq!(v.patch, 27);
        assert!(v.prerelease.is_none());

        let v = parse_version("vx-v0.6.27").unwrap();
        assert_eq!(v.major, 0);
        assert_eq!(v.minor, 6);
        assert_eq!(v.patch, 27);

        let v = parse_version("1.0.0-beta.1").unwrap();
        assert_eq!(v.prerelease, Some("beta.1".to_string()));

        // Two-part version
        let v = parse_version("0.6").unwrap();
        assert_eq!(v.patch, 0);

        // Invalid versions
        assert!(parse_version("invalid").is_none());
        assert!(parse_version("").is_none());
        assert!(parse_version("v").is_none());
    }

    #[test]
    fn test_is_newer_version() {
        assert!(is_newer_version("1.0.0", "0.9.9"));
        assert!(is_newer_version("0.5.29", "0.5.28"));
        assert!(is_newer_version("1.0.0", "0.99.99"));
        assert!(!is_newer_version("0.5.28", "0.5.29"));
        assert!(!is_newer_version("0.5.28", "0.5.28"));

        // Prerelease handling
        assert!(is_newer_version("0.6.27", "0.6.27-beta.1"));
        assert!(!is_newer_version("0.6.27-beta.1", "0.6.27"));

        // Prefix handling
        assert!(is_newer_version("vx-v0.6.27", "vx-v0.6.26"));
        assert!(is_newer_version("vx-v0.6.27", "0.6.26"));
    }

    #[test]
    fn test_is_prerelease() {
        // Stable versions (no prerelease suffix)
        assert!(!is_prerelease("vx-v0.6.27"));
        assert!(!is_prerelease("x-v0.6.27"));
        assert!(!is_prerelease("v0.6.27"));
        assert!(!is_prerelease("0.6.27"));

        // Prerelease versions without prefix
        assert!(is_prerelease("0.6.27-alpha.1"));
        assert!(is_prerelease("0.6.27-beta.1"));
        assert!(is_prerelease("0.6.27-rc.1"));
        assert!(is_prerelease("0.6.27-dev"));
        assert!(is_prerelease("0.6.27-pre.1"));

        // Prerelease versions WITH prefix should also be detected
        assert!(is_prerelease("vx-v0.6.27-beta.1"));
        assert!(is_prerelease("vx-v0.6.27-alpha.1"));
        assert!(is_prerelease("x-v0.6.27-rc.1"));
        assert!(is_prerelease("v0.6.27-dev"));
    }

    #[test]
    fn test_version_ordering() {
        let v1 = ParsedVersion::new(0, 6, 27);
        let v2 = ParsedVersion::new(0, 6, 26);
        assert!(v1 > v2);

        let v3 = ParsedVersion::with_prerelease(0, 6, 27, "beta.1");
        assert!(v1 > v3); // stable > prerelease

        let v4 = ParsedVersion::with_prerelease(0, 6, 27, "alpha.1");
        assert!(v3 > v4); // beta > alpha (alphabetical)
    }

    #[test]
    fn test_sort_versions_desc() {
        let mut versions = vec!["0.6.25", "0.6.27", "0.6.26"];
        sort_versions_desc(&mut versions);
        assert_eq!(versions, vec!["0.6.27", "0.6.26", "0.6.25"]);

        // With invalid versions
        let mut versions = vec!["0.6.25", "invalid", "0.6.27"];
        sort_versions_desc(&mut versions);
        assert_eq!(versions[0], "0.6.27");
        assert_eq!(versions[1], "0.6.25");
    }

    #[test]
    fn test_find_latest_version() {
        let versions = vec!["0.6.25", "0.6.27", "0.6.26"];
        assert_eq!(find_latest_version(&versions, false), Some("0.6.27"));

        // With prerelease
        let versions = vec!["0.6.25", "0.6.28-beta.1", "0.6.27"];
        assert_eq!(find_latest_version(&versions, true), Some("0.6.27")); // excludes beta
        assert_eq!(find_latest_version(&versions, false), Some("0.6.28-beta.1"));
        // includes beta
    }

    // =========================================================================
    // Regression tests for fix/python-env-and-self-update branch
    // =========================================================================

    /// Regression test: Ensure {install_dir} fallback selects the LATEST version
    /// Bug: Previously used versions[0] which depended on filesystem ordering
    #[test]
    fn test_regression_install_dir_selects_latest_version() {
        // Simulate filesystem ordering that might not be semver-sorted
        let filesystem_ordered = vec!["18.0.0", "20.0.0", "19.5.0", "22.1.0", "21.0.0"];

        let latest = find_latest_version(&filesystem_ordered, false);

        // Should always pick 22.1.0, regardless of filesystem ordering
        assert_eq!(
            latest,
            Some("22.1.0"),
            "Should select latest version (22.1.0), not first in list (18.0.0)"
        );
    }

    /// Regression test: Prerelease versions should be less than stable
    /// Bug: 0.6.27-beta.1 was incorrectly considered newer than 0.6.27
    #[test]
    fn test_regression_stable_newer_than_prerelease() {
        // The key bug case: stable 0.6.27 should be newer than 0.6.27-beta.1
        assert!(
            is_newer_version("0.6.27", "0.6.27-beta.1"),
            "Stable 0.6.27 should be newer than 0.6.27-beta.1"
        );
        assert!(
            !is_newer_version("0.6.27-beta.1", "0.6.27"),
            "0.6.27-beta.1 should NOT be newer than stable 0.6.27"
        );

        // But prerelease of higher version IS newer than stable of lower version
        assert!(
            is_newer_version("0.6.28-beta.1", "0.6.27"),
            "0.6.28-beta.1 should be newer than 0.6.27"
        );
    }

    /// Regression test: self_update version comparison with various prefixes
    /// Bug: Different prefix formats (vx-v, x-v, v) caused comparison failures
    #[test]
    fn test_regression_cross_prefix_comparison() {
        // All these comparisons should work correctly
        assert!(is_newer_version("vx-v0.6.27", "vx-v0.6.26"));
        assert!(is_newer_version("vx-v0.6.27", "x-v0.6.26"));
        assert!(is_newer_version("vx-v0.6.27", "v0.6.26"));
        assert!(is_newer_version("vx-v0.6.27", "0.6.26"));
        assert!(is_newer_version("x-v0.6.27", "vx-v0.6.26"));
        assert!(is_newer_version("v0.6.27", "vx-v0.6.26"));
        assert!(is_newer_version("0.6.27", "vx-v0.6.26"));

        // Equal versions with different prefixes should NOT be "newer"
        assert!(!is_newer_version("vx-v0.6.27", "0.6.27"));
        assert!(!is_newer_version("0.6.27", "vx-v0.6.27"));
    }

    /// Regression test: sort_versions_desc with mixed valid/invalid entries
    /// Bug: Invalid versions could break sorting or appear at wrong positions
    #[test]
    fn test_regression_sort_with_invalid_versions() {
        let mut versions = vec![
            "20.0.0",
            "temp",   // Invalid
            ".cache", // Invalid (hidden directory)
            "18.0.0",
            "invalid_dir", // Invalid
            "22.0.0",
            "19.5.0-rc.1", // Prerelease
        ];

        sort_versions_desc(&mut versions);

        // Valid versions should be sorted first, descending
        assert_eq!(versions[0], "22.0.0");
        assert_eq!(versions[1], "20.0.0");
        // 19.5.0-rc.1 should come before 18.0.0 (it's a higher version)
        assert_eq!(versions[2], "19.5.0-rc.1");
        assert_eq!(versions[3], "18.0.0");
        // Invalid versions should be at the end
    }

    /// Regression test: find_latest_version with only prerelease versions
    #[test]
    fn test_regression_only_prerelease_versions() {
        let versions = vec!["0.6.27-alpha.1", "0.6.27-beta.1", "0.6.27-rc.1"];

        // Without excluding prerelease, should find rc.1 (latest alphabetically among prereleases)
        let latest = find_latest_version(&versions, false);
        assert_eq!(latest, Some("0.6.27-rc.1"));

        // With excluding prerelease, should find nothing
        let latest = find_latest_version(&versions, true);
        assert_eq!(
            latest, None,
            "Should return None when all versions are prerelease and exclude_prerelease=true"
        );
    }

    /// Regression test: version parsing with platform suffix (e.g., "3.11.0-linux-x64")
    /// Note: This is NOT a prerelease, but could be confused with one
    #[test]
    fn test_regression_version_with_platform_suffix() {
        // Platform suffixes in directory names should still parse the version part
        let v = parse_version("3.11.0");
        assert!(v.is_some());
        assert_eq!(v.as_ref().unwrap().major, 3);
        assert_eq!(v.as_ref().unwrap().minor, 11);
        assert_eq!(v.as_ref().unwrap().patch, 0);

        // But "3.11.0-linux-x64" would be parsed as having prerelease "linux-x64"
        // This is expected behavior - the directory structure should use
        // version/platform layout, not version-platform
        let v = parse_version("3.11.0-linux-x64");
        assert!(v.is_some());
        assert_eq!(
            v.as_ref().unwrap().prerelease,
            Some("linux-x64".to_string())
        );
    }

    /// Regression test: compare_versions_str with invalid inputs
    #[test]
    fn test_regression_compare_invalid_versions() {
        // Should return None when either version is invalid
        assert_eq!(compare_versions_str("invalid", "0.6.27"), None);
        assert_eq!(compare_versions_str("0.6.27", "invalid"), None);
        assert_eq!(compare_versions_str("invalid", "also_invalid"), None);

        // Valid comparison should return Some
        assert!(compare_versions_str("0.6.27", "0.6.26").is_some());
    }

    /// Regression test: prerelease ordering (alpha < beta < rc)
    #[test]
    fn test_regression_prerelease_ordering() {
        let alpha = ParsedVersion::with_prerelease(0, 6, 27, "alpha.1");
        let beta = ParsedVersion::with_prerelease(0, 6, 27, "beta.1");
        let rc = ParsedVersion::with_prerelease(0, 6, 27, "rc.1");
        let stable = ParsedVersion::new(0, 6, 27);

        // Ordering: alpha < beta < rc < stable
        assert!(alpha < beta);
        assert!(beta < rc);
        assert!(rc < stable);

        // Transitive
        assert!(alpha < stable);
        assert!(beta < stable);
    }

    /// Regression test: two-part versions (e.g., "3.11" vs "3.11.0")
    #[test]
    fn test_regression_two_part_version_comparison() {
        // Two-part versions should have patch = 0
        let v1 = parse_version("3.11").unwrap();
        let v2 = parse_version("3.11.0").unwrap();

        assert_eq!(v1.major, v2.major);
        assert_eq!(v1.minor, v2.minor);
        assert_eq!(v1.patch, v2.patch); // Both should be 0

        // They should be equal
        assert_eq!(v1.cmp(&v2), Ordering::Equal);
        assert!(!is_newer_version("3.11", "3.11.0"));
        assert!(!is_newer_version("3.11.0", "3.11"));
    }

    /// Regression test: large version numbers
    #[test]
    fn test_regression_large_version_numbers() {
        let v = parse_version("100.200.300").unwrap();
        assert_eq!(v.major, 100);
        assert_eq!(v.minor, 200);
        assert_eq!(v.patch, 300);

        assert!(is_newer_version("100.200.300", "99.999.999"));
        assert!(is_newer_version("1.0.0", "0.999.999"));
    }

    /// Regression test: empty version list
    #[test]
    fn test_regression_empty_version_list() {
        let empty: Vec<&str> = vec![];
        assert_eq!(find_latest_version(&empty, false), None);
        assert_eq!(find_latest_version(&empty, true), None);
    }

    /// Regression test: single version in list
    #[test]
    fn test_regression_single_version() {
        let single = vec!["0.6.27"];
        assert_eq!(find_latest_version(&single, false), Some("0.6.27"));

        let single_prerelease = vec!["0.6.27-beta.1"];
        assert_eq!(
            find_latest_version(&single_prerelease, false),
            Some("0.6.27-beta.1")
        );
        assert_eq!(find_latest_version(&single_prerelease, true), None);
    }
}
