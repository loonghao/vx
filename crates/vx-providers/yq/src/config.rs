//! URL builder and platform configuration for yq
//!
//! yq releases: https://github.com/mikefarah/yq/releases
//! Download URL format: https://github.com/mikefarah/yq/releases/download/v{version}/yq_{platform}
//!
//! Note: yq releases are direct binary downloads (not archives).

use vx_runtime::{Arch, Os, Platform};

pub struct YqUrlBuilder;

impl YqUrlBuilder {
    const BASE_URL: &'static str = "https://github.com/mikefarah/yq/releases/download";

    pub fn download_url(version: &str, platform: &Platform) -> Option<String> {
        let asset_name = Self::get_asset_name(platform)?;
        Some(format!("{}/v{}/{}", Self::BASE_URL, version, asset_name))
    }

    pub fn get_asset_name(platform: &Platform) -> Option<String> {
        match (&platform.os, &platform.arch) {
            (Os::Windows, Arch::X86_64) => Some("yq_windows_amd64.exe".to_string()),
            (Os::Windows, Arch::Aarch64) => Some("yq_windows_arm64.exe".to_string()),
            (Os::MacOS, Arch::X86_64) => Some("yq_darwin_amd64".to_string()),
            (Os::MacOS, Arch::Aarch64) => Some("yq_darwin_arm64".to_string()),
            (Os::Linux, Arch::X86_64) => Some("yq_linux_amd64".to_string()),
            (Os::Linux, Arch::Aarch64) => Some("yq_linux_arm64".to_string()),
            (Os::Linux, Arch::Arm) => Some("yq_linux_arm".to_string()),
            _ => None,
        }
    }

    pub fn get_executable_name(platform: &Platform) -> &'static str {
        match platform.os {
            Os::Windows => "yq.exe",
            _ => "yq",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_download_url_linux_x64() {
        let platform = Platform::new(Os::Linux, Arch::X86_64);
        let url = YqUrlBuilder::download_url("4.44.1", &platform);
        assert_eq!(
            url,
            Some(
                "https://github.com/mikefarah/yq/releases/download/v4.44.1/yq_linux_amd64"
                    .to_string()
            )
        );
    }

    #[test]
    fn test_download_url_windows_x64() {
        let platform = Platform::new(Os::Windows, Arch::X86_64);
        let url = YqUrlBuilder::download_url("4.44.1", &platform);
        assert_eq!(
            url,
            Some(
                "https://github.com/mikefarah/yq/releases/download/v4.44.1/yq_windows_amd64.exe"
                    .to_string()
            )
        );
    }

    #[test]
    fn test_download_url_macos_arm64() {
        let platform = Platform::new(Os::MacOS, Arch::Aarch64);
        let url = YqUrlBuilder::download_url("4.44.1", &platform);
        assert_eq!(
            url,
            Some(
                "https://github.com/mikefarah/yq/releases/download/v4.44.1/yq_darwin_arm64"
                    .to_string()
            )
        );
    }

    #[test]
    fn test_executable_name_windows() {
        let platform = Platform::new(Os::Windows, Arch::X86_64);
        assert_eq!(YqUrlBuilder::get_executable_name(&platform), "yq.exe");
    }

    #[test]
    fn test_executable_name_unix() {
        let platform = Platform::new(Os::Linux, Arch::X86_64);
        assert_eq!(YqUrlBuilder::get_executable_name(&platform), "yq");
    }
}
