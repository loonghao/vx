//! Bun runtime configuration
//!
//! This module provides Bun-specific configuration,
//! including dependencies and installation methods.

use std::path::PathBuf;
use vx_installer::{ArchiveFormat, InstallConfig, InstallMethod};
use vx_tool_standard::{StandardToolConfig, StandardUrlBuilder, ToolDependency};

/// Standard configuration for Bun tool
pub struct Config;

/// Bun URL builder for consistent download URL generation
pub struct BunUrlBuilder;

impl StandardUrlBuilder for BunUrlBuilder {
    /// Generate download URL for Bun version
    fn download_url(version: &str) -> Option<String> {
        let filename = Self::get_filename(version);
        Some(format!(
            "https://github.com/oven-sh/bun/releases/download/bun-v{}/{}",
            version, filename
        ))
    }

    /// Get platform-specific filename
    fn get_filename(_version: &str) -> String {
        let platform = Self::get_platform_string();
        format!("bun-{}.zip", platform)
    }

    /// Get platform string for Bun downloads
    fn get_platform_string() -> String {
        match (std::env::consts::OS, std::env::consts::ARCH) {
            ("windows", "x86_64") => "windows-x64".to_string(),
            ("windows", "aarch64") => "windows-aarch64".to_string(),
            ("macos", "x86_64") => "darwin-x64".to_string(),
            ("macos", "aarch64") => "darwin-aarch64".to_string(),
            ("linux", "x86_64") => "linux-x64".to_string(),
            ("linux", "aarch64") => "linux-aarch64".to_string(),
            _ => "linux-x64".to_string(), // Default fallback
        }
    }
}

/// Implementation of standard tool configuration for Bun
impl StandardToolConfig for Config {
    fn tool_name() -> &'static str {
        "bun"
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
        // Bun is a standalone runtime with no dependencies
        vec![]
    }

    fn get_default_version() -> &'static str {
        "1.0.0" // Latest stable version
    }
}

/// Create Bun installation configuration
pub fn create_install_config(version: &str, install_dir: PathBuf) -> InstallConfig {
    let actual_version = if version == "latest" {
        "1.0.0" // Default to stable version
    } else {
        version
    };

    let download_url = BunUrlBuilder::download_url(actual_version);
    let install_method = InstallMethod::Archive {
        format: ArchiveFormat::Zip,
    };

    InstallConfig::builder()
        .tool_name("bun")
        .version(version.to_string())
        .install_method(install_method)
        .download_url(download_url.unwrap_or_default())
        .install_dir(install_dir)
        .build()
}

/// Get available Bun installation methods
pub fn get_install_methods() -> Vec<String> {
    vec![
        "GitHub releases (recommended)".to_string(),
        "NPM global install".to_string(),
        if cfg!(target_os = "macos") {
            "Homebrew".to_string()
        } else {
            "Package manager".to_string()
        },
    ]
}

/// Get manual installation instructions for Bun
pub fn get_manual_instructions() -> String {
    "To install Bun manually:\n\
     • curl -fsSL https://bun.sh/install | bash\n\
     • Or visit: https://bun.sh/docs/installation\n\
     • No dependencies required"
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bun_dependencies() {
        let deps = Config::get_dependencies();
        assert_eq!(deps.len(), 0); // Bun has no dependencies
    }

    #[test]
    fn test_bun_config() {
        assert_eq!(Config::tool_name(), "bun");
        assert!(Config::supports_auto_install());
        assert_eq!(Config::get_default_version(), "1.0.0");
    }

    #[test]
    fn test_bun_url_builder() {
        let url = BunUrlBuilder::download_url("1.0.0");
        assert!(url.is_some());
        assert!(url.unwrap().contains("github.com/oven-sh/bun"));
    }

    #[test]
    fn test_create_install_config() {
        let config = create_install_config("latest", PathBuf::from("/tmp/bun"));
        assert_eq!(config.tool_name, "bun");
        assert_eq!(config.version, "latest");
    }
}
