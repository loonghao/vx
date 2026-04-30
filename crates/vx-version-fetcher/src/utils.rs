//! Version processing utilities
//!
//! Common functions for parsing, sorting, and filtering versions.
//!
//! String-level parsing and comparison is delegated to `vx-core::version_utils`.
//! This module adds `VersionInfo`-level helpers (sorting, creation) and extended
//! prerelease markers for ecosystem-specific tags like `canary` and `-snapshot`.

use std::cmp::Ordering;
use vx_runtime_core::version_utils as core_vu;
use vx_versions::VersionInfo;

/// Version processing utility functions
pub mod version_utils {
    use super::*;

    /// Extended prerelease markers for ecosystem fetchers.
    ///
    /// Superset of `vx-core`'s built-in markers, adding `canary` (Deno/Chrome)
    /// and `-snapshot` (Maven/JVM ecosystem).
    pub const DEFAULT_PRERELEASE_MARKERS: &[&str] = &[
        "-alpha",
        "-beta",
        "-rc",
        "-dev",
        "-pre",
        "canary",
        "-snapshot",
    ];

    /// Sort `VersionInfo` list descending (newest first).
    ///
    /// Uses `vx-core`'s `ParsedVersion` for correct semver ordering.
    ///
    /// # Examples
    /// ```
    /// use vx_version_fetcher::version_utils::sort_versions_desc;
    /// use vx_versions::VersionInfo;
    ///
    /// let mut versions = vec![
    ///     VersionInfo::new("1.0.0"),
    ///     VersionInfo::new("2.0.0"),
    ///     VersionInfo::new("1.5.0"),
    /// ];
    /// sort_versions_desc(&mut versions);
    /// assert_eq!(versions[0].version, "2.0.0");
    /// ```
    pub fn sort_versions_desc(versions: &mut [VersionInfo]) {
        versions.sort_by(|a, b| {
            let pa = core_vu::parse_version(&a.version);
            let pb = core_vu::parse_version(&b.version);
            match (pb, pa) {
                (Some(b), Some(a)) => b.cmp(&a),
                (Some(_), None) => Ordering::Less,
                (None, Some(_)) => Ordering::Greater,
                (None, None) => Ordering::Equal,
            }
        });
    }

    /// Check if a version string represents a prerelease, using default markers.
    ///
    /// Covers `-alpha`, `-beta`, `-rc`, `-dev`, `-pre`, `canary`, `-snapshot`.
    ///
    /// # Examples
    /// ```
    /// use vx_version_fetcher::version_utils::is_prerelease;
    /// assert!(is_prerelease("1.0.0-alpha"));
    /// assert!(is_prerelease("1.0.0-beta.1"));
    /// assert!(is_prerelease("1.0.0-rc.1"));
    /// assert!(is_prerelease("1.0.0-canary"));
    /// assert!(!is_prerelease("1.0.0"));
    /// ```
    pub fn is_prerelease(version: &str) -> bool {
        is_prerelease_with_markers(version, DEFAULT_PRERELEASE_MARKERS)
    }

    /// Check if version is prerelease with custom markers.
    ///
    /// # Examples
    /// ```
    /// use vx_version_fetcher::version_utils::is_prerelease_with_markers;
    /// assert!(is_prerelease_with_markers("1.0.0-canary", &["canary"]));
    /// assert!(!is_prerelease_with_markers("1.0.0", &["canary"]));
    /// ```
    pub fn is_prerelease_with_markers(version: &str, markers: &[&str]) -> bool {
        let lower = version.to_lowercase();
        markers.iter().any(|m| lower.contains(m))
    }

    /// Validate that a string is a parseable semver (at least `major.minor`).
    ///
    /// Delegates to `vx-core::version_utils::parse_version`.
    ///
    /// # Examples
    /// ```
    /// use vx_version_fetcher::version_utils::is_valid_semver;
    /// assert!(is_valid_semver("1.2.3"));
    /// assert!(is_valid_semver("1.2"));
    /// assert!(!is_valid_semver("1"));
    /// assert!(!is_valid_semver("abc"));
    /// ```
    pub fn is_valid_semver(version: &str) -> bool {
        core_vu::parse_version(version).is_some()
    }

    /// Strip a version prefix, or trim a leading `v` if prefix is empty.
    ///
    /// # Examples
    /// ```
    /// use vx_version_fetcher::version_utils::strip_version_prefix;
    /// assert_eq!(strip_version_prefix("v1.2.3", "v"), Some("1.2.3"));
    /// assert_eq!(strip_version_prefix("jq-1.7", "jq-"), Some("1.7"));
    /// assert_eq!(strip_version_prefix("bun-v1.0", "bun-v"), Some("1.0"));
    /// assert_eq!(strip_version_prefix("1.2.3", ""), Some("1.2.3"));
    /// ```
    pub fn strip_version_prefix<'a>(version: &'a str, prefix: &str) -> Option<&'a str> {
        if prefix.is_empty() {
            Some(version.trim_start_matches('v'))
        } else {
            version.strip_prefix(prefix)
        }
    }

    /// Compare two version strings, returning semver ordering.
    ///
    /// Delegates to `vx-core::version_utils::compare_versions_str`.
    /// Returns `Ordering::Equal` when either string cannot be parsed.
    ///
    /// # Examples
    /// ```
    /// use vx_version_fetcher::version_utils::compare_versions;
    /// use std::cmp::Ordering;
    /// assert_eq!(compare_versions("2.0.0", "1.0.0"), Ordering::Greater);
    /// assert_eq!(compare_versions("1.0.0", "2.0.0"), Ordering::Less);
    /// assert_eq!(compare_versions("1.0.0", "1.0.0"), Ordering::Equal);
    /// ```
    pub fn compare_versions(a: &str, b: &str) -> Ordering {
        core_vu::compare_versions_str(a, b).unwrap_or(Ordering::Equal)
    }

    /// Filter and transform raw version strings with common operations:
    /// strip prefix, skip prereleases, validate semver.
    pub fn filter_versions<'a>(
        versions: impl Iterator<Item = &'a str>,
        prefix: Option<&str>,
        skip_prereleases: bool,
        prerelease_markers: &[&str],
    ) -> Vec<String> {
        versions
            .filter_map(|v| {
                let version = match prefix {
                    Some(p) if !p.is_empty() => v.strip_prefix(p)?,
                    _ => v.trim_start_matches('v'),
                };

                if skip_prereleases && is_prerelease_with_markers(version, prerelease_markers) {
                    return None;
                }

                if !is_valid_semver(version) {
                    return None;
                }

                Some(version.to_string())
            })
            .collect()
    }

    /// Create a `VersionInfo` from a version string with optional metadata.
    pub fn create_version_info(
        version: &str,
        release_date: Option<&str>,
        is_lts: bool,
    ) -> VersionInfo {
        let mut info = VersionInfo::new(version)
            .with_prerelease(false)
            .with_lts(is_lts);

        if let Some(date) = release_date {
            info = info.with_release_date(date);
        }

        info
    }
}

/// Extension trait for VersionInfo for optional release date
pub trait VersionInfoExt {
    /// Set release date if Some, otherwise no-op
    fn with_optional_release_date(self, date: Option<String>) -> Self;
}

impl VersionInfoExt for VersionInfo {
    fn with_optional_release_date(self, date: Option<String>) -> Self {
        match date {
            Some(d) => self.with_release_date(d),
            None => self,
        }
    }
}
