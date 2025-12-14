//! Node.js installation configuration
//!
//! This module provides Node.js-specific installation configuration,
//! including URL building, platform detection, and installation methods.

use std::path::PathBuf;
use vx_installer::{ArchiveFormat, InstallConfig, InstallMethod};

/// Node.js URL builder for consistent download URL generation
pub struct NodeUrlBuilder;

impl NodeUrlBuilder {
    /// Generate download URL for Node.js version
    pub fn download_url(version: &str) -> Option<String> {
        let filename = Self::get_filename(version);
        Some(format!("https://nodejs.org/dist/v{}/{}", version, filename))
    }

    /// Get platform-specific filename
    pub fn get_filename(version: &str) -> String {
        let platform = Self::get_platform_string();
        if cfg!(windows) {
            format!("node-v{}-{}.zip", version, platform)
        } else {
            format!("node-v{}-{}.tar.gz", version, platform)
        }
    }

    /// Get platform string for Node.js downloads
    pub fn get_platform_string() -> String {
        match (std::env::consts::OS, std::env::consts::ARCH) {
            ("windows", "x86_64") => "win-x64".to_string(),
            ("windows", "x86") => "win-x86".to_string(),
            ("windows", "aarch64") => "win-arm64".to_string(),
            ("macos", "x86_64") => "darwin-x64".to_string(),
            ("macos", "aarch64") => "darwin-arm64".to_string(),
            ("linux", "x86_64") => "linux-x64".to_string(),
            ("linux", "x86") => "linux-x86".to_string(),
            ("linux", "aarch64") => "linux-arm64".to_string(),
            ("linux", "arm") => "linux-armv7l".to_string(),
            _ => "linux-x64".to_string(), // Default fallback
        }
    }
}

/// Create Node.js installation configuration
pub fn create_install_config(version: &str, install_dir: PathBuf) -> InstallConfig {
    let actual_version = if version == "latest" {
        "20.11.0" // Default to LTS version
    } else {
        version
    };

    let download_url = NodeUrlBuilder::download_url(actual_version);
    let install_method = InstallMethod::Archive {
        format: if cfg!(windows) {
            ArchiveFormat::Zip
        } else {
            ArchiveFormat::TarGz
        },
    };

    InstallConfig::builder()
        .tool_name("node")
        .version(version.to_string())
        .install_method(install_method)
        .download_url(download_url.unwrap_or_default())
        .install_dir(install_dir)
        .build()
}

/// Get available Node.js installation methods
pub fn get_install_methods() -> Vec<String> {
    vec![
        "Official releases".to_string(),
        if cfg!(target_os = "macos") {
            "Homebrew".to_string()
        } else {
            "Package manager".to_string()
        },
        "Node Version Manager (nvm)".to_string(),
    ]
}

/// Check if Node.js supports automatic installation
pub fn supports_auto_install() -> bool {
    true
}

/// Get manual installation instructions for Node.js
pub fn get_manual_instructions() -> String {
    "To install Node.js manually:\n\
     • Visit: https://nodejs.org/\n\
     • Or use a version manager like nvm"
        .to_string()
}
