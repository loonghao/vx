//! Go installation configuration
//!
//! This module provides Go-specific installation configuration,
//! including URL building, platform detection, and installation methods.

use std::path::PathBuf;
use vx_installer::{ArchiveFormat, InstallConfig, InstallMethod};

/// Go URL builder for consistent download URL generation
pub struct GoUrlBuilder;

impl GoUrlBuilder {
    /// Generate download URL for Go version
    pub fn download_url(version: &str) -> Option<String> {
        let filename = Self::get_filename(version);
        Some(format!("https://golang.org/dl/{}", filename))
    }

    /// Get platform-specific filename
    pub fn get_filename(version: &str) -> String {
        let platform = Self::get_platform_string();
        if cfg!(windows) {
            format!("go{}.{}.zip", version, platform)
        } else {
            format!("go{}.{}.tar.gz", version, platform)
        }
    }

    /// Get platform string for Go downloads
    pub fn get_platform_string() -> String {
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

/// Create Go installation configuration
#[allow(dead_code)]
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

    InstallConfig::builder()
        .tool_name("go")
        .version(version.to_string())
        .install_method(install_method)
        .download_url(download_url.unwrap_or_default())
        .install_dir(install_dir)
        .build()
}

/// Get available Go installation methods
#[allow(dead_code)]
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
#[allow(dead_code)]
pub fn get_manual_instructions() -> String {
    "To install Go manually:\n\
     • Visit: https://golang.org/dl/\n\
     • Or use your system package manager"
        .to_string()
}
