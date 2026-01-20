//! URL builder and platform configuration for ImageMagick
//!
//! ImageMagick releases are available at: https://imagemagick.org/archive/binaries/
//!
//! # Platform Support
//!
//! - **Linux x86_64**: AppImage binary (direct download)
//! - **Windows x86_64/ARM64**: Portable .7z archive (direct download)
//! - **macOS**: Not available for direct download (use Homebrew)
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
    /// Supports:
    /// - Linux x86_64: AppImage binary (direct download)
    /// - Windows: No direct download (use winget/choco/scoop via system_install strategies)
    /// - macOS: No direct download (use Homebrew)
    ///
    /// Note: Windows portable .7z archives exist but we prefer system package managers
    /// (winget priority=95) for better reliability and integration with CI environments.
    pub fn download_url(_version: &str, platform: &Platform) -> Option<String> {
        match (&platform.os, &platform.arch) {
            // Linux x86_64: Use AppImage
            // The AppImage is always named "magick" (no version in filename)
            (Os::Linux, Arch::X86_64) => Some(format!("{}/magick", Self::BASE_URL)),

            // Windows: Use system package managers (winget/choco/scoop)
            // Portable .7z archives exist but package managers are more reliable in CI
            // See provider.toml system_install.strategies for priority order
            (Os::Windows, _) => None,

            // macOS: No direct download available
            // Users should use: brew install imagemagick
            (Os::MacOS, _) => None,

            // Other platforms not supported
            _ => None,
        }
    }

    /// Get the archive extension for the platform
    pub fn get_archive_extension(platform: &Platform) -> Option<&'static str> {
        match &platform.os {
            Os::Linux => None, // AppImage is a binary, not an archive
            // Windows uses package managers, no archive to download
            Os::Windows => None,
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
    ///
    /// Only Linux x86_64 supports direct download (AppImage).
    /// Windows uses system package managers (winget/choco/scoop).
    /// macOS uses Homebrew.
    pub fn is_direct_download_supported(platform: &Platform) -> bool {
        matches!((&platform.os, &platform.arch), (Os::Linux, Arch::X86_64))
    }

    /// Get installation instructions for unsupported platforms
    pub fn get_installation_instructions(platform: &Platform) -> Option<&'static str> {
        match &platform.os {
            Os::MacOS => Some("brew install imagemagick"),
            // Windows uses package managers via system_install strategies
            Os::Windows => Some("winget install ImageMagick.ImageMagick  # or: choco install imagemagick"),
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
    fn test_download_url_windows_x64_uses_package_manager() {
        // Windows should use package managers, not direct download
        let platform = Platform {
            os: Os::Windows,
            arch: Arch::X86_64,
        };
        let url = ImageMagickUrlBuilder::download_url("7.1.2-12", &platform);
        assert!(url.is_none(), "Windows should use package managers, not direct download");
    }

    #[test]
    fn test_download_url_windows_arm64_uses_package_manager() {
        // Windows ARM64 should also use package managers
        let platform = Platform {
            os: Os::Windows,
            arch: Arch::Aarch64,
        };
        let url = ImageMagickUrlBuilder::download_url("7.1.2-12", &platform);
        assert!(url.is_none(), "Windows ARM64 should use package managers, not direct download");
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
        // Only Linux x86_64 supports direct download (AppImage)
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

        // Windows uses package managers (winget/choco/scoop)
        let windows_x64 = Platform {
            os: Os::Windows,
            arch: Arch::X86_64,
        };
        assert!(!ImageMagickUrlBuilder::is_direct_download_supported(
            &windows_x64
        ));

        let windows_arm64 = Platform {
            os: Os::Windows,
            arch: Arch::Aarch64,
        };
        assert!(!ImageMagickUrlBuilder::is_direct_download_supported(
            &windows_arm64
        ));

        // macOS uses Homebrew
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

        // Windows should have instructions for package managers
        let windows_x64 = Platform {
            os: Os::Windows,
            arch: Arch::X86_64,
        };
        let instructions = ImageMagickUrlBuilder::get_installation_instructions(&windows_x64);
        assert!(instructions.is_some());
        assert!(instructions.unwrap().contains("winget") || instructions.unwrap().contains("choco"));

        // Linux x86_64 has direct download, no instructions needed
        let linux_x64 = Platform {
            os: Os::Linux,
            arch: Arch::X86_64,
        };
        let instructions = ImageMagickUrlBuilder::get_installation_instructions(&linux_x64);
        assert!(instructions.is_none()); // Direct download supported
    }

    #[test]
    fn test_archive_extension() {
        let linux = Platform {
            os: Os::Linux,
            arch: Arch::X86_64,
        };
        assert_eq!(ImageMagickUrlBuilder::get_archive_extension(&linux), None);

        // Windows uses package managers, no archive extension
        let windows = Platform {
            os: Os::Windows,
            arch: Arch::X86_64,
        };
        assert_eq!(
            ImageMagickUrlBuilder::get_archive_extension(&windows),
            None
        );
    }
}
