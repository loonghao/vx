//! URL builder and platform configuration for Hadolint
//!
//! Hadolint releases are available at: https://github.com/hadolint/hadolint/releases
//! Download URL format: https://github.com/hadolint/hadolint/releases/download/v{version}/hadolint-{os}-{arch}[.exe]
//!
//! Supported platforms:
//! - Linux x86_64, ARM64
//! - macOS x86_64, ARM64
//! - Windows x86_64 only

use serde::{Deserialize, Serialize};
use vx_runtime::{Arch, Os, Platform};

/// Hadolint configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HadolintConfig {
    /// Default hadolint version
    pub default_version: Option<String>,
}

/// URL builder for Hadolint downloads
pub struct HadolintUrlBuilder;

impl HadolintUrlBuilder {
    /// Base URL for Hadolint releases
    const BASE_URL: &'static str = "https://github.com/hadolint/hadolint/releases/download";

    /// Build the download URL for a specific version and platform
    ///
    /// Hadolint uses the naming convention:
    /// - hadolint-linux-x86_64
    /// - hadolint-linux-arm64
    /// - hadolint-macos-x86_64
    /// - hadolint-macos-arm64
    /// - hadolint-windows-x86_64.exe
    pub fn download_url(version: &str, platform: &Platform) -> Option<String> {
        let asset_name = Self::get_asset_name(platform)?;
        Some(format!("{}/v{}/{}", Self::BASE_URL, version, asset_name))
    }

    /// Get the release asset name for the platform
    pub fn get_asset_name(platform: &Platform) -> Option<String> {
        let os_str = Self::get_os_string(platform)?;
        let arch_str = Self::get_arch_string(platform)?;
        let ext = Self::get_executable_extension(platform);

        Some(format!("hadolint-{}-{}{}", os_str, arch_str, ext))
    }

    /// Get the OS string for the download URL
    fn get_os_string(platform: &Platform) -> Option<&'static str> {
        match &platform.os {
            Os::Windows => Some("windows"),
            Os::MacOS => Some("macos"),
            Os::Linux => Some("linux"),
            _ => None,
        }
    }

    /// Get the architecture string for the download URL
    fn get_arch_string(platform: &Platform) -> Option<&'static str> {
        match (&platform.os, &platform.arch) {
            // Windows only supports x86_64
            (Os::Windows, Arch::X86_64) => Some("x86_64"),
            (Os::Windows, _) => None,
            // macOS and Linux support both x86_64 and arm64
            (_, Arch::X86_64) => Some("x86_64"),
            (_, Arch::Aarch64) => Some("arm64"),
            _ => None,
        }
    }

    /// Get the executable extension for the platform
    fn get_executable_extension(platform: &Platform) -> &'static str {
        match platform.os {
            Os::Windows => ".exe",
            _ => "",
        }
    }

    /// Get the executable name for the platform
    pub fn get_executable_name(platform: &Platform) -> &'static str {
        match platform.os {
            Os::Windows => "hadolint.exe",
            _ => "hadolint",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_download_url_linux_x64() {
        let platform = Platform::new(Os::Linux, Arch::X86_64);
        let url = HadolintUrlBuilder::download_url("2.14.0", &platform);
        assert!(url.is_some());
        assert_eq!(
            url.unwrap(),
            "https://github.com/hadolint/hadolint/releases/download/v2.14.0/hadolint-linux-x86_64"
        );
    }

    #[test]
    fn test_download_url_linux_arm64() {
        let platform = Platform::new(Os::Linux, Arch::Aarch64);
        let url = HadolintUrlBuilder::download_url("2.14.0", &platform);
        assert!(url.is_some());
        assert_eq!(
            url.unwrap(),
            "https://github.com/hadolint/hadolint/releases/download/v2.14.0/hadolint-linux-arm64"
        );
    }

    #[test]
    fn test_download_url_macos_x64() {
        let platform = Platform::new(Os::MacOS, Arch::X86_64);
        let url = HadolintUrlBuilder::download_url("2.14.0", &platform);
        assert!(url.is_some());
        assert_eq!(
            url.unwrap(),
            "https://github.com/hadolint/hadolint/releases/download/v2.14.0/hadolint-macos-x86_64"
        );
    }

    #[test]
    fn test_download_url_macos_arm64() {
        let platform = Platform::new(Os::MacOS, Arch::Aarch64);
        let url = HadolintUrlBuilder::download_url("2.14.0", &platform);
        assert!(url.is_some());
        assert_eq!(
            url.unwrap(),
            "https://github.com/hadolint/hadolint/releases/download/v2.14.0/hadolint-macos-arm64"
        );
    }

    #[test]
    fn test_download_url_windows_x64() {
        let platform = Platform::new(Os::Windows, Arch::X86_64);
        let url = HadolintUrlBuilder::download_url("2.14.0", &platform);
        assert!(url.is_some());
        assert_eq!(
            url.unwrap(),
            "https://github.com/hadolint/hadolint/releases/download/v2.14.0/hadolint-windows-x86_64.exe"
        );
    }

    #[test]
    fn test_download_url_windows_arm64_not_supported() {
        let platform = Platform::new(Os::Windows, Arch::Aarch64);
        let url = HadolintUrlBuilder::download_url("2.14.0", &platform);
        assert!(url.is_none());
    }
}
