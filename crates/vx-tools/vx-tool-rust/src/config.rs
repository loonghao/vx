//! Rust installation configuration
//!
//! This module provides Rust-specific installation configuration,
//! including URL building, platform detection, and installation methods.

use std::path::PathBuf;
use vx_installer::{ArchiveFormat, InstallConfig, InstallMethod};
use vx_tool_standard::{StandardToolConfig, StandardUrlBuilder, ToolDependency};

/// Standard configuration for Rust tool
pub struct Config;

/// Rust URL builder for consistent download URL generation
#[derive(Debug, Clone)]
pub struct RustUrlBuilder;

impl Default for RustUrlBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl RustUrlBuilder {
    pub fn new() -> Self {
        Self
    }
}

impl StandardUrlBuilder for RustUrlBuilder {
    /// Generate download URL for Rust version
    fn download_url(version: &str) -> Option<String> {
        let filename = Self::get_filename(version);

        Some(format!(
            "https://forge.rust-lang.org/infra/channel-releases.html#{}",
            filename
        ))
    }

    /// Get platform-specific filename
    fn get_filename(version: &str) -> String {
        let platform = Self::get_platform_string();
        if cfg!(windows) {
            format!("rust-{}-{}.msi", version, platform)
        } else {
            format!("rust-{}-{}.tar.gz", version, platform)
        }
    }

    /// Get platform string for downloads
    fn get_platform_string() -> String {
        if cfg!(target_os = "windows") {
            if cfg!(target_arch = "x86_64") {
                "x86_64-pc-windows-msvc".to_string()
            } else if cfg!(target_arch = "x86") {
                "i686-pc-windows-msvc".to_string()
            } else {
                "unknown-pc-windows-msvc".to_string()
            }
        } else if cfg!(target_os = "macos") {
            if cfg!(target_arch = "x86_64") {
                "x86_64-apple-darwin".to_string()
            } else if cfg!(target_arch = "aarch64") {
                "aarch64-apple-darwin".to_string()
            } else {
                "unknown-apple-darwin".to_string()
            }
        } else if cfg!(target_os = "linux") {
            if cfg!(target_arch = "x86_64") {
                "x86_64-unknown-linux-gnu".to_string()
            } else if cfg!(target_arch = "aarch64") {
                "aarch64-unknown-linux-gnu".to_string()
            } else {
                "unknown-unknown-linux-gnu".to_string()
            }
        } else {
            "unknown".to_string()
        }
    }
}

impl StandardToolConfig for Config {
    fn tool_name() -> &'static str {
        "rust"
    }

    fn create_install_config(version: &str, install_dir: PathBuf) -> InstallConfig {
        create_install_config(version, install_dir)
    }

    fn get_install_methods() -> Vec<String> {
        vec!["archive".to_string(), "installer".to_string()]
    }

    fn supports_auto_install() -> bool {
        true
    }

    fn get_manual_instructions() -> String {
        "Visit https://rustup.rs/ to install Rust manually".to_string()
    }

    fn get_dependencies() -> Vec<ToolDependency> {
        vec![]
    }

    fn get_default_version() -> &'static str {
        "stable"
    }
}

/// Create installation configuration for Rust
pub fn create_install_config(version: &str, install_dir: PathBuf) -> InstallConfig {
    let download_url = RustUrlBuilder::download_url(version);

    InstallConfig::builder()
        .tool_name(Config::tool_name())
        .version(version)
        .download_url(download_url.unwrap_or_default())
        .install_method(InstallMethod::Archive {
            format: if cfg!(windows) {
                ArchiveFormat::Zip
            } else {
                ArchiveFormat::TarGz
            },
        })
        .install_dir(install_dir)
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rust_url_builder() {
        let url = RustUrlBuilder::download_url("1.75.0");
        assert!(url.is_some());
        assert!(url.unwrap().contains("rust-lang.org"));
    }

    #[test]
    fn test_platform_string() {
        let platform = RustUrlBuilder::get_platform_string();
        assert!(!platform.is_empty());
    }

    #[test]
    fn test_create_install_config() {
        let config = create_install_config("1.75.0", PathBuf::from("/tmp/rust"));
        assert_eq!(config.tool_name, "rust");
        assert_eq!(config.version, "1.75.0");
        assert!(config.download_url.is_some());
    }
}
