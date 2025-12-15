//! Go configuration and URL building
//!
//! This module provides Go-specific configuration,
//! including URL building and platform detection.

use vx_runtime::Platform;

/// Go URL builder for consistent download URL generation
pub struct GoUrlBuilder;

impl GoUrlBuilder {
    /// Generate download URL for Go version
    pub fn download_url(version: &str, platform: &Platform) -> Option<String> {
        let filename = Self::get_filename(version, platform);
        Some(format!("https://golang.org/dl/{}", filename))
    }

    /// Get platform-specific filename
    pub fn get_filename(version: &str, platform: &Platform) -> String {
        let platform_str = Self::get_platform_string(platform);
        if platform.os == vx_runtime::Os::Windows {
            format!("go{}.{}.zip", version, platform_str)
        } else {
            format!("go{}.{}.tar.gz", version, platform_str)
        }
    }

    /// Get platform string for Go downloads
    pub fn get_platform_string(platform: &Platform) -> String {
        use vx_runtime::{Arch, Os};

        match (&platform.os, &platform.arch) {
            (Os::Windows, Arch::X86_64) => "windows-amd64".to_string(),
            (Os::Windows, Arch::X86) => "windows-386".to_string(),
            (Os::Windows, Arch::Aarch64) => "windows-arm64".to_string(),
            (Os::MacOS, Arch::X86_64) => "darwin-amd64".to_string(),
            (Os::MacOS, Arch::Aarch64) => "darwin-arm64".to_string(),
            (Os::Linux, Arch::X86_64) => "linux-amd64".to_string(),
            (Os::Linux, Arch::X86) => "linux-386".to_string(),
            (Os::Linux, Arch::Aarch64) => "linux-arm64".to_string(),
            (Os::Linux, Arch::Arm) => "linux-armv6l".to_string(),
            _ => "linux-amd64".to_string(), // Default fallback
        }
    }
}
