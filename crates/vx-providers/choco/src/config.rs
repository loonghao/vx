//! Chocolatey configuration and URL builder

use serde::{Deserialize, Serialize};
use vx_runtime::{Arch, Os, Platform};

/// Chocolatey configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ChocoConfig {
    /// Default Chocolatey version
    pub default_version: Option<String>,
    /// Install directory
    pub install_dir: Option<String>,
}

/// Chocolatey URL builder for download URLs
pub struct ChocoUrlBuilder;

impl ChocoUrlBuilder {
    /// Generate download URL for Chocolatey version
    ///
    /// Chocolatey releases are available at:
    /// https://github.com/chocolatey/choco/releases/download/{version}/chocolatey.{version}.nupkg
    /// or the portable zip:
    /// https://github.com/chocolatey/choco/releases/download/{version}/chocolatey.v{version}.zip
    pub fn download_url(version: &str, _platform: &Platform) -> Option<String> {
        // Chocolatey is Windows-only, use the portable zip distribution
        Some(format!(
            "https://github.com/chocolatey/choco/releases/download/{}/chocolatey.v{}.zip",
            version, version
        ))
    }

    /// Get the target triple for the platform
    #[allow(dead_code)]
    pub fn get_target_triple(platform: &Platform) -> Option<&'static str> {
        match (&platform.os, &platform.arch) {
            (Os::Windows, Arch::X86_64) => Some("win-x64"),
            (Os::Windows, Arch::X86) => Some("win-x86"),
            // Chocolatey is Windows-only
            _ => None,
        }
    }

    /// Get the archive extension for the platform
    #[allow(dead_code)]
    pub fn get_archive_extension(_platform: &Platform) -> &'static str {
        // Chocolatey uses zip archives
        "zip"
    }

    /// Get the executable name for the platform
    pub fn get_executable_name(platform: &Platform) -> &'static str {
        match &platform.os {
            Os::Windows => "choco.exe",
            _ => "choco", // Won't be used, but provide a fallback
        }
    }

    /// Check if the platform is supported
    #[allow(dead_code)]
    pub fn is_platform_supported(platform: &Platform) -> bool {
        matches!(&platform.os, Os::Windows)
    }

    /// Get the directory name inside the archive
    pub fn get_archive_dir_name() -> &'static str {
        // Chocolatey zip extracts to a 'tools' directory
        "tools"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_download_url() {
        let platform = Platform::current();
        let url = ChocoUrlBuilder::download_url("2.4.3", &platform);
        assert!(url.is_some());
        let url = url.unwrap();
        assert!(url.contains("2.4.3"));
        assert!(url.contains("chocolatey"));
        assert!(url.ends_with(".zip"));
    }

    #[test]
    fn test_archive_extension() {
        let platform = Platform::current();
        assert_eq!(ChocoUrlBuilder::get_archive_extension(&platform), "zip");
    }

    #[test]
    fn test_executable_name_windows() {
        let platform = Platform {
            os: Os::Windows,
            arch: Arch::X86_64,
        };
        assert_eq!(ChocoUrlBuilder::get_executable_name(&platform), "choco.exe");
    }
}
