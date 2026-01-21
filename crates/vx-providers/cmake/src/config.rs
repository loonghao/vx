//! URL builder and platform configuration for CMake
//!
//! CMake releases are available at: https://github.com/Kitware/CMake/releases
//! Download URL format varies by platform:
//! - Windows: cmake-{version}-windows-x86_64.zip
//! - macOS: cmake-{version}-macos-universal.tar.gz
//! - Linux: cmake-{version}-linux-x86_64.tar.gz

use vx_runtime::{Arch, Os, Platform};

/// URL builder for CMake downloads
pub struct CMakeUrlBuilder;

impl CMakeUrlBuilder {
    /// Base URL for CMake releases
    const BASE_URL: &'static str = "https://github.com/Kitware/CMake/releases/download";

    /// Build the download URL for a specific version and platform
    pub fn download_url(version: &str, platform: &Platform) -> Option<String> {
        let platform_suffix = Self::get_platform_suffix(platform)?;
        let ext = Self::get_archive_extension(platform);
        // CMake uses 'v' prefix in release tags
        let version_tag = if version.starts_with('v') {
            version.to_string()
        } else {
            format!("v{}", version)
        };
        // Version in filename doesn't have 'v' prefix
        let version_num = version.trim_start_matches('v');
        Some(format!(
            "{}/{}/cmake-{}-{}.{}",
            Self::BASE_URL,
            version_tag,
            version_num,
            platform_suffix,
            ext
        ))
    }

    /// Get the platform suffix for CMake downloads
    pub fn get_platform_suffix(platform: &Platform) -> Option<&'static str> {
        match (&platform.os, &platform.arch) {
            // Windows
            (Os::Windows, Arch::X86_64) => Some("windows-x86_64"),
            (Os::Windows, Arch::Aarch64) => Some("windows-arm64"),
            // macOS - universal binary
            (Os::MacOS, Arch::X86_64 | Arch::Aarch64) => Some("macos-universal"),
            // Linux
            (Os::Linux, Arch::X86_64) => Some("linux-x86_64"),
            (Os::Linux, Arch::Aarch64) => Some("linux-aarch64"),
            _ => None,
        }
    }

    /// Get the archive extension for the platform
    pub fn get_archive_extension(platform: &Platform) -> &'static str {
        match platform.os {
            Os::Windows => "zip",
            _ => "tar.gz",
        }
    }

    /// Get the executable name for the platform
    pub fn get_executable_name(platform: &Platform) -> &'static str {
        match platform.os {
            Os::Windows => "cmake.exe",
            _ => "cmake",
        }
    }

    /// Get the directory name inside the archive
    pub fn get_archive_dir_name(version: &str, platform: &Platform) -> Option<String> {
        let platform_suffix = Self::get_platform_suffix(platform)?;
        let version_num = version.trim_start_matches('v');
        Some(format!("cmake-{}-{}", version_num, platform_suffix))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_download_url_linux_x64() {
        let platform = Platform::new(Os::Linux, Arch::X86_64);
        let url = CMakeUrlBuilder::download_url("3.31.3", &platform);
        assert_eq!(
            url,
            Some(
                "https://github.com/Kitware/CMake/releases/download/v3.31.3/cmake-3.31.3-linux-x86_64.tar.gz"
                    .to_string()
            )
        );
    }

    #[test]
    fn test_download_url_windows_x64() {
        let platform = Platform::new(Os::Windows, Arch::X86_64);
        let url = CMakeUrlBuilder::download_url("3.31.3", &platform);
        assert_eq!(
            url,
            Some(
                "https://github.com/Kitware/CMake/releases/download/v3.31.3/cmake-3.31.3-windows-x86_64.zip"
                    .to_string()
            )
        );
    }

    #[test]
    fn test_download_url_macos_arm64() {
        let platform = Platform::new(Os::MacOS, Arch::Aarch64);
        let url = CMakeUrlBuilder::download_url("3.31.3", &platform);
        assert_eq!(
            url,
            Some(
                "https://github.com/Kitware/CMake/releases/download/v3.31.3/cmake-3.31.3-macos-universal.tar.gz"
                    .to_string()
            )
        );
    }

    #[test]
    fn test_archive_dir_name() {
        let platform = Platform::new(Os::Linux, Arch::X86_64);
        let dir = CMakeUrlBuilder::get_archive_dir_name("3.31.3", &platform);
        assert_eq!(dir, Some("cmake-3.31.3-linux-x86_64".to_string()));
    }
}
