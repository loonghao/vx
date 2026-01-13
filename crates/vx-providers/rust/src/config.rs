//! Rust configuration and URL builder
//!
//! Downloads Rust toolchains from static.rust-lang.org

use vx_runtime::{Arch, Os, Platform};

/// Rust URL builder for download URLs
pub struct RustUrlBuilder;

impl RustUrlBuilder {
    /// Generate download URL for Rust version
    ///
    /// Uses `.tar.gz` format for all platforms (including Windows) as it's
    /// universally supported by our archive extraction logic.
    pub fn download_url(version: &str, platform: &Platform) -> Option<String> {
        let platform_str = Self::get_platform_string(platform);
        Some(format!(
            "https://static.rust-lang.org/dist/rust-{}-{}.tar.gz",
            version, platform_str
        ))
    }

    /// Get platform string for downloads
    pub fn get_platform_string(platform: &Platform) -> String {
        match (&platform.os, &platform.arch) {
            // Windows
            (Os::Windows, Arch::X86_64) => "x86_64-pc-windows-msvc".to_string(),
            (Os::Windows, Arch::X86) => "i686-pc-windows-msvc".to_string(),
            (Os::Windows, Arch::Aarch64) => "aarch64-pc-windows-msvc".to_string(),

            // macOS
            (Os::MacOS, Arch::X86_64) => "x86_64-apple-darwin".to_string(),
            (Os::MacOS, Arch::Aarch64) => "aarch64-apple-darwin".to_string(),

            // Linux
            (Os::Linux, Arch::X86_64) => "x86_64-unknown-linux-gnu".to_string(),
            (Os::Linux, Arch::Aarch64) => "aarch64-unknown-linux-gnu".to_string(),
            (Os::Linux, Arch::Arm) => "arm-unknown-linux-gnueabihf".to_string(),

            // Default fallback
            _ => "x86_64-unknown-linux-gnu".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_download_url_linux_x64() {
        let platform = Platform {
            os: Os::Linux,
            arch: Arch::X86_64,
        };
        let url = RustUrlBuilder::download_url("1.75.0", &platform);
        assert_eq!(
            url,
            Some(
                "https://static.rust-lang.org/dist/rust-1.75.0-x86_64-unknown-linux-gnu.tar.gz"
                    .to_string()
            )
        );
    }

    #[test]
    fn test_download_url_windows_x64() {
        let platform = Platform {
            os: Os::Windows,
            arch: Arch::X86_64,
        };
        let url = RustUrlBuilder::download_url("1.75.0", &platform);
        assert_eq!(
            url,
            Some(
                "https://static.rust-lang.org/dist/rust-1.75.0-x86_64-pc-windows-msvc.tar.gz"
                    .to_string()
            )
        );
    }

    #[test]
    fn test_download_url_macos_arm64() {
        let platform = Platform {
            os: Os::MacOS,
            arch: Arch::Aarch64,
        };
        let url = RustUrlBuilder::download_url("1.75.0", &platform);
        assert_eq!(
            url,
            Some(
                "https://static.rust-lang.org/dist/rust-1.75.0-aarch64-apple-darwin.tar.gz"
                    .to_string()
            )
        );
    }

    #[test]
    fn test_platform_string() {
        let platform = Platform {
            os: Os::Linux,
            arch: Arch::X86_64,
        };
        assert_eq!(
            RustUrlBuilder::get_platform_string(&platform),
            "x86_64-unknown-linux-gnu"
        );
    }
}
