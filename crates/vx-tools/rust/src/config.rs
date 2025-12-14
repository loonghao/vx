//! Rust installation configuration
//!
//! This module provides Rust-specific installation configuration,
//! including URL building, platform detection, and installation methods.

use std::path::PathBuf;
use vx_installer::{ArchiveFormat, InstallConfig, InstallMethod};

/// Rust URL builder for consistent download URL generation
#[derive(Debug, Clone, Default)]
pub struct RustUrlBuilder;

impl RustUrlBuilder {
    pub fn new() -> Self {
        Self
    }

    /// Generate download URL for Rust version
    pub fn download_url(version: &str) -> Option<String> {
        let filename = Self::get_filename(version);
        Some(format!(
            "https://forge.rust-lang.org/infra/channel-releases.html#{}",
            filename
        ))
    }

    /// Get platform-specific filename
    pub fn get_filename(version: &str) -> String {
        let platform = Self::get_platform_string();
        if cfg!(windows) {
            format!("rust-{}-{}.msi", version, platform)
        } else {
            format!("rust-{}-{}.tar.gz", version, platform)
        }
    }

    /// Get platform string for downloads
    pub fn get_platform_string() -> String {
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

/// Create installation configuration for Rust
#[allow(dead_code)]
pub fn create_install_config(version: &str, install_dir: PathBuf) -> InstallConfig {
    let download_url = RustUrlBuilder::download_url(version);

    InstallConfig::builder()
        .tool_name("rust")
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
