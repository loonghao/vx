//! PNPM package manager configuration
//!
//! This module provides PNPM-specific configuration,
//! including dependencies and installation methods.

use std::path::PathBuf;
use vx_installer::{ArchiveFormat, InstallConfig, InstallMethod};
use vx_tool_standard::{StandardToolConfig, StandardUrlBuilder, ToolDependency};

/// Standard configuration for PNPM tool
pub struct Config;

/// PNPM URL builder for consistent download URL generation
pub struct PnpmUrlBuilder;

impl StandardUrlBuilder for PnpmUrlBuilder {
    /// Generate download URL for PNPM version
    fn download_url(version: &str) -> Option<String> {
        let platform = Self::get_platform_string();
        let filename = Self::get_filename(version);

        Some(format!(
            "https://github.com/pnpm/pnpm/releases/download/v{}/{}",
            version, filename
        ))
    }

    /// Get platform-specific filename
    fn get_filename(version: &str) -> String {
        let platform = Self::get_platform_string();
        if cfg!(windows) {
            format!("pnpm-{}-{}.exe", platform, version)
        } else {
            format!("pnpm-{}-{}", platform, version)
        }
    }

    /// Get platform string for PNPM downloads
    fn get_platform_string() -> String {
        match (std::env::consts::OS, std::env::consts::ARCH) {
            ("windows", "x86_64") => "win-x64".to_string(),
            ("windows", "aarch64") => "win-arm64".to_string(),
            ("macos", "x86_64") => "macos-x64".to_string(),
            ("macos", "aarch64") => "macos-arm64".to_string(),
            ("linux", "x86_64") => "linux-x64".to_string(),
            ("linux", "aarch64") => "linux-arm64".to_string(),
            _ => "linux-x64".to_string(), // Default fallback
        }
    }
}

/// Implementation of standard tool configuration for PNPM
impl StandardToolConfig for Config {
    fn tool_name() -> &'static str {
        "pnpm"
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
        vec![
            ToolDependency::required("node", "PNPM requires Node.js runtime")
                .with_version(">=16.14.0"),
        ]
    }

    fn get_default_version() -> &'static str {
        "latest"
    }
}

/// Create PNPM installation configuration
pub fn create_install_config(version: &str, install_dir: PathBuf) -> InstallConfig {
    let actual_version = if version == "latest" {
        "8.15.0" // Default to stable version
    } else {
        version
    };

    let download_url = PnpmUrlBuilder::download_url(actual_version);
    let install_method = if cfg!(windows) {
        InstallMethod::Binary
    } else {
        InstallMethod::Binary
    };

    InstallConfig::builder()
        .tool_name("pnpm")
        .version(version.to_string())
        .install_method(install_method)
        .download_url(download_url.unwrap_or_default())
        .install_dir(install_dir)
        .build()
}

/// Get available PNPM installation methods
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

/// Check if PNPM supports automatic installation
pub fn supports_auto_install() -> bool {
    true
}

/// Get manual installation instructions for PNPM
pub fn get_manual_instructions() -> String {
    "To install PNPM manually:\n\
     • npm install -g pnpm\n\
     • Or visit: https://pnpm.io/installation\n\
     • Requires Node.js >=16.14.0"
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pnpm_dependencies() {
        let deps = Config::get_dependencies();
        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].tool_name, "node");
        assert!(deps[0].required);
        assert_eq!(deps[0].version_requirement, Some(">=16.14.0".to_string()));
    }

    #[test]
    fn test_pnpm_config() {
        assert_eq!(Config::tool_name(), "pnpm");
        assert!(Config::supports_auto_install());
        assert_eq!(Config::get_default_version(), "latest");
    }

    #[test]
    fn test_pnpm_url_builder() {
        let url = PnpmUrlBuilder::download_url("8.15.0");
        assert!(url.is_some());
        assert!(url.unwrap().contains("github.com/pnpm/pnpm"));
    }

    #[test]
    fn test_create_install_config() {
        let config = create_install_config("latest", PathBuf::from("/tmp/pnpm"));
        assert_eq!(config.tool_name, "pnpm");
        assert_eq!(config.version, "latest");
    }
}
