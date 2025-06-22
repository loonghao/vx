//! UV installation configuration
//!
//! This module provides UV-specific installation configuration,
//! including URL building, platform detection, and installation methods.

use std::path::PathBuf;
use vx_installer::{ArchiveFormat, InstallConfig, InstallMethod, LifecycleAction, LifecycleHooks};
use vx_tool_standard::{StandardToolConfig, StandardUrlBuilder, ToolDependency};

/// Standard configuration for UV tool
pub struct Config;

/// UV URL builder for consistent download URL generation
#[derive(Debug, Clone)]
pub struct UvUrlBuilder;

impl Default for UvUrlBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl UvUrlBuilder {
    pub fn new() -> Self {
        Self
    }
}

impl StandardUrlBuilder for UvUrlBuilder {
    /// Generate download URL for UV version - USES GLOBAL CONFIG
    fn download_url(version: &str) -> Option<String> {
        // IMPORTANT: This method should NOT receive "latest" as version
        // The caller should resolve "latest" to actual version first
        if version == "latest" {
            eprintln!(
                "Warning: download_url received 'latest' version, this should be resolved first"
            );
            return None;
        }

        // Use global config system for sync access
        use vx_config::{get_platform_string, get_tool_download_url_sync};

        // Try to get URL from config first
        if let Some(url) = get_tool_download_url_sync("uv", version) {
            return Some(url);
        }

        // Fallback to hardcoded logic with actual version
        let platform = get_platform_string();
        let filename = if cfg!(windows) {
            format!("uv-{}.zip", platform)
        } else {
            format!("uv-{}.tar.gz", platform)
        };

        Some(format!(
            "https://github.com/astral-sh/uv/releases/download/{}/{}",
            version, filename
        ))
    }

    /// Get platform-specific filename - USES GLOBAL CONFIG
    fn get_filename(version: &str) -> String {
        // Use global config system for sync access
        use vx_config::{get_platform_string, get_tool_filename_sync};

        // Try to get filename from config first
        if let Some(filename) = get_tool_filename_sync("uv", version) {
            return filename;
        }

        // Fallback to hardcoded logic
        let platform = get_platform_string();
        if cfg!(windows) {
            format!("uv-{}.zip", platform)
        } else {
            format!("uv-{}.tar.gz", platform)
        }
    }

    /// Get platform string for downloads - DELEGATES TO GLOBAL CONFIG
    fn get_platform_string() -> String {
        // Delegate to global config system
        vx_config::get_platform_string()
    }
}

impl StandardToolConfig for Config {
    fn tool_name() -> &'static str {
        "uv"
    }

    fn create_install_config(version: &str, install_dir: PathBuf) -> InstallConfig {
        create_install_config(version, install_dir, false)
    }

    fn get_install_methods() -> Vec<String> {
        vec!["archive".to_string(), "binary".to_string()]
    }

    fn supports_auto_install() -> bool {
        true
    }

    fn get_manual_instructions() -> String {
        // Use global config system for sync access
        vx_config::get_tool_manual_instructions_sync("uv")
    }

    fn get_dependencies() -> Vec<ToolDependency> {
        vec![]
    }

    fn get_default_version() -> &'static str {
        "latest"
    }
}

/// Create installation configuration for UV
pub fn create_install_config(version: &str, install_dir: PathBuf, force: bool) -> InstallConfig {
    // Handle "latest" version by resolving to default version
    let actual_version = if version == "latest" {
        Config::get_default_version() // Use the default stable version
    } else {
        version
    };

    let download_url = UvUrlBuilder::download_url(actual_version);

    // Create lifecycle hooks to optimize path structure
    let mut hooks = LifecycleHooks::default();

    // UV archives may extract to subdirectories, flatten them
    // Also set executable permissions on Unix systems
    hooks.post_install.push(LifecycleAction::FlattenDirectory {
        source_pattern: "uv".to_string(),
    });

    #[cfg(unix)]
    hooks.post_install.push(LifecycleAction::SetExecutable {
        path: "uv".to_string(),
    });

    InstallConfig::builder()
        .tool_name(Config::tool_name())
        .version(actual_version)
        .download_url(download_url.unwrap_or_default())
        .install_method(InstallMethod::Archive {
            format: if cfg!(windows) {
                ArchiveFormat::Zip
            } else {
                ArchiveFormat::TarGz
            },
        })
        .install_dir(install_dir)
        .force(force)
        .lifecycle_hooks(hooks)
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uv_url_builder() {
        let url = UvUrlBuilder::download_url("0.1.0");
        assert!(url.is_some());
        assert!(url.expect("URL should be generated").contains("github.com"));
    }

    #[test]
    fn test_platform_string() {
        let platform = UvUrlBuilder::get_platform_string();
        assert!(!platform.is_empty());
    }

    #[test]
    fn test_create_install_config() {
        let config = create_install_config("0.1.0", PathBuf::from("/tmp/uv"), false);
        assert_eq!(config.tool_name, "uv");
        assert_eq!(config.version, "0.1.0");
        assert!(config.download_url.is_some());
        assert!(!config.force);
    }

    #[test]
    fn test_latest_version_handling() {
        // Test that create_install_config properly handles "latest" version
        let config = create_install_config("latest", PathBuf::from("/tmp/uv"), false);

        // Should resolve "latest" to the default version
        assert_eq!(config.version, Config::get_default_version());
        assert_eq!(config.tool_name, "uv");
        assert!(config.download_url.is_some());
        assert!(!config.force);
    }

    #[test]
    fn test_uv_url_format_fix() {
        // Test that the URL format is correct (no version in filename)
        let url = UvUrlBuilder::download_url("0.7.13");
        assert!(url.is_some());
        let url_str = url.expect("URL should be generated");

        // Should contain the version in the path but not in the filename
        assert!(url_str.contains("/0.7.13/"));

        // Should NOT contain version twice (in both path and filename)
        if cfg!(windows) {
            assert!(url_str.ends_with("uv-x86_64-pc-windows-msvc.zip"));
            assert!(!url_str.contains("0.7.13.zip"));
        } else {
            assert!(url_str.ends_with(".tar.gz"));
            assert!(!url_str.contains("0.7.13.tar.gz"));
        }
    }
}
