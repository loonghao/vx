//! URL builder and platform configuration for YASM
//!
//! YASM releases are available on GitHub: https://github.com/yasm/yasm/releases
//! Download URL format:
//! - Windows x64: yasm-{version}-win64.exe
//! - Windows x86: yasm-{version}-win32.exe
//! - Source: yasm-{version}.tar.gz

use vx_runtime::{Arch, Os, Platform};

/// URL builder for YASM downloads
pub struct YasmUrlBuilder;

impl YasmUrlBuilder {
    /// GitHub releases base URL
    const BASE_URL: &'static str = "https://github.com/yasm/yasm/releases/download";

    /// Build the download URL for a specific version and platform
    pub fn download_url(version: &str, platform: &Platform) -> Option<String> {
        let filename = Self::get_filename(version, platform)?;
        Some(format!("{}/v{}/{}", Self::BASE_URL, version, filename))
    }

    /// Get the download filename for the platform
    pub fn get_filename(version: &str, platform: &Platform) -> Option<String> {
        match (&platform.os, &platform.arch) {
            (Os::Windows, Arch::X86_64) => Some(format!("yasm-{}-win64.exe", version)),
            (Os::Windows, Arch::X86) => Some(format!("yasm-{}-win32.exe", version)),
            // macOS and Linux require source compilation
            _ => None,
        }
    }

    /// Get the executable name for the platform
    pub fn get_executable_name(platform: &Platform) -> &'static str {
        match platform.os {
            Os::Windows => "yasm.exe",
            _ => "yasm",
        }
    }

    /// Check if the platform is supported for binary downloads
    pub fn is_binary_supported(platform: &Platform) -> bool {
        matches!(
            (&platform.os, &platform.arch),
            (Os::Windows, Arch::X86_64) | (Os::Windows, Arch::X86)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_download_url_windows_x64() {
        let platform = Platform {
            os: Os::Windows,
            arch: Arch::X86_64,
        };
        let url = YasmUrlBuilder::download_url("1.3.0", &platform);
        assert_eq!(
            url,
            Some(
                "https://github.com/yasm/yasm/releases/download/v1.3.0/yasm-1.3.0-win64.exe"
                    .to_string()
            )
        );
    }

    #[test]
    fn test_download_url_windows_x86() {
        let platform = Platform {
            os: Os::Windows,
            arch: Arch::X86,
        };
        let url = YasmUrlBuilder::download_url("1.3.0", &platform);
        assert_eq!(
            url,
            Some(
                "https://github.com/yasm/yasm/releases/download/v1.3.0/yasm-1.3.0-win32.exe"
                    .to_string()
            )
        );
    }

    #[test]
    fn test_download_url_macos_not_supported() {
        let platform = Platform {
            os: Os::MacOS,
            arch: Arch::Aarch64,
        };
        let url = YasmUrlBuilder::download_url("1.3.0", &platform);
        assert_eq!(url, None);
    }

    #[test]
    fn test_download_url_linux_not_supported() {
        let platform = Platform {
            os: Os::Linux,
            arch: Arch::X86_64,
        };
        let url = YasmUrlBuilder::download_url("1.3.0", &platform);
        assert_eq!(url, None);
    }

    #[test]
    fn test_is_binary_supported() {
        let win64 = Platform {
            os: Os::Windows,
            arch: Arch::X86_64,
        };
        let linux = Platform {
            os: Os::Linux,
            arch: Arch::X86_64,
        };
        assert!(YasmUrlBuilder::is_binary_supported(&win64));
        assert!(!YasmUrlBuilder::is_binary_supported(&linux));
    }
}
