//! URL builder and platform configuration for Ninja
//!
//! Ninja releases are available at: https://github.com/ninja-build/ninja/releases
//! Download URL format: https://github.com/ninja-build/ninja/releases/download/v{version}/ninja-{platform}.zip

use vx_runtime::{Arch, Os, Platform};

/// URL builder for Ninja downloads
pub struct NinjaUrlBuilder;

impl NinjaUrlBuilder {
    /// Base URL for Ninja releases
    const BASE_URL: &'static str = "https://github.com/ninja-build/ninja/releases/download";

    /// Build the download URL for a specific version and platform
    pub fn download_url(version: &str, platform: &Platform) -> Option<String> {
        let platform_name = Self::get_platform_name(platform)?;
        // Ninja uses 'v' prefix in release tags
        let version_tag = if version.starts_with('v') {
            version.to_string()
        } else {
            format!("v{}", version)
        };
        Some(format!(
            "{}/{}/ninja-{}.zip",
            Self::BASE_URL,
            version_tag,
            platform_name
        ))
    }

    /// Get the platform name for Ninja downloads
    pub fn get_platform_name(platform: &Platform) -> Option<&'static str> {
        match (&platform.os, &platform.arch) {
            // Windows - only x64
            (Os::Windows, Arch::X86_64) => Some("win"),
            // macOS - universal binary
            (Os::MacOS, Arch::X86_64 | Arch::Aarch64) => Some("mac"),
            // Linux - only x64
            (Os::Linux, Arch::X86_64) => Some("linux"),
            // Linux ARM64 added in recent versions
            (Os::Linux, Arch::Aarch64) => Some("linux-aarch64"),
            _ => None,
        }
    }

    /// Get the executable name for the platform
    pub fn get_executable_name(platform: &Platform) -> &'static str {
        match platform.os {
            Os::Windows => "ninja.exe",
            _ => "ninja",
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
        let url = NinjaUrlBuilder::download_url("1.12.1", &platform);
        assert_eq!(
            url,
            Some(
                "https://github.com/ninja-build/ninja/releases/download/v1.12.1/ninja-linux.zip"
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
        let url = NinjaUrlBuilder::download_url("1.12.1", &platform);
        assert_eq!(
            url,
            Some(
                "https://github.com/ninja-build/ninja/releases/download/v1.12.1/ninja-win.zip"
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
        let url = NinjaUrlBuilder::download_url("1.12.1", &platform);
        assert_eq!(
            url,
            Some(
                "https://github.com/ninja-build/ninja/releases/download/v1.12.1/ninja-mac.zip"
                    .to_string()
            )
        );
    }

    #[test]
    fn test_download_url_with_v_prefix() {
        let platform = Platform {
            os: Os::Linux,
            arch: Arch::X86_64,
        };
        let url = NinjaUrlBuilder::download_url("v1.12.1", &platform);
        assert_eq!(
            url,
            Some(
                "https://github.com/ninja-build/ninja/releases/download/v1.12.1/ninja-linux.zip"
                    .to_string()
            )
        );
    }
}
