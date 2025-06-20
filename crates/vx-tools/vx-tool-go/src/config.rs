//! Go installation configuration
//!
//! This module provides Go-specific installation configuration,
//! including URL building, platform detection, and installation methods.

use std::path::PathBuf;
use vx_installer::{ArchiveFormat, InstallConfig, InstallMethod, LifecycleAction, LifecycleHooks};
use vx_tool_standard::{StandardToolConfig, StandardUrlBuilder, ToolDependency};

/// Standard configuration for Go tool
#[allow(dead_code)]
pub struct Config;

/// Go URL builder for consistent download URL generation
pub struct GoUrlBuilder;

impl StandardUrlBuilder for GoUrlBuilder {
    /// Generate download URL for Go version
    fn download_url(version: &str) -> Option<String> {
        let filename = Self::get_filename(version);

        Some(format!("https://golang.org/dl/{}", filename))
    }

    /// Get platform-specific filename
    fn get_filename(version: &str) -> String {
        let platform = Self::get_platform_string();
        if cfg!(windows) {
            format!("go{}.{}.zip", version, platform)
        } else {
            format!("go{}.{}.tar.gz", version, platform)
        }
    }

    /// Get platform string for Go downloads
    fn get_platform_string() -> String {
        match (std::env::consts::OS, std::env::consts::ARCH) {
            ("windows", "x86_64") => "windows-amd64".to_string(),
            ("windows", "x86") => "windows-386".to_string(),
            ("windows", "aarch64") => "windows-arm64".to_string(),
            ("macos", "x86_64") => "darwin-amd64".to_string(),
            ("macos", "aarch64") => "darwin-arm64".to_string(),
            ("linux", "x86_64") => "linux-amd64".to_string(),
            ("linux", "x86") => "linux-386".to_string(),
            ("linux", "aarch64") => "linux-arm64".to_string(),
            ("linux", "arm") => "linux-armv6l".to_string(),
            _ => "linux-amd64".to_string(), // Default fallback
        }
    }
}

/// Implementation of standard tool configuration for Go
impl StandardToolConfig for Config {
    fn tool_name() -> &'static str {
        "go"
    }

    fn create_install_config(version: &str, install_dir: PathBuf) -> InstallConfig {
        create_install_config(version, install_dir)
    }

    fn get_install_methods() -> Vec<String> {
        get_install_methods()
    }

    fn supports_auto_install() -> bool {
        true
    }

    fn get_manual_instructions() -> String {
        get_manual_instructions()
    }

    fn get_dependencies() -> Vec<ToolDependency> {
        // Go has no dependencies - it's a standalone runtime
        vec![]
    }

    fn get_default_version() -> &'static str {
        "1.21.6" // Stable version
    }
}

/// Create Go installation configuration
pub fn create_install_config(version: &str, install_dir: PathBuf) -> InstallConfig {
    let actual_version = if version == "latest" {
        "1.21.6" // Default to stable version
    } else {
        version
    };

    let download_url = GoUrlBuilder::download_url(actual_version);
    let install_method = InstallMethod::Archive {
        format: if cfg!(windows) {
            ArchiveFormat::Zip
        } else {
            ArchiveFormat::TarGz
        },
    };

    // Create lifecycle hooks to optimize path structure
    let mut hooks = LifecycleHooks::default();

    // Go archives extract to go/ subdirectory
    // We want to move everything from go/ to the root install directory
    hooks.post_install.push(LifecycleAction::FlattenDirectory {
        source_pattern: "go".to_string(),
    });

    InstallConfig::builder()
        .tool_name("go")
        .version(version.to_string())
        .install_method(install_method)
        .download_url(download_url.unwrap_or_default())
        .install_dir(install_dir)
        .lifecycle_hooks(hooks)
        .build()
}

/// Get available Go installation methods
pub fn get_install_methods() -> Vec<String> {
    vec![
        "Official releases".to_string(),
        if cfg!(target_os = "macos") {
            "Homebrew".to_string()
        } else {
            "Package manager".to_string()
        },
    ]
}

/// Check if Go supports automatic installation
#[allow(dead_code)]
pub fn supports_auto_install() -> bool {
    true
}

/// Get manual installation instructions for Go
pub fn get_manual_instructions() -> String {
    "To install Go manually:\n\
     • Visit: https://golang.org/dl/\n\
     • Or use your system package manager"
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_go_url_builder() {
        let url = GoUrlBuilder::download_url("1.21.6");
        assert!(url.is_some());
        assert!(url.unwrap().contains("golang.org"));
    }

    #[test]
    fn test_platform_string() {
        let platform = GoUrlBuilder::get_platform_string();
        assert!(!platform.is_empty());
    }

    #[test]
    fn test_create_install_config() {
        let config = create_install_config("1.21.6", PathBuf::from("/tmp/go"));
        assert_eq!(config.tool_name, "go");
        assert_eq!(config.version, "1.21.6");
        assert!(config.download_url.is_some());
    }

    #[test]
    fn test_latest_version_handling() {
        let config = create_install_config("latest", PathBuf::from("/tmp/go"));
        assert_eq!(config.version, "latest");
        // Should use actual version in URL
        assert!(config.download_url.unwrap().contains("1.21.6"));
    }
}
