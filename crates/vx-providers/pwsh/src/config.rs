//! URL builder and platform configuration for PowerShell
//!
//! PowerShell releases are available at: https://github.com/PowerShell/PowerShell/releases
//! Download URL format: https://github.com/PowerShell/PowerShell/releases/download/v{version}/powershell-{version}-{platform}.{ext}

use vx_runtime::{Arch, Os, Platform};

/// URL builder for PowerShell downloads
pub struct PwshUrlBuilder;

impl PwshUrlBuilder {
    /// Base URL for PowerShell releases
    const BASE_URL: &'static str = "https://github.com/PowerShell/PowerShell/releases/download";

    /// Build the download URL for a specific version and platform
    pub fn download_url(version: &str, platform: &Platform) -> Option<String> {
        let platform_str = Self::get_platform_string(platform)?;
        let ext = Self::get_archive_extension(platform);
        Some(format!(
            "{}/v{}/powershell-{}-{}.{}",
            Self::BASE_URL,
            version,
            version,
            platform_str,
            ext
        ))
    }

    /// Get the platform string for the download URL
    pub fn get_platform_string(platform: &Platform) -> Option<String> {
        match (&platform.os, &platform.arch) {
            // Windows
            (Os::Windows, Arch::X86_64) => Some("win-x64".to_string()),
            (Os::Windows, Arch::X86) => Some("win-x86".to_string()),
            (Os::Windows, Arch::Aarch64) => Some("win-arm64".to_string()),

            // macOS
            (Os::MacOS, Arch::X86_64) => Some("osx-x64".to_string()),
            (Os::MacOS, Arch::Aarch64) => Some("osx-arm64".to_string()),

            // Linux
            (Os::Linux, Arch::X86_64) => Some("linux-x64".to_string()),
            (Os::Linux, Arch::Aarch64) => Some("linux-arm64".to_string()),
            (Os::Linux, Arch::Arm) => Some("linux-arm32".to_string()),

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
            Os::Windows => "pwsh.exe",
            _ => "pwsh",
        }
    }

    /// Get the archive directory name (PowerShell extracts directly, no subdirectory)
    pub fn get_archive_dir_name(_version: &str, _platform: &Platform) -> String {
        // PowerShell archives extract files directly without a subdirectory
        String::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_download_url_linux_x64() {
        let platform = Platform::new(Os::Linux, Arch::X86_64);
        let url = PwshUrlBuilder::download_url("7.4.0", &platform);
        assert_eq!(
            url,
            Some("https://github.com/PowerShell/PowerShell/releases/download/v7.4.0/powershell-7.4.0-linux-x64.tar.gz".to_string())
        );
    }

    #[test]
    fn test_download_url_windows_x64() {
        let platform = Platform::new(Os::Windows, Arch::X86_64);
        let url = PwshUrlBuilder::download_url("7.4.0", &platform);
        assert_eq!(
            url,
            Some("https://github.com/PowerShell/PowerShell/releases/download/v7.4.0/powershell-7.4.0-win-x64.zip".to_string())
        );
    }

    #[test]
    fn test_download_url_macos_arm64() {
        let platform = Platform::new(Os::MacOS, Arch::Aarch64);
        let url = PwshUrlBuilder::download_url("7.4.0", &platform);
        assert_eq!(
            url,
            Some("https://github.com/PowerShell/PowerShell/releases/download/v7.4.0/powershell-7.4.0-osx-arm64.tar.gz".to_string())
        );
    }
}
