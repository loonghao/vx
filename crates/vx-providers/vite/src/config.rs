//! URL builder and platform configuration for Vite
//!
//! Vite standalone releases are available at:
//! <https://github.com/nicholasruunu/vite-standalone/releases>

use vx_runtime::{Arch, Os, Platform};

/// URL builder for Vite downloads
pub struct ViteUrlBuilder;

impl ViteUrlBuilder {
    /// Base URL for Vite standalone releases
    const BASE_URL: &'static str =
        "https://github.com/nicholasruunu/vite-standalone/releases/download";

    /// Build the download URL for a specific version and platform
    pub fn download_url(version: &str, platform: &Platform) -> Option<String> {
        let target = Self::get_target_triple(platform)?;
        let ext = Self::get_archive_extension(platform);
        Some(format!(
            "{}/v{}/vite-{}.{}",
            Self::BASE_URL,
            version,
            target,
            ext
        ))
    }

    /// Get the target triple for the platform
    pub fn get_target_triple(platform: &Platform) -> Option<String> {
        match (&platform.os, &platform.arch) {
            // Windows
            (Os::Windows, Arch::X86_64) => Some("x86_64-pc-windows-msvc".to_string()),
            (Os::Windows, Arch::Aarch64) => Some("aarch64-pc-windows-msvc".to_string()),

            // macOS
            (Os::MacOS, Arch::X86_64) => Some("x86_64-apple-darwin".to_string()),
            (Os::MacOS, Arch::Aarch64) => Some("aarch64-apple-darwin".to_string()),

            // Linux
            (Os::Linux, Arch::X86_64) => Some("x86_64-unknown-linux-musl".to_string()),
            (Os::Linux, Arch::Aarch64) => Some("aarch64-unknown-linux-musl".to_string()),

            _ => None,
        }
    }

    /// Get the archive extension for the platform
    pub fn get_archive_extension(platform: &Platform) -> &'static str {
        match platform.os {
            Os::Windows => "zip",
            _ => "tar.gz",
        }
    }

    /// Get the executable name for the platform
    pub fn get_executable_name(platform: &Platform) -> &'static str {
        match platform.os {
            Os::Windows => "vite.exe",
            _ => "vite",
        }
    }
}
