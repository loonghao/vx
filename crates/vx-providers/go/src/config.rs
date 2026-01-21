//! Go configuration and URL building
//!
//! This module provides Go-specific configuration,
//! including URL building and platform detection.

use vx_runtime::{Arch, Os, Platform};

/// Go URL builder for consistent download URL generation
pub struct GoUrlBuilder;

impl GoUrlBuilder {
    /// Generate download URL for Go version
    pub fn download_url(version: &str, platform: &Platform) -> Option<String> {
        let filename = Self::get_filename(version, platform)?;
        Some(format!("https://golang.org/dl/{}", filename))
    }

    /// Get platform-specific filename
    pub fn get_filename(version: &str, platform: &Platform) -> Option<String> {
        let platform_str = Self::get_platform_string(platform)?;
        if platform.os == Os::Windows {
            Some(format!("go{}.{}.zip", version, platform_str))
        } else {
            Some(format!("go{}.{}.tar.gz", version, platform_str))
        }
    }

    /// Get platform string for Go downloads
    /// Go uses static linking, so no glibc/musl distinction needed
    pub fn get_platform_string(platform: &Platform) -> Option<String> {
        match (&platform.os, &platform.arch) {
            // Windows
            (Os::Windows, Arch::X86_64) => Some("windows-amd64".to_string()),
            (Os::Windows, Arch::X86) => Some("windows-386".to_string()),
            (Os::Windows, Arch::Aarch64) => Some("windows-arm64".to_string()),

            // macOS
            (Os::MacOS, Arch::X86_64) => Some("darwin-amd64".to_string()),
            (Os::MacOS, Arch::Aarch64) => Some("darwin-arm64".to_string()),

            // Linux
            (Os::Linux, Arch::X86_64) => Some("linux-amd64".to_string()),
            (Os::Linux, Arch::X86) => Some("linux-386".to_string()),
            (Os::Linux, Arch::Aarch64) => Some("linux-arm64".to_string()),
            (Os::Linux, Arch::Arm) => Some("linux-armv6l".to_string()),
            (Os::Linux, Arch::PowerPC64) => Some("linux-ppc64".to_string()),
            (Os::Linux, Arch::PowerPC64LE) => Some("linux-ppc64le".to_string()),
            (Os::Linux, Arch::S390x) => Some("linux-s390x".to_string()),
            (Os::Linux, Arch::Riscv64) => Some("linux-riscv64".to_string()),

            // FreeBSD
            (Os::FreeBSD, Arch::X86_64) => Some("freebsd-amd64".to_string()),
            (Os::FreeBSD, Arch::X86) => Some("freebsd-386".to_string()),
            (Os::FreeBSD, Arch::Aarch64) => Some("freebsd-arm64".to_string()),

            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linux_platforms() {
        let platform = Platform::new(Os::Linux, Arch::X86_64);
        assert_eq!(
            GoUrlBuilder::get_platform_string(&platform),
            Some("linux-amd64".to_string())
        );

        let platform = Platform::new(Os::Linux, Arch::Aarch64);
        assert_eq!(
            GoUrlBuilder::get_platform_string(&platform),
            Some("linux-arm64".to_string())
        );

        let platform = Platform::new(Os::Linux, Arch::S390x);
        assert_eq!(
            GoUrlBuilder::get_platform_string(&platform),
            Some("linux-s390x".to_string())
        );
    }

    #[test]
    fn test_download_url() {
        let platform = Platform::new(Os::Linux, Arch::X86_64);
        let url = GoUrlBuilder::download_url("1.22.0", &platform);
        assert_eq!(
            url,
            Some("https://golang.org/dl/go1.22.0.linux-amd64.tar.gz".to_string())
        );
    }
}
