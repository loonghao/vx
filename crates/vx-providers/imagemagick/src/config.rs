//! URL builder and platform configuration for ImageMagick
//!
//! ImageMagick releases are available at: https://imagemagick.org/archive/binaries/
//!
//! # Platform Support
//!
//! - **Linux x86_64**: AppImage binary (direct download)
//! - **macOS**: Not available for direct download (use Homebrew)
//! - **Windows**: Not available for direct download (use Chocolatey/Scoop)
//!
//! The AppImage is a self-contained binary that works on most Linux distributions.

use vx_runtime::{Arch, Os, Platform};

/// URL builder for ImageMagick downloads
pub struct ImageMagickUrlBuilder;

impl ImageMagickUrlBuilder {
    /// Base URL for ImageMagick binaries
    const BASE_URL: &'static str = "https://imagemagick.org/archive/binaries";

    /// Build the download URL for a specific version and platform
    ///
    /// Currently only supports Linux x86_64 via AppImage.
    /// Other platforms should use system package managers.
    pub fn download_url(_version: &str, platform: &Platform) -> Option<String> {
        match (&platform.os, &platform.arch) {
            // Linux x86_64: Use AppImage
            // The AppImage is always named "magick" (no version in filename)
            (Os::Linux, Arch::X86_64) => Some(format!("{}/magick", Self::BASE_URL)),

            // macOS: No direct download available
            // Users should use: brew install imagemagick
            (Os::MacOS, _) => None,

            // Windows: No supported format available
            // Windows uses .7z format which vx doesn't support
            // Users should use: choco install imagemagick
            // Or: scoop install imagemagick
            (Os::Windows, _) => None,

            // Other platforms not supported
            _ => None,
        }
    }

    /// Get the archive extension for the platform
    ///
    /// Linux AppImage is a binary file (no extension needed)
    pub fn get_archive_extension(platform: &Platform) -> Option<&'static str> {
        match &platform.os {
            Os::Linux => None, // AppImage is a binary, not an archive
            _ => None,
        }
    }

    /// Get the executable name for the platform
    pub fn get_executable_name(platform: &Platform) -> &'static str {
        match platform.os {
            Os::Windows => "magick.exe",
            _ => "magick",
        }
    }

    /// Check if direct download is supported for the platform
    pub fn is_direct_download_supported(platform: &Platform) -> bool {
        matches!((&platform.os, &platform.arch), (Os::Linux, Arch::X86_64))
    }

    /// Get installation instructions for unsupported platforms
    pub fn get_installation_instructions(platform: &Platform) -> Option<&'static str> {
        match &platform.os {
            Os::MacOS => Some("brew install imagemagick"),
            Os::Windows => Some("choco install imagemagick  # or: scoop install imagemagick"),
            Os::Linux => match &platform.arch {
                Arch::X86_64 => None, // Direct download supported
                _ => Some("Use your distribution's package manager (apt, dnf, pacman, etc.)"),
            },
            _ => Some("Use your system's package manager to install ImageMagick"),
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
        let url = ImageMagickUrlBuilder::download_url("7.1.2-12", &platform);
        assert!(url.is_some());
        let url = url.unwrap();
        assert!(url.contains("imagemagick.org/archive/binaries"));
        assert!(url.ends_with("/magick"));
    }

    #[test]
    fn test_download_url_windows_not_supported() {
        let platform = Platform {
            os: Os::Windows,
            arch: Arch::X86_64,
        };
        let url = ImageMagickUrlBuilder::download_url("7.1.2-12", &platform);
        assert!(url.is_none());
    }

    #[test]
    fn test_download_url_macos_not_supported() {
        let platform = Platform {
            os: Os::MacOS,
            arch: Arch::Aarch64,
        };
        let url = ImageMagickUrlBuilder::download_url("7.1.2-12", &platform);
        assert!(url.is_none());
    }

    #[test]
    fn test_executable_name_linux() {
        let platform = Platform {
            os: Os::Linux,
            arch: Arch::X86_64,
        };
        assert_eq!(
            ImageMagickUrlBuilder::get_executable_name(&platform),
            "magick"
        );
    }

    #[test]
    fn test_executable_name_windows() {
        let platform = Platform {
            os: Os::Windows,
            arch: Arch::X86_64,
        };
        assert_eq!(
            ImageMagickUrlBuilder::get_executable_name(&platform),
            "magick.exe"
        );
    }

    #[test]
    fn test_is_direct_download_supported() {
        let linux_x64 = Platform {
            os: Os::Linux,
            arch: Arch::X86_64,
        };
        assert!(ImageMagickUrlBuilder::is_direct_download_supported(
            &linux_x64
        ));

        let linux_arm = Platform {
            os: Os::Linux,
            arch: Arch::Aarch64,
        };
        assert!(!ImageMagickUrlBuilder::is_direct_download_supported(
            &linux_arm
        ));

        let windows = Platform {
            os: Os::Windows,
            arch: Arch::X86_64,
        };
        assert!(!ImageMagickUrlBuilder::is_direct_download_supported(
            &windows
        ));

        let macos = Platform {
            os: Os::MacOS,
            arch: Arch::Aarch64,
        };
        assert!(!ImageMagickUrlBuilder::is_direct_download_supported(&macos));
    }

    #[test]
    fn test_installation_instructions() {
        let macos = Platform {
            os: Os::MacOS,
            arch: Arch::Aarch64,
        };
        let instructions = ImageMagickUrlBuilder::get_installation_instructions(&macos);
        assert!(instructions.is_some());
        assert!(instructions.unwrap().contains("brew"));

        let windows = Platform {
            os: Os::Windows,
            arch: Arch::X86_64,
        };
        let instructions = ImageMagickUrlBuilder::get_installation_instructions(&windows);
        assert!(instructions.is_some());
        assert!(instructions.unwrap().contains("choco"));

        let linux_x64 = Platform {
            os: Os::Linux,
            arch: Arch::X86_64,
        };
        let instructions = ImageMagickUrlBuilder::get_installation_instructions(&linux_x64);
        assert!(instructions.is_none()); // Direct download supported
    }
}
