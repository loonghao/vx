//! Version utility functions

use crate::VersionInfo;

/// Version comparison and manipulation utilities
pub struct VersionUtils;

impl VersionUtils {
    /// Sort versions in descending order (latest first)
    pub fn sort_versions_desc(mut versions: Vec<VersionInfo>) -> Vec<VersionInfo> {
        versions.sort_by(|a, b| {
            // Use semantic version comparison for better sorting
            match (
                Self::parse_semantic_version(&a.version),
                Self::parse_semantic_version(&b.version),
            ) {
                (Ok(va), Ok(vb)) => vb.cmp(&va), // Descending order
                _ => b.version.cmp(&a.version),  // Fallback to string comparison
            }
        });
        versions
    }

    /// Filter out prerelease versions
    pub fn filter_stable_only(versions: Vec<VersionInfo>) -> Vec<VersionInfo> {
        versions.into_iter().filter(|v| !v.prerelease).collect()
    }

    /// Get the latest N versions
    pub fn take_latest(versions: Vec<VersionInfo>, count: usize) -> Vec<VersionInfo> {
        versions.into_iter().take(count).collect()
    }

    /// Filter versions by LTS status
    pub fn filter_lts_only(versions: Vec<VersionInfo>) -> Vec<VersionInfo> {
        versions
            .into_iter()
            .filter(|v| {
                v.metadata
                    .get("lts")
                    .map(|val| val == "true")
                    .unwrap_or(false)
            })
            .collect()
    }

    /// Find a specific version in a list
    pub fn find_version<'a>(versions: &'a [VersionInfo], version: &str) -> Option<&'a VersionInfo> {
        versions.iter().find(|v| v.version == version)
    }

    /// Check if a version string matches a pattern
    pub fn matches_pattern(version: &str, pattern: &str) -> bool {
        match pattern {
            "latest" => true,
            "stable" => !Self::is_prerelease(version),
            "lts" => false, // Would need additional metadata
            _ => version == pattern,
        }
    }

    /// Check if a version string indicates a prerelease
    pub fn is_prerelease(version: &str) -> bool {
        version.contains("alpha")
            || version.contains("beta")
            || version.contains("rc")
            || version.contains("pre")
            || version.contains("dev")
            || version.contains("snapshot")
    }

    /// Clean version string (remove prefixes like 'v', 'go', etc.)
    pub fn clean_version(version: &str, prefixes: &[&str]) -> String {
        let mut cleaned = version.to_string();
        for prefix in prefixes {
            if cleaned.starts_with(prefix) {
                cleaned = cleaned[prefix.len()..].to_string();
                break;
            }
        }
        cleaned
    }

    /// Parse a semantic version string into comparable components
    fn parse_semantic_version(version: &str) -> Result<(u32, u32, u32, String), String> {
        let clean_version = version.trim_start_matches('v');
        let parts: Vec<&str> = clean_version.split('.').collect();

        if parts.len() < 2 {
            return Err(format!("Invalid version format: {}", version));
        }

        let major = parts[0]
            .parse::<u32>()
            .map_err(|_| format!("Invalid major version: {}", parts[0]))?;

        let minor = parts[1]
            .parse::<u32>()
            .map_err(|_| format!("Invalid minor version: {}", parts[1]))?;

        let (patch, suffix) = if parts.len() > 2 {
            let patch_part = parts[2];
            if let Some(dash_pos) = patch_part.find('-') {
                let patch_num = patch_part[..dash_pos]
                    .parse::<u32>()
                    .map_err(|_| format!("Invalid patch version: {}", &patch_part[..dash_pos]))?;
                let suffix = patch_part[dash_pos..].to_string();
                (patch_num, suffix)
            } else {
                let patch_num = patch_part
                    .parse::<u32>()
                    .map_err(|_| format!("Invalid patch version: {}", patch_part))?;
                (patch_num, String::new())
            }
        } else {
            (0, String::new())
        };

        Ok((major, minor, patch, suffix))
    }

    /// Compare two version strings
    pub fn compare_versions(a: &str, b: &str) -> std::cmp::Ordering {
        match (
            Self::parse_semantic_version(a),
            Self::parse_semantic_version(b),
        ) {
            (Ok(va), Ok(vb)) => va.cmp(&vb),
            _ => a.cmp(b), // Fallback to string comparison
        }
    }

    /// Check if version A is greater than version B
    pub fn is_greater_than(a: &str, b: &str) -> bool {
        Self::compare_versions(a, b) == std::cmp::Ordering::Greater
    }

    /// Check if version A is less than version B
    pub fn is_less_than(a: &str, b: &str) -> bool {
        Self::compare_versions(a, b) == std::cmp::Ordering::Less
    }

    /// Check if two versions are equal
    pub fn is_equal(a: &str, b: &str) -> bool {
        Self::compare_versions(a, b) == std::cmp::Ordering::Equal
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_utils() {
        assert!(VersionUtils::is_prerelease("1.0.0-alpha"));
        assert!(VersionUtils::is_prerelease("2.0.0-beta.1"));
        assert!(VersionUtils::is_prerelease("3.0.0-rc.1"));
        assert!(!VersionUtils::is_prerelease("1.0.0"));

        assert_eq!(VersionUtils::clean_version("v1.0.0", &["v"]), "1.0.0");
        assert_eq!(VersionUtils::clean_version("go1.21.0", &["go"]), "1.21.0");
        assert_eq!(VersionUtils::clean_version("1.0.0", &["v", "go"]), "1.0.0");
    }

    #[test]
    fn test_version_comparison() {
        assert!(VersionUtils::is_greater_than("1.2.3", "1.2.2"));
        assert!(VersionUtils::is_less_than("1.2.2", "1.2.3"));
        assert!(VersionUtils::is_equal("1.2.3", "1.2.3"));

        assert!(VersionUtils::is_greater_than("2.0.0", "1.9.9"));
        assert!(VersionUtils::is_greater_than("1.3.0", "1.2.9"));
    }

    #[test]
    fn test_matches_pattern() {
        assert!(VersionUtils::matches_pattern("1.2.3", "latest"));
        assert!(VersionUtils::matches_pattern("1.2.3", "stable"));
        assert!(!VersionUtils::matches_pattern("1.2.3-alpha", "stable"));
        assert!(VersionUtils::matches_pattern("1.2.3", "1.2.3"));
        assert!(!VersionUtils::matches_pattern("1.2.3", "1.2.4"));
    }

    #[test]
    fn test_filter_stable_only() {
        let versions = vec![
            VersionInfo::new("1.0.0".to_string()),
            VersionInfo::new("1.1.0-alpha".to_string()).with_prerelease(true),
            VersionInfo::new("1.1.0".to_string()),
        ];

        let stable = VersionUtils::filter_stable_only(versions);
        assert_eq!(stable.len(), 2);
        assert_eq!(stable[0].version, "1.0.0");
        assert_eq!(stable[1].version, "1.1.0");
    }
}
