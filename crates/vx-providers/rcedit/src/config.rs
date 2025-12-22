//! URL builder and platform configuration for rcedit
//!
//! rcedit releases are available at: https://github.com/electron/rcedit/releases
//! Download URL format: https://github.com/electron/rcedit/releases/download/v{version}/rcedit-{arch}.exe
//!
//! Note: rcedit is Windows-only and downloads as a single executable (no archive)

use vx_runtime::{Arch, Os, Platform};

/// URL builder for rcedit downloads
pub struct RceditUrlBuilder;

impl RceditUrlBuilder {
    /// Base URL for rcedit releases
    const BASE_URL: &'static str = "https://github.com/electron/rcedit/releases/download";

    /// Build the download URL for a specific version and platform
    ///
    /// rcedit is Windows-only and provides x64 and arm64 builds
    pub fn download_url(version: &str, platform: &Platform) -> Option<String> {
        // rcedit is Windows-only
        if platform.os != Os::Windows {
            return None;
        }

        let arch_suffix = Self::get_arch_suffix(platform)?;
        Some(format!(
            "{}/v{}/rcedit-{}.exe",
            Self::BASE_URL,
            version,
            arch_suffix
        ))
    }

    /// Get the architecture suffix for the download URL
    pub fn get_arch_suffix(platform: &Platform) -> Option<&'static str> {
        match (&platform.os, &platform.arch) {
            (Os::Windows, Arch::X86_64) => Some("x64"),
            (Os::Windows, Arch::Aarch64) => Some("arm64"),
            // x86 (32-bit) is also supported
            (Os::Windows, Arch::X86) => Some("x86"),
            _ => None,
        }
    }

    /// Get the archive extension for the platform
    ///
    /// rcedit is distributed as a single executable, not an archive
    pub fn get_archive_extension(_platform: &Platform) -> Option<&'static str> {
        // rcedit is a single .exe file, not an archive
        None
    }

    /// Get the executable name for the platform
    pub fn get_executable_name(platform: &Platform) -> &'static str {
        match (&platform.os, &platform.arch) {
            (Os::Windows, Arch::X86_64) => "rcedit-x64.exe",
            (Os::Windows, Arch::Aarch64) => "rcedit-arm64.exe",
            (Os::Windows, Arch::X86) => "rcedit-x86.exe",
            _ => "rcedit.exe",
        }
    }

    /// Check if the platform is supported
    pub fn is_platform_supported(platform: &Platform) -> bool {
        matches!(
            (&platform.os, &platform.arch),
            (Os::Windows, Arch::X86_64) | (Os::Windows, Arch::Aarch64) | (Os::Windows, Arch::X86)
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
        let url = RceditUrlBuilder::download_url("2.0.0", &platform);
        assert_eq!(
            url,
            Some(
                "https://github.com/electron/rcedit/releases/download/v2.0.0/rcedit-x64.exe"
                    .to_string()
            )
        );
    }

    #[test]
    fn test_download_url_windows_arm64() {
        let platform = Platform {
            os: Os::Windows,
            arch: Arch::Aarch64,
        };
        let url = RceditUrlBuilder::download_url("2.0.0", &platform);
        assert_eq!(
            url,
            Some(
                "https://github.com/electron/rcedit/releases/download/v2.0.0/rcedit-arm64.exe"
                    .to_string()
            )
        );
    }

    #[test]
    fn test_download_url_linux_unsupported() {
        let platform = Platform {
            os: Os::Linux,
            arch: Arch::X86_64,
        };
        let url = RceditUrlBuilder::download_url("2.0.0", &platform);
        assert_eq!(url, None);
    }

    #[test]
    fn test_download_url_macos_unsupported() {
        let platform = Platform {
            os: Os::MacOS,
            arch: Arch::Aarch64,
        };
        let url = RceditUrlBuilder::download_url("2.0.0", &platform);
        assert_eq!(url, None);
    }

    #[test]
    fn test_executable_name() {
        let platform_x64 = Platform {
            os: Os::Windows,
            arch: Arch::X86_64,
        };
        assert_eq!(
            RceditUrlBuilder::get_executable_name(&platform_x64),
            "rcedit-x64.exe"
        );

        let platform_arm64 = Platform {
            os: Os::Windows,
            arch: Arch::Aarch64,
        };
        assert_eq!(
            RceditUrlBuilder::get_executable_name(&platform_arm64),
            "rcedit-arm64.exe"
        );
    }

    #[test]
    fn test_platform_supported() {
        let windows_x64 = Platform {
            os: Os::Windows,
            arch: Arch::X86_64,
        };
        assert!(RceditUrlBuilder::is_platform_supported(&windows_x64));

        let linux_x64 = Platform {
            os: Os::Linux,
            arch: Arch::X86_64,
        };
        assert!(!RceditUrlBuilder::is_platform_supported(&linux_x64));
    }
}
