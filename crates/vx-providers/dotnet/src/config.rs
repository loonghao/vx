//! URL builder and platform configuration for .NET SDK
//!
//! .NET SDK releases are available at: https://dotnet.microsoft.com/download
//! Download URL format: https://builds.dotnet.microsoft.com/dotnet/Sdk/{version}/dotnet-sdk-{version}-{rid}.{ext}
//!
//! RID (Runtime Identifier) format:
//! - Windows: win-x64, win-x86, win-arm64
//! - macOS: osx-x64, osx-arm64
//! - Linux: linux-x64, linux-arm64, linux-arm, linux-musl-x64, linux-musl-arm64

use vx_runtime::{Arch, Os, Platform};

/// URL builder for .NET SDK downloads
pub struct DotnetUrlBuilder;

impl DotnetUrlBuilder {
    /// Base URL for .NET SDK releases
    const BASE_URL: &'static str = "https://builds.dotnet.microsoft.com/dotnet/Sdk";

    /// Build the download URL for a specific version and platform
    pub fn download_url(version: &str, platform: &Platform) -> Option<String> {
        let rid = Self::get_runtime_identifier(platform)?;
        let ext = Self::get_archive_extension(platform);

        Some(format!(
            "{}/{}/dotnet-sdk-{}-{}.{}",
            Self::BASE_URL,
            version,
            version,
            rid,
            ext
        ))
    }

    /// Get the Runtime Identifier (RID) for the platform
    /// See: https://learn.microsoft.com/dotnet/core/rid-catalog
    pub fn get_runtime_identifier(platform: &Platform) -> Option<String> {
        match (&platform.os, &platform.arch) {
            // Windows
            (Os::Windows, Arch::X86_64) => Some("win-x64".to_string()),
            (Os::Windows, Arch::X86) => Some("win-x86".to_string()),
            (Os::Windows, Arch::Aarch64) => Some("win-arm64".to_string()),

            // macOS
            (Os::MacOS, Arch::X86_64) => Some("osx-x64".to_string()),
            (Os::MacOS, Arch::Aarch64) => Some("osx-arm64".to_string()),

            // Linux (glibc)
            (Os::Linux, Arch::X86_64) => Some("linux-x64".to_string()),
            (Os::Linux, Arch::Aarch64) => Some("linux-arm64".to_string()),
            (Os::Linux, Arch::Arm) => Some("linux-arm".to_string()),

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
            Os::Windows => "dotnet.exe",
            _ => "dotnet",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_download_url_linux_x64() {
        let platform = Platform::new(Os::Linux, Arch::X86_64);
        let url = DotnetUrlBuilder::download_url("9.0.310", &platform);
        assert!(url.is_some());
        let url = url.unwrap();
        assert!(url.contains("builds.dotnet.microsoft.com"));
        assert!(url.contains("9.0.310"));
        assert!(url.contains("linux-x64"));
        assert!(url.ends_with(".tar.gz"));
    }

    #[test]
    fn test_download_url_windows_x64() {
        let platform = Platform::new(Os::Windows, Arch::X86_64);
        let url = DotnetUrlBuilder::download_url("9.0.310", &platform);
        assert!(url.is_some());
        let url = url.unwrap();
        assert!(url.contains("win-x64"));
        assert!(url.ends_with(".zip"));
    }

    #[test]
    fn test_download_url_macos_arm64() {
        let platform = Platform::new(Os::MacOS, Arch::Aarch64);
        let url = DotnetUrlBuilder::download_url("9.0.310", &platform);
        assert!(url.is_some());
        let url = url.unwrap();
        assert!(url.contains("osx-arm64"));
        assert!(url.ends_with(".tar.gz"));
    }

    #[test]
    fn test_runtime_identifier() {
        // Windows
        assert_eq!(
            DotnetUrlBuilder::get_runtime_identifier(&Platform::new(Os::Windows, Arch::X86_64)),
            Some("win-x64".to_string())
        );
        assert_eq!(
            DotnetUrlBuilder::get_runtime_identifier(&Platform::new(Os::Windows, Arch::Aarch64)),
            Some("win-arm64".to_string())
        );

        // macOS
        assert_eq!(
            DotnetUrlBuilder::get_runtime_identifier(&Platform::new(Os::MacOS, Arch::X86_64)),
            Some("osx-x64".to_string())
        );
        assert_eq!(
            DotnetUrlBuilder::get_runtime_identifier(&Platform::new(Os::MacOS, Arch::Aarch64)),
            Some("osx-arm64".to_string())
        );

        // Linux
        assert_eq!(
            DotnetUrlBuilder::get_runtime_identifier(&Platform::new(Os::Linux, Arch::X86_64)),
            Some("linux-x64".to_string())
        );
        assert_eq!(
            DotnetUrlBuilder::get_runtime_identifier(&Platform::new(Os::Linux, Arch::Aarch64)),
            Some("linux-arm64".to_string())
        );
    }

    #[test]
    fn test_executable_name() {
        assert_eq!(
            DotnetUrlBuilder::get_executable_name(&Platform::new(Os::Windows, Arch::X86_64)),
            "dotnet.exe"
        );
        assert_eq!(
            DotnetUrlBuilder::get_executable_name(&Platform::new(Os::Linux, Arch::X86_64)),
            "dotnet"
        );
        assert_eq!(
            DotnetUrlBuilder::get_executable_name(&Platform::new(Os::MacOS, Arch::X86_64)),
            "dotnet"
        );
    }
}
