//! # vx-tool-standard
//!
//! Standard interfaces and utilities for implementing vx tools.
//!
//! This crate provides standardized traits and types that tool implementations
//! should use to ensure consistency across the vx ecosystem.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use vx_core::{Platform, VxResult};
use vx_installer::InstallConfig;

/// Standard tool configuration interface
pub trait StandardToolConfig {
    /// Get the tool name
    fn tool_name() -> &'static str;

    /// Create installation configuration for a version
    fn create_install_config(version: &str, install_dir: PathBuf) -> InstallConfig;

    /// Get available installation methods
    fn get_install_methods() -> Vec<String>;

    /// Check if the tool supports automatic installation
    fn supports_auto_install() -> bool;

    /// Get manual installation instructions
    fn get_manual_instructions() -> String;

    /// Get tool dependencies
    fn get_dependencies() -> Vec<ToolDependency>;

    /// Get default version
    fn get_default_version() -> &'static str;
}

/// Standard URL builder interface
pub trait StandardUrlBuilder {
    /// Generate download URL for a version
    fn download_url(version: &str) -> Option<String>;

    /// Get platform-specific filename
    fn get_filename(version: &str) -> String;

    /// Get platform string for downloads
    fn get_platform_string() -> String;
}

/// Tool dependency specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDependency {
    /// Name of the dependency tool
    pub tool_name: String,
    /// Human-readable description
    pub description: String,
    /// Whether this dependency is required
    pub required: bool,
    /// Version requirement (e.g., ">=16.0.0")
    pub version_requirement: Option<String>,
    /// Platforms this dependency applies to
    pub platforms: Vec<Platform>,
}

impl ToolDependency {
    /// Create a required dependency
    pub fn required(tool_name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            tool_name: tool_name.into(),
            description: description.into(),
            required: true,
            version_requirement: None,
            platforms: vec![],
        }
    }

    /// Create an optional dependency
    pub fn optional(tool_name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            tool_name: tool_name.into(),
            description: description.into(),
            required: false,
            version_requirement: None,
            platforms: vec![],
        }
    }

    /// Set version requirement
    pub fn with_version(mut self, requirement: impl Into<String>) -> Self {
        self.version_requirement = Some(requirement.into());
        self
    }

    /// Set platform constraints
    pub fn for_platforms(mut self, platforms: Vec<Platform>) -> Self {
        self.platforms = platforms;
        self
    }
}

/// Standard tool runtime interface
pub trait ToolRuntime {
    /// Check if the tool is available
    fn is_available(&self) -> impl std::future::Future<Output = VxResult<bool>> + Send;

    /// Get installed version
    fn get_version(&self) -> impl std::future::Future<Output = VxResult<Option<String>>> + Send;

    /// Get tool installation path
    fn get_path(&self) -> impl std::future::Future<Output = VxResult<Option<PathBuf>>> + Send;

    /// Execute the tool with arguments
    fn execute(&self, args: &[String]) -> impl std::future::Future<Output = VxResult<i32>> + Send;
}

/// Version parser interface
pub trait VersionParser {
    /// Parse version from tool output
    fn parse_version(output: &str) -> Option<String>;

    /// Validate version format
    fn is_valid_version(version: &str) -> bool;

    /// Compare two versions
    fn compare_versions(a: &str, b: &str) -> std::cmp::Ordering;
}

/// Platform-specific URL builder utilities
pub struct PlatformUrlBuilder;

impl PlatformUrlBuilder {
    /// Get standard platform string for downloads
    pub fn get_platform_string() -> String {
        let platform = Platform::current();
        platform.to_string()
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
}

/// Version utilities
pub struct VersionUtils;

impl VersionUtils {
    /// Check if version is "latest"
    pub fn is_latest(version: &str) -> bool {
        version == "latest" || version == "stable"
    }

    /// Normalize version string
    pub fn normalize_version(version: &str) -> String {
        // Remove 'v' prefix if present
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_dependency_creation() {
        let dep = ToolDependency::required("node", "Node.js runtime").with_version(">=16.0.0");

        assert_eq!(dep.tool_name, "node");
        assert!(dep.required);
        assert_eq!(dep.version_requirement, Some(">=16.0.0".to_string()));
    }

    #[test]
    fn test_platform_url_builder() {
        let platform = PlatformUrlBuilder::get_platform_string();
        assert!(!platform.is_empty());

        let ext = PlatformUrlBuilder::get_archive_extension();
        assert!(ext == "zip" || ext == "tar.gz");
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
    fn test_version_utils() {
        assert!(VersionUtils::is_latest("latest"));
        assert!(VersionUtils::is_latest("stable"));
        assert!(!VersionUtils::is_latest("1.0.0"));

        assert_eq!(VersionUtils::normalize_version("v1.0.0"), "1.0.0");
        assert_eq!(VersionUtils::normalize_version("1.0.0"), "1.0.0");

        assert!(VersionUtils::is_prerelease("1.0.0-beta.1"));
        assert!(!VersionUtils::is_prerelease("1.0.0"));
    }
}
