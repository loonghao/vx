//! URL builder and platform configuration for fzf
//!
//! fzf releases: https://github.com/junegunn/fzf/releases
//! URL format: https://github.com/junegunn/fzf/releases/download/v{version}/fzf-{version}-{platform}.{ext}

use vx_runtime::{Arch, Os, Platform};

pub struct FzfUrlBuilder;

impl FzfUrlBuilder {
    const BASE_URL: &'static str = "https://github.com/junegunn/fzf/releases/download";

    pub fn download_url(version: &str, platform: &Platform) -> Option<String> {
        let target = Self::get_platform_suffix(platform)?;
        let ext = Self::get_archive_extension(platform);
        Some(format!(
            "{}/v{}/fzf-{}-{}.{}",
            Self::BASE_URL,
            version,
            version,
            target,
            ext
        ))
    }

    pub fn get_platform_suffix(platform: &Platform) -> Option<&'static str> {
        match (&platform.os, &platform.arch) {
            (Os::Windows, Arch::X86_64) => Some("windows_amd64"),
            (Os::Windows, Arch::Aarch64) => Some("windows_arm64"),
            (Os::MacOS, Arch::X86_64) => Some("darwin_amd64"),
            (Os::MacOS, Arch::Aarch64) => Some("darwin_arm64"),
            (Os::Linux, Arch::X86_64) => Some("linux_amd64"),
            (Os::Linux, Arch::Aarch64) => Some("linux_arm64"),
            (Os::Linux, Arch::Arm) => Some("linux_armv7"),
            _ => None,
        }
    }

    pub fn get_archive_extension(platform: &Platform) -> &'static str {
        match platform.os {
            Os::Windows => "zip",
            _ => "tar.gz",
        }
    }

    pub fn get_executable_name(platform: &Platform) -> &'static str {
        match platform.os {
            Os::Windows => "fzf.exe",
            _ => "fzf",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_download_url_linux_x64() {
        let platform = Platform::new(Os::Linux, Arch::X86_64);
        let url = FzfUrlBuilder::download_url("0.57.0", &platform);
        assert_eq!(
            url,
            Some("https://github.com/junegunn/fzf/releases/download/v0.57.0/fzf-0.57.0-linux_amd64.tar.gz".to_string())
        );
    }

    #[test]
    fn test_download_url_windows_x64() {
        let platform = Platform::new(Os::Windows, Arch::X86_64);
        let url = FzfUrlBuilder::download_url("0.57.0", &platform);
        assert_eq!(
            url,
            Some("https://github.com/junegunn/fzf/releases/download/v0.57.0/fzf-0.57.0-windows_amd64.zip".to_string())
        );
    }

    #[test]
    fn test_download_url_macos_arm64() {
        let platform = Platform::new(Os::MacOS, Arch::Aarch64);
        let url = FzfUrlBuilder::download_url("0.57.0", &platform);
        assert_eq!(
            url,
            Some("https://github.com/junegunn/fzf/releases/download/v0.57.0/fzf-0.57.0-darwin_arm64.tar.gz".to_string())
        );
    }

    #[test]
    fn test_executable_name() {
        assert_eq!(
            FzfUrlBuilder::get_executable_name(&Platform::new(Os::Windows, Arch::X86_64)),
            "fzf.exe"
        );
        assert_eq!(
            FzfUrlBuilder::get_executable_name(&Platform::new(Os::Linux, Arch::X86_64)),
            "fzf"
        );
    }
}
