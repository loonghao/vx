//! UV configuration and URL building
//!
//! This module provides UV-specific configuration,
//! including URL building and platform detection.

use vx_runtime::{Arch, Libc, Os, Platform};

/// UV URL builder for consistent download URL generation
#[derive(Debug, Clone, Default)]
pub struct UvUrlBuilder;

impl UvUrlBuilder {
    pub fn new() -> Self {
        Self
    }

    /// Generate download URL for UV version
    /// UV download URLs format: https://github.com/astral-sh/uv/releases/download/{version}/uv-{platform}.{ext}
    pub fn download_url(version: &str, platform: &Platform) -> Option<String> {
        let filename = Self::get_filename(platform)?;
        // Version in UV releases can be with or without 'v' prefix
        // The release tag uses the version as-is (e.g., "0.9.17" not "v0.9.17")
        let clean_version = version.trim_start_matches('v');
        Some(format!(
            "https://github.com/astral-sh/uv/releases/download/{}/{}",
            clean_version, filename
        ))
    }

    /// Get platform-specific filename (without version in filename)
    pub fn get_filename(platform: &Platform) -> Option<String> {
        let platform_str = Self::get_platform_string(platform)?;
        if platform.os == Os::Windows {
            Some(format!("uv-{}.zip", platform_str))
        } else {
            Some(format!("uv-{}.tar.gz", platform_str))
        }
    }

    /// Get platform string for downloads
    /// Supports all platforms that uv provides binaries for
    pub fn get_platform_string(platform: &Platform) -> Option<String> {
        match (&platform.os, &platform.arch, &platform.libc) {
            // Windows
            (Os::Windows, Arch::X86_64, _) => Some("x86_64-pc-windows-msvc".to_string()),
            (Os::Windows, Arch::X86, _) => Some("i686-pc-windows-msvc".to_string()),
            (Os::Windows, Arch::Aarch64, _) => Some("aarch64-pc-windows-msvc".to_string()),

            // macOS
            (Os::MacOS, Arch::X86_64, _) => Some("x86_64-apple-darwin".to_string()),
            (Os::MacOS, Arch::Aarch64, _) => Some("aarch64-apple-darwin".to_string()),

            // Linux GNU
            (Os::Linux, Arch::X86_64, Libc::Gnu) => Some("x86_64-unknown-linux-gnu".to_string()),
            (Os::Linux, Arch::X86, Libc::Gnu) => Some("i686-unknown-linux-gnu".to_string()),
            (Os::Linux, Arch::Aarch64, Libc::Gnu) => Some("aarch64-unknown-linux-gnu".to_string()),
            (Os::Linux, Arch::Armv7, Libc::Gnu) => {
                Some("armv7-unknown-linux-gnueabihf".to_string())
            }
            (Os::Linux, Arch::PowerPC64, Libc::Gnu) => {
                Some("powerpc64-unknown-linux-gnu".to_string())
            }
            (Os::Linux, Arch::PowerPC64LE, Libc::Gnu) => {
                Some("powerpc64le-unknown-linux-gnu".to_string())
            }
            (Os::Linux, Arch::S390x, Libc::Gnu) => Some("s390x-unknown-linux-gnu".to_string()),
            (Os::Linux, Arch::Riscv64, Libc::Gnu) => {
                Some("riscv64gc-unknown-linux-gnu".to_string())
            }

            // Linux MUSL
            (Os::Linux, Arch::X86_64, Libc::Musl) => Some("x86_64-unknown-linux-musl".to_string()),
            (Os::Linux, Arch::X86, Libc::Musl) => Some("i686-unknown-linux-musl".to_string()),
            (Os::Linux, Arch::Aarch64, Libc::Musl) => {
                Some("aarch64-unknown-linux-musl".to_string())
            }
            (Os::Linux, Arch::Arm, Libc::Musl) => Some("arm-unknown-linux-musleabihf".to_string()),
            (Os::Linux, Arch::Armv7, Libc::Musl) => {
                Some("armv7-unknown-linux-musleabihf".to_string())
            }

            // Unsupported platforms
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linux_gnu_platforms() {
        let platform = Platform::new(Os::Linux, Arch::X86_64);
        assert_eq!(
            UvUrlBuilder::get_platform_string(&platform),
            Some("x86_64-unknown-linux-gnu".to_string())
        );

        let platform = Platform::new(Os::Linux, Arch::Aarch64);
        assert_eq!(
            UvUrlBuilder::get_platform_string(&platform),
            Some("aarch64-unknown-linux-gnu".to_string())
        );
    }

    #[test]
    fn test_linux_musl_platforms() {
        let platform = Platform::with_libc(Os::Linux, Arch::X86_64, Libc::Musl);
        assert_eq!(
            UvUrlBuilder::get_platform_string(&platform),
            Some("x86_64-unknown-linux-musl".to_string())
        );

        let platform = Platform::with_libc(Os::Linux, Arch::Aarch64, Libc::Musl);
        assert_eq!(
            UvUrlBuilder::get_platform_string(&platform),
            Some("aarch64-unknown-linux-musl".to_string())
        );
    }

    #[test]
    fn test_macos_platforms() {
        let platform = Platform::new(Os::MacOS, Arch::X86_64);
        assert_eq!(
            UvUrlBuilder::get_platform_string(&platform),
            Some("x86_64-apple-darwin".to_string())
        );

        let platform = Platform::new(Os::MacOS, Arch::Aarch64);
        assert_eq!(
            UvUrlBuilder::get_platform_string(&platform),
            Some("aarch64-apple-darwin".to_string())
        );
    }

    #[test]
    fn test_windows_platforms() {
        let platform = Platform::new(Os::Windows, Arch::X86_64);
        assert_eq!(
            UvUrlBuilder::get_platform_string(&platform),
            Some("x86_64-pc-windows-msvc".to_string())
        );

        let platform = Platform::new(Os::Windows, Arch::X86);
        assert_eq!(
            UvUrlBuilder::get_platform_string(&platform),
            Some("i686-pc-windows-msvc".to_string())
        );
    }

    #[test]
    fn test_download_url() {
        let platform = Platform::new(Os::Linux, Arch::X86_64);
        let url = UvUrlBuilder::download_url("0.9.26", &platform);
        assert_eq!(
            url,
            Some(
                "https://github.com/astral-sh/uv/releases/download/0.9.26/uv-x86_64-unknown-linux-gnu.tar.gz".to_string()
            )
        );
    }
}
