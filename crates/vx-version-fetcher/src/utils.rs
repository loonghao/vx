//! Version processing utilities
//!
//! Common functions for parsing, sorting, and filtering versions.

use vx_runtime::VersionInfo;

/// Version processing utility functions
pub mod version_utils {
    use super::*;

    /// Default prerelease markers
    pub const DEFAULT_PRERELEASE_MARKERS: &[&str] = &[
        "-alpha",
        "-beta",
        "-rc",
        "-dev",
        "canary",
        "-pre",
        "-snapshot",
    ];

    /// Parse semver to tuple (major, minor, patch)
    ///
    /// # Examples
    /// ```
    /// use vx_version_fetcher::version_utils::parse_semver_tuple;
    /// assert_eq!(parse_semver_tuple("1.2.3"), (1, 2, 3));
    /// assert_eq!(parse_semver_tuple("1.2"), (1, 2, 0));
    /// assert_eq!(parse_semver_tuple("1.2.3-alpha"), (1, 2, 3));
    /// ```
    pub fn parse_semver_tuple(v: &str) -> (u64, u64, u64) {
        let parts: Vec<&str> = v.split('.').collect();
        let major = parts.first().and_then(|s| s.parse().ok()).unwrap_or(0);
        let minor = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
        let patch = parts
            .get(2)
            .and_then(|s| s.split('-').next())
            .and_then(|s| s.split('+').next())
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);
        (major, minor, patch)
    }

    /// Sort versions descending (newest first)
    ///
    /// # Examples
    /// ```
    /// use vx_version_fetcher::version_utils::sort_versions_desc;
    /// use vx_runtime::VersionInfo;
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
        versions
            .sort_by(|a, b| parse_semver_tuple(&b.version).cmp(&parse_semver_tuple(&a.version)));
    }

    /// Check if version is prerelease using default markers
    ///
    /// # Examples
    /// ```
    /// use vx_version_fetcher::version_utils::is_prerelease;
    /// assert!(is_prerelease("1.0.0-alpha"));
    /// assert!(is_prerelease("1.0.0-beta.1"));
    /// assert!(is_prerelease("1.0.0-rc.1"));
    /// assert!(!is_prerelease("1.0.0"));
    /// ```
    pub fn is_prerelease(version: &str) -> bool {
        is_prerelease_with_markers(version, DEFAULT_PRERELEASE_MARKERS)
    }

    /// Check if version is prerelease with custom markers
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

    /// Validate basic semver format (at least major.minor)
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
        let parts: Vec<&str> = version.split('.').collect();
        parts.len() >= 2 && parts[0].parse::<u32>().is_ok()
    }

    /// Strip version prefix
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

    /// Compare two version strings
    ///
    /// Returns Ordering::Greater if a > b, Ordering::Less if a < b, Ordering::Equal if equal
    ///
    /// # Examples
    /// ```
    /// use vx_version_fetcher::version_utils::compare_versions;
    /// use std::cmp::Ordering;
    /// assert_eq!(compare_versions("2.0.0", "1.0.0"), Ordering::Greater);
    /// assert_eq!(compare_versions("1.0.0", "2.0.0"), Ordering::Less);
    /// assert_eq!(compare_versions("1.0.0", "1.0.0"), Ordering::Equal);
    /// ```
    pub fn compare_versions(a: &str, b: &str) -> std::cmp::Ordering {
        parse_semver_tuple(a).cmp(&parse_semver_tuple(b))
    }

    /// Filter and transform versions with common operations
    ///
    /// This is a helper that applies common filtering operations:
    /// - Strip prefix
    /// - Skip prereleases
    /// - Validate semver
    pub fn filter_versions<'a>(
        versions: impl Iterator<Item = &'a str>,
        prefix: Option<&str>,
        skip_prereleases: bool,
        prerelease_markers: &[&str],
    ) -> Vec<String> {
        versions
            .filter_map(|v| {
                // Strip prefix
                let version = match prefix {
                    Some(p) if !p.is_empty() => v.strip_prefix(p)?,
                    _ => v.trim_start_matches('v'),
                };

                // Skip prereleases if requested
                if skip_prereleases && is_prerelease_with_markers(version, prerelease_markers) {
                    return None;
                }

                // Validate basic semver
                if !is_valid_semver(version) {
                    return None;
                }

                Some(version.to_string())
            })
            .collect()
    }

    /// Create VersionInfo from version string with optional release date
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
