//! PNPM configuration

use serde::{Deserialize, Serialize};
use vx_runtime::{Arch, Os, Platform};

/// PNPM configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PnpmConfig {
    /// Default PNPM version
    pub default_version: Option<String>,
    /// Store directory
    pub store_dir: Option<String>,
    /// Virtual store directory
    pub virtual_store_dir: Option<String>,
}

/// PNPM URL builder for download URLs
pub struct PnpmUrlBuilder;

impl PnpmUrlBuilder {
    /// Generate download URL for PNPM version
    /// PNPM releases are named without version in filename:
    /// e.g., https://github.com/pnpm/pnpm/releases/download/v9.0.0/pnpm-macos-arm64
    pub fn download_url(version: &str, platform: &Platform) -> Option<String> {
        let filename = Self::get_filename(platform);
        Some(format!(
            "https://github.com/pnpm/pnpm/releases/download/v{}/{}",
            version, filename
        ))
    }

    /// Get platform-specific filename (without version number)
    pub fn get_filename(platform: &Platform) -> String {
        let platform_str = Self::get_platform_string(platform);
        if platform.os == Os::Windows {
            format!("pnpm-{}.exe", platform_str)
        } else {
            format!("pnpm-{}", platform_str)
        }
    }

    /// Get platform string for PNPM downloads
    pub fn get_platform_string(platform: &Platform) -> &'static str {
        match (&platform.os, &platform.arch) {
            (Os::Windows, Arch::X86_64) => "win-x64",
            (Os::Windows, Arch::Aarch64) => "win-arm64",
            (Os::MacOS, Arch::X86_64) => "macos-x64",
            (Os::MacOS, Arch::Aarch64) => "macos-arm64",
            (Os::Linux, Arch::X86_64) => "linux-x64",
            (Os::Linux, Arch::Aarch64) => "linux-arm64",
            _ => "linux-x64",
        }
    }
}
