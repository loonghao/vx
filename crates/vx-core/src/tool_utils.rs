//! Shared utilities for tool implementations
//!
//! This module provides common functionality to reduce code duplication
//! across different tool implementations.

use crate::{Result, VxError};
use std::collections::HashMap;

/// Common platform detection utilities
pub struct PlatformUtils;

impl PlatformUtils {
    /// Get the current platform string in the format used by most tools
    pub fn get_platform_string() -> String {
        let os = if cfg!(target_os = "windows") {
            "windows"
        } else if cfg!(target_os = "macos") {
            "darwin"
        } else {
            "linux"
        };

        let arch = if cfg!(target_arch = "x86_64") {
            "x64"
        } else if cfg!(target_arch = "aarch64") {
            "arm64"
        } else {
            "x64" // fallback
        };

        format!("{}-{}", os, arch)
    }

    /// Get platform-specific executable extension
    pub fn get_executable_extension() -> &'static str {
        if cfg!(target_os = "windows") {
            ".exe"
        } else {
            ""
        }
    }

    /// Get platform-specific archive extension
    pub fn get_archive_extension() -> &'static str {
        if cfg!(target_os = "windows") {
            ".zip"
        } else {
            ".tar.gz"
        }
    }
}

/// Common URL building utilities
pub struct UrlUtils;

impl UrlUtils {
    /// Build a GitHub release download URL
    pub fn github_release_url(owner: &str, repo: &str, version: &str, filename: &str) -> String {
        format!(
            "https://github.com/{}/{}/releases/download/{}/{}",
            owner, repo, version, filename
        )
    }

    /// Build a GitHub API releases URL
    pub fn github_api_releases_url(owner: &str, repo: &str) -> String {
        format!("https://api.github.com/repos/{}/{}/releases", owner, repo)
    }

    /// Build a Node.js download URL
    pub fn nodejs_download_url(version: &str) -> String {
        let platform = PlatformUtils::get_platform_string();
        let ext = PlatformUtils::get_archive_extension();
        format!(
            "https://nodejs.org/dist/v{}/node-v{}-{}{}",
            version, version, platform, ext
        )
    }
}

/// Common version parsing utilities
pub struct VersionUtils;

impl VersionUtils {
    /// Parse GitHub releases JSON to extract version information
    pub fn parse_github_releases(
        json: &str,
        include_prerelease: bool,
    ) -> Result<Vec<crate::VersionInfo>> {
        let releases: Vec<serde_json::Value> =
            serde_json::from_str(json).map_err(|e| VxError::ParseError {
                message: format!("Failed to parse GitHub releases: {}", e),
            })?;

        let mut versions = Vec::new();
        for release in releases {
            let tag_name = release["tag_name"]
                .as_str()
                .ok_or_else(|| VxError::ParseError {
                    message: "Missing tag_name in release".to_string(),
                })?;

            let is_prerelease = release["prerelease"].as_bool().unwrap_or(false);
            if !include_prerelease && is_prerelease {
                continue;
            }

            // Remove 'v' prefix if present
            let version = tag_name.trim_start_matches('v');

            let mut version_info =
                crate::VersionInfo::new(version.to_string()).with_prerelease(is_prerelease);

            if let Some(date) = release["published_at"].as_str() {
                version_info = version_info.with_release_date(date.to_string());
            }

            versions.push(version_info);
        }

        Ok(versions)
    }

    /// Clean version string by removing common prefixes
    pub fn clean_version(version: &str) -> String {
        version
            .trim_start_matches('v')
            .trim_start_matches('V')
            .to_string()
    }

    /// Check if a version string looks like a prerelease
    pub fn is_prerelease(version: &str) -> bool {
        let lower = version.to_lowercase();
        lower.contains("alpha")
            || lower.contains("beta")
            || lower.contains("rc")
            || lower.contains("pre")
            || lower.contains("dev")
            || lower.contains("snapshot")
    }
}

/// Common metadata utilities
pub struct MetadataUtils;

impl MetadataUtils {
    /// Create a standard metadata map for tools
    pub fn create_tool_metadata(
        name: &str,
        ecosystem: &str,
        homepage: Option<&str>,
        repository: Option<&str>,
        license: Option<&str>,
    ) -> HashMap<String, String> {
        let mut metadata = HashMap::new();

        metadata.insert("name".to_string(), name.to_string());
        metadata.insert("ecosystem".to_string(), ecosystem.to_string());

        if let Some(homepage) = homepage {
            metadata.insert("homepage".to_string(), homepage.to_string());
        }

        if let Some(repository) = repository {
            metadata.insert("repository".to_string(), repository.to_string());
        }

        if let Some(license) = license {
            metadata.insert("license".to_string(), license.to_string());
        }

        metadata
    }
}

/// Common validation utilities
pub struct ValidationUtils;

impl ValidationUtils {
    /// Validate that a version string is reasonable
    pub fn validate_version(version: &str) -> Result<()> {
        if version.is_empty() {
            return Err(VxError::Other {
                message: "Version cannot be empty".to_string(),
            });
        }

        // Basic semver-like validation
        let clean_version = VersionUtils::clean_version(version);
        if !clean_version.chars().next().unwrap_or('0').is_ascii_digit() {
            return Err(VxError::Other {
                message: format!("Version '{}' must start with a digit", version),
            });
        }

        Ok(())
    }

    /// Validate tool name
    pub fn validate_tool_name(name: &str) -> Result<()> {
        if name.is_empty() {
            return Err(VxError::Other {
                message: "Tool name cannot be empty".to_string(),
            });
        }

        if name.contains(char::is_whitespace) {
            return Err(VxError::Other {
                message: format!("Tool name '{}' cannot contain whitespace", name),
            });
        }

        Ok(())
    }
}

/// Macro to reduce boilerplate in tool implementations
#[macro_export]
macro_rules! impl_tool_basics {
    ($tool_type:ty, $name:literal, $description:literal, $aliases:expr) => {
        impl $tool_type {
            pub fn name(&self) -> &str {
                $name
            }

            pub fn description(&self) -> &str {
                $description
            }

            pub fn aliases(&self) -> Vec<&str> {
                $aliases
            }
        }
    };
}

/// Macro to create standard GitHub-based tool implementations
#[macro_export]
macro_rules! github_tool {
    (
        $tool_name:ident,
        name: $name:literal,
        description: $desc:literal,
        owner: $owner:literal,
        repo: $repo:literal,
        ecosystem: $ecosystem:literal
        $(, aliases: [$($alias:literal),*])?
        $(, homepage: $homepage:literal)?
        $(, license: $license:literal)?
    ) => {
        #[derive(Debug, Clone)]
        pub struct $tool_name;

        impl $tool_name {
            pub fn new() -> Self {
                Self
            }

            fn get_github_releases_url(&self) -> String {
                $crate::tool_utils::UrlUtils::github_api_releases_url($owner, $repo)
            }

            fn build_download_url(&self, version: &str, filename: &str) -> String {
                $crate::tool_utils::UrlUtils::github_release_url($owner, $repo, version, filename)
            }
        }

        impl Default for $tool_name {
            fn default() -> Self {
                Self::new()
            }
        }

        $crate::impl_tool_basics!(
            $tool_name,
            $name,
            $desc,
            vec![$($($alias),*)?]
        );
    };
}
