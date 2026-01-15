//! URL builder and platform configuration for jq
//!
//! jq releases are available at: https://github.com/jqlang/jq/releases
//! Download URL format: https://github.com/jqlang/jq/releases/download/jq-{version}/jq-{platform}
//!
//! Note: jq releases are direct binary downloads (not archives).
//! - Linux: jq-linux-amd64, jq-linux-arm64
//! - macOS: jq-macos-amd64, jq-macos-arm64
//! - Windows: jq-windows-amd64.exe

use vx_runtime::{Arch, Os, Platform};

/// URL builder for jq downloads
pub struct JqUrlBuilder;

impl JqUrlBuilder {
    /// Base URL for jq releases
    const BASE_URL: &'static str = "https://github.com/jqlang/jq/releases/download";

    /// Build the download URL for a specific version and platform
    pub fn download_url(version: &str, platform: &Platform) -> Option<String> {
        let asset_name = Self::get_asset_name(platform)?;
        Some(format!("{}/jq-{}/{}", Self::BASE_URL, version, asset_name))
    }

    /// Get the asset name for the platform
    pub fn get_asset_name(platform: &Platform) -> Option<String> {
        match (&platform.os, &platform.arch) {
            // Windows
            (Os::Windows, Arch::X86_64) => Some("jq-windows-amd64.exe".to_string()),
            (Os::Windows, Arch::Aarch64) => Some("jq-windows-arm64.exe".to_string()),
            (Os::Windows, Arch::X86) => Some("jq-windows-i386.exe".to_string()),

            // macOS
            (Os::MacOS, Arch::X86_64) => Some("jq-macos-amd64".to_string()),
            (Os::MacOS, Arch::Aarch64) => Some("jq-macos-arm64".to_string()),

            // Linux
            (Os::Linux, Arch::X86_64) => Some("jq-linux-amd64".to_string()),
            (Os::Linux, Arch::Aarch64) => Some("jq-linux-arm64".to_string()),
            (Os::Linux, Arch::Arm) => Some("jq-linux-armhf".to_string()),
            (Os::Linux, Arch::X86) => Some("jq-linux-i386".to_string()),

            _ => None,
        }
    }

    /// Get the executable name for the platform
    pub fn get_executable_name(platform: &Platform) -> &'static str {
        match platform.os {
            Os::Windows => "jq.exe",
            _ => "jq",
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
        let url = JqUrlBuilder::download_url("1.7.1", &platform);
        assert_eq!(
            url,
            Some(
                "https://github.com/jqlang/jq/releases/download/jq-1.7.1/jq-linux-amd64"
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
        let url = JqUrlBuilder::download_url("1.7.1", &platform);
        assert_eq!(
            url,
            Some(
                "https://github.com/jqlang/jq/releases/download/jq-1.7.1/jq-windows-amd64.exe"
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
        let url = JqUrlBuilder::download_url("1.7.1", &platform);
        assert_eq!(
            url,
            Some(
                "https://github.com/jqlang/jq/releases/download/jq-1.7.1/jq-macos-arm64"
                    .to_string()
            )
        );
    }

    #[test]
    fn test_executable_name_windows() {
        let platform = Platform {
            os: Os::Windows,
            arch: Arch::X86_64,
        };
        assert_eq!(JqUrlBuilder::get_executable_name(&platform), "jq.exe");
    }

    #[test]
    fn test_executable_name_unix() {
        let platform = Platform {
            os: Os::Linux,
            arch: Arch::X86_64,
        };
        assert_eq!(JqUrlBuilder::get_executable_name(&platform), "jq");
    }
}
