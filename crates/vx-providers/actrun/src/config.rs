//! URL builder and platform configuration for Actrun
//!
//! Actrun CLI releases are available at: https://github.com/actionforge/actrun-cli/releases
//! Download URL format:
//!   https://github.com/actionforge/actrun-cli/releases/download/v{version}/actrun-v{version}.cli-{arch}-{os}.{ext}
//!
//! Supported platforms (CLI variant):
//!   - Linux x64: .tar.gz
//!   - Linux arm64: .tar.gz
//!   - Windows x64: .zip
//!   - Windows arm64: .zip
//!   - macOS x64: .pkg
//!   - macOS arm64: .pkg

use vx_runtime::{Arch, Os, Platform};

/// URL builder for Actrun downloads
pub struct ActrunUrlBuilder;

impl ActrunUrlBuilder {
    /// Base URL for Actrun releases
    const BASE_URL: &'static str =
        "https://github.com/actionforge/actrun-cli/releases/download";

    /// Build the download URL for a specific version and platform
    pub fn download_url(version: &str, platform: &Platform) -> Option<String> {
        let arch = Self::get_arch_string(platform)?;
        let os = Self::get_os_string(platform)?;
        let ext = Self::get_archive_extension(platform);
        Some(format!(
            "{}/v{}/actrun-v{}.cli-{}-{}.{}",
            Self::BASE_URL,
            version,
            version,
            arch,
            os,
            ext
        ))
    }

    /// Get the architecture string matching the release naming convention
    pub fn get_arch_string(platform: &Platform) -> Option<&'static str> {
        match &platform.arch {
            Arch::X86_64 => Some("x64"),
            Arch::Aarch64 => Some("arm64"),
            _ => None,
        }
    }

    /// Get the OS string matching the release naming convention
    pub fn get_os_string(platform: &Platform) -> Option<&'static str> {
        match &platform.os {
            Os::Linux => Some("linux"),
            Os::Windows => Some("windows"),
            Os::MacOS => Some("macos"),
            _ => None,
        }
    }

    /// Get the archive extension for the platform
    pub fn get_archive_extension(platform: &Platform) -> &'static str {
        match platform.os {
            Os::Windows => "zip",
            Os::MacOS => "pkg",
            _ => "tar.gz",
        }
    }

    /// Get the executable name for the platform
    pub fn get_executable_name(platform: &Platform) -> &'static str {
        match platform.os {
            Os::Windows => "actrun.exe",
            _ => "actrun",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_download_url_linux_x64() {
        let platform = Platform::new(Os::Linux, Arch::X86_64);
        let url = ActrunUrlBuilder::download_url("0.14.6", &platform);
        assert_eq!(
            url,
            Some("https://github.com/actionforge/actrun-cli/releases/download/v0.14.6/actrun-v0.14.6.cli-x64-linux.tar.gz".to_string())
        );
    }

    #[test]
    fn test_download_url_linux_arm64() {
        let platform = Platform::new(Os::Linux, Arch::Aarch64);
        let url = ActrunUrlBuilder::download_url("0.14.6", &platform);
        assert_eq!(
            url,
            Some("https://github.com/actionforge/actrun-cli/releases/download/v0.14.6/actrun-v0.14.6.cli-arm64-linux.tar.gz".to_string())
        );
    }

    #[test]
    fn test_download_url_windows_x64() {
        let platform = Platform::new(Os::Windows, Arch::X86_64);
        let url = ActrunUrlBuilder::download_url("0.14.6", &platform);
        assert_eq!(
            url,
            Some("https://github.com/actionforge/actrun-cli/releases/download/v0.14.6/actrun-v0.14.6.cli-x64-windows.zip".to_string())
        );
    }

    #[test]
    fn test_download_url_windows_arm64() {
        let platform = Platform::new(Os::Windows, Arch::Aarch64);
        let url = ActrunUrlBuilder::download_url("0.14.6", &platform);
        assert_eq!(
            url,
            Some("https://github.com/actionforge/actrun-cli/releases/download/v0.14.6/actrun-v0.14.6.cli-arm64-windows.zip".to_string())
        );
    }

    #[test]
    fn test_download_url_macos_arm64() {
        let platform = Platform::new(Os::MacOS, Arch::Aarch64);
        let url = ActrunUrlBuilder::download_url("0.14.6", &platform);
        assert_eq!(
            url,
            Some("https://github.com/actionforge/actrun-cli/releases/download/v0.14.6/actrun-v0.14.6.cli-arm64-macos.pkg".to_string())
        );
    }

    #[test]
    fn test_download_url_macos_x64() {
        let platform = Platform::new(Os::MacOS, Arch::X86_64);
        let url = ActrunUrlBuilder::download_url("0.14.6", &platform);
        assert_eq!(
            url,
            Some("https://github.com/actionforge/actrun-cli/releases/download/v0.14.6/actrun-v0.14.6.cli-x64-macos.pkg".to_string())
        );
    }

    #[test]
    fn test_archive_extension_macos() {
        let platform = Platform::new(Os::MacOS, Arch::Aarch64);
        assert_eq!(ActrunUrlBuilder::get_archive_extension(&platform), "pkg");
    }

    #[test]
    fn test_os_string_macos() {
        let platform = Platform::new(Os::MacOS, Arch::Aarch64);
        assert_eq!(ActrunUrlBuilder::get_os_string(&platform), Some("macos"));
    }
}
