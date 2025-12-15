//! UV configuration and URL building
//!
//! This module provides UV-specific configuration,
//! including URL building and platform detection.

use vx_runtime::Platform;

/// UV URL builder for consistent download URL generation
#[derive(Debug, Clone, Default)]
pub struct UvUrlBuilder;

impl UvUrlBuilder {
    pub fn new() -> Self {
        Self
    }

    /// Generate download URL for UV version
    pub fn download_url(version: &str, platform: &Platform) -> Option<String> {
        let filename = Self::get_filename(version, platform);
        Some(format!(
            "https://github.com/astral-sh/uv/releases/download/{}/{}",
            version, filename
        ))
    }

    /// Get platform-specific filename
    pub fn get_filename(version: &str, platform: &Platform) -> String {
        let platform_str = Self::get_platform_string(platform);
        if platform.os == vx_runtime::Os::Windows {
            format!("uv-{}-{}.zip", platform_str, version)
        } else {
            format!("uv-{}-{}.tar.gz", platform_str, version)
        }
    }

    /// Get platform string for downloads
    pub fn get_platform_string(platform: &Platform) -> String {
        use vx_runtime::{Arch, Os};

        match (&platform.os, &platform.arch) {
            (Os::Windows, Arch::X86_64) => "x86_64-pc-windows-msvc".to_string(),
            (Os::Windows, Arch::X86) => "i686-pc-windows-msvc".to_string(),
            (Os::MacOS, Arch::X86_64) => "x86_64-apple-darwin".to_string(),
            (Os::MacOS, Arch::Aarch64) => "aarch64-apple-darwin".to_string(),
            (Os::Linux, Arch::X86_64) => "x86_64-unknown-linux-gnu".to_string(),
            (Os::Linux, Arch::Aarch64) => "aarch64-unknown-linux-gnu".to_string(),
            _ => "x86_64-unknown-linux-gnu".to_string(), // Default fallback
        }
    }
}
