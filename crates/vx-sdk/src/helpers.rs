//! Helper utilities for tool development
//!
//! This module provides common utilities for building tools, including
//! URL builders, version utilities, and platform helpers.

/// Platform-specific URL builder utilities
pub struct PlatformUrlBuilder;

impl PlatformUrlBuilder {
    /// Get standard platform string for downloads
    pub fn get_platform_string() -> String {
        #[cfg(target_os = "windows")]
        {
            "windows".to_string()
        }
        #[cfg(target_os = "macos")]
        {
            "darwin".to_string()
        }
        #[cfg(target_os = "linux")]
        {
            "linux".to_string()
        }
        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        {
            "unknown".to_string()
        }
    }

    /// Get architecture string
    pub fn get_arch_string() -> String {
        #[cfg(target_arch = "x86_64")]
        {
            "x64".to_string()
        }
        #[cfg(target_arch = "aarch64")]
        {
            "arm64".to_string()
        }
        #[cfg(target_arch = "x86")]
        {
            "x86".to_string()
        }
        #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64", target_arch = "x86")))]
        {
            "unknown".to_string()
        }
    }

    /// Get archive extension for current platform
    pub fn get_archive_extension() -> &'static str {
        if cfg!(windows) {
            "zip"
        } else {
            "tar.gz"
        }
    }

    /// Get executable extension for current platform
    pub fn get_exe_extension() -> &'static str {
        if cfg!(windows) {
            ".exe"
        } else {
            ""
        }
    }
}

/// Common URL building utilities
pub struct UrlUtils;

impl UrlUtils {
    /// Build GitHub release URL
    pub fn github_release_url(owner: &str, repo: &str, version: &str, filename: &str) -> String {
        format!(
            "https://github.com/{}/{}/releases/download/{}/{}",
            owner, repo, version, filename
        )
    }

    /// Build official download URL
    pub fn official_download_url(base_url: &str, version: &str, filename: &str) -> String {
        format!("{}/v{}/{}", base_url, version, filename)
    }

    /// Build GitHub API releases URL
    pub fn github_api_releases_url(owner: &str, repo: &str) -> String {
        format!("https://api.github.com/repos/{}/{}/releases", owner, repo)
    }
}

/// Version utilities
pub struct VersionUtils;

impl VersionUtils {
    /// Check if version is "latest"
    pub fn is_latest(version: &str) -> bool {
        version == "latest" || version == "stable"
    }

    /// Normalize version string (remove 'v' prefix)
    pub fn normalize_version(version: &str) -> String {
        version.strip_prefix('v').unwrap_or(version).to_string()
    }

    /// Check if version is prerelease
    pub fn is_prerelease(version: &str) -> bool {
        version.contains('-')
            && (version.contains("alpha")
                || version.contains("beta")
                || version.contains("rc")
                || version.contains("pre"))
    }

    /// Compare two semantic versions
    pub fn compare_versions(a: &str, b: &str) -> std::cmp::Ordering {
        let a_parts: Vec<u32> = Self::normalize_version(a)
            .split('.')
            .filter_map(|s| s.split('-').next())
            .filter_map(|s| s.parse().ok())
            .collect();

        let b_parts: Vec<u32> = Self::normalize_version(b)
            .split('.')
            .filter_map(|s| s.split('-').next())
            .filter_map(|s| s.parse().ok())
            .collect();

        for i in 0..std::cmp::max(a_parts.len(), b_parts.len()) {
            let a_val = a_parts.get(i).copied().unwrap_or(0);
            let b_val = b_parts.get(i).copied().unwrap_or(0);
            match a_val.cmp(&b_val) {
                std::cmp::Ordering::Equal => continue,
                other => return other,
            }
        }

        std::cmp::Ordering::Equal
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_utils() {
        assert!(VersionUtils::is_latest("latest"));
        assert!(VersionUtils::is_latest("stable"));
        assert!(!VersionUtils::is_latest("1.0.0"));

        assert_eq!(VersionUtils::normalize_version("v1.0.0"), "1.0.0");
        assert_eq!(VersionUtils::normalize_version("1.0.0"), "1.0.0");

        assert!(VersionUtils::is_prerelease("1.0.0-beta.1"));
        assert!(!VersionUtils::is_prerelease("1.0.0"));
    }

    #[test]
    fn test_url_utils() {
        let url = UrlUtils::github_release_url("owner", "repo", "v1.0.0", "file.zip");
        assert_eq!(
            url,
            "https://github.com/owner/repo/releases/download/v1.0.0/file.zip"
        );
    }

    #[test]
    fn test_version_compare() {
        assert_eq!(
            VersionUtils::compare_versions("1.0.0", "1.0.0"),
            std::cmp::Ordering::Equal
        );
        assert_eq!(
            VersionUtils::compare_versions("1.0.0", "2.0.0"),
            std::cmp::Ordering::Less
        );
        assert_eq!(
            VersionUtils::compare_versions("2.0.0", "1.0.0"),
            std::cmp::Ordering::Greater
        );
    }
}
