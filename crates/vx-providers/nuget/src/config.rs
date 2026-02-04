//! NuGet configuration and URL builder

use serde::{Deserialize, Serialize};
use vx_runtime::{Os, Platform};

/// NuGet configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NugetConfig {
    /// Default NuGet sources
    pub sources: Option<Vec<String>>,
    /// Global packages folder
    pub global_packages_folder: Option<String>,
}

/// NuGet URL builder for download URLs
pub struct NugetUrlBuilder;

impl NugetUrlBuilder {
    /// Generate download URL for NuGet CLI version
    ///
    /// NuGet CLI releases are available at:
    /// https://dist.nuget.org/win-x86-commandline/v{version}/nuget.exe
    /// or latest:
    /// https://dist.nuget.org/win-x86-commandline/latest/nuget.exe
    pub fn download_url(version: &str) -> String {
        if version == "latest" {
            "https://dist.nuget.org/win-x86-commandline/latest/nuget.exe".to_string()
        } else {
            format!(
                "https://dist.nuget.org/win-x86-commandline/v{}/nuget.exe",
                version
            )
        }
    }

    /// Get the executable name for the platform
    pub fn get_executable_name(platform: &Platform) -> &'static str {
        match &platform.os {
            Os::Windows => "nuget.exe",
            _ => "nuget", // Won't be used (nuget.exe is Windows-only)
        }
    }

    /// Check if the platform is supported
    pub fn is_platform_supported(platform: &Platform) -> bool {
        matches!(&platform.os, Os::Windows)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vx_runtime::Arch;

    #[test]
    fn test_download_url() {
        let url = NugetUrlBuilder::download_url("6.11.1");
        assert_eq!(
            url,
            "https://dist.nuget.org/win-x86-commandline/v6.11.1/nuget.exe"
        );
    }

    #[test]
    fn test_download_url_latest() {
        let url = NugetUrlBuilder::download_url("latest");
        assert_eq!(
            url,
            "https://dist.nuget.org/win-x86-commandline/latest/nuget.exe"
        );
    }

    #[test]
    fn test_executable_name_windows() {
        let platform = Platform::new(Os::Windows, Arch::X86_64);
        assert_eq!(NugetUrlBuilder::get_executable_name(&platform), "nuget.exe");
    }

    #[test]
    fn test_platform_support() {
        let windows = Platform::new(Os::Windows, Arch::X86_64);
        let linux = Platform::new(Os::Linux, Arch::X86_64);
        let macos = Platform::new(Os::MacOS, Arch::Aarch64);

        assert!(NugetUrlBuilder::is_platform_supported(&windows));
        assert!(!NugetUrlBuilder::is_platform_supported(&linux));
        assert!(!NugetUrlBuilder::is_platform_supported(&macos));
    }
}
