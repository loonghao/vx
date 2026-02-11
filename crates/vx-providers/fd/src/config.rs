//! URL builder and platform configuration for fd
//!
//! fd releases: https://github.com/sharkdp/fd/releases
//! URL format: https://github.com/sharkdp/fd/releases/download/v{version}/fd-v{version}-{target}.{ext}

use vx_runtime::{Arch, Os, Platform};

pub struct FdUrlBuilder;

impl FdUrlBuilder {
    const BASE_URL: &'static str = "https://github.com/sharkdp/fd/releases/download";

    pub fn download_url(version: &str, platform: &Platform) -> Option<String> {
        let target = Self::get_target_triple(platform)?;
        let ext = Self::get_archive_extension(platform);
        Some(format!(
            "{}/v{}/fd-v{}-{}.{}",
            Self::BASE_URL,
            version,
            version,
            target,
            ext
        ))
    }

    pub fn get_target_triple(platform: &Platform) -> Option<&'static str> {
        match (&platform.os, &platform.arch) {
            (Os::Windows, Arch::X86_64) => Some("x86_64-pc-windows-msvc"),
            (Os::MacOS, Arch::X86_64) => Some("x86_64-apple-darwin"),
            (Os::MacOS, Arch::Aarch64) => Some("aarch64-apple-darwin"),
            (Os::Linux, Arch::X86_64) => Some("x86_64-unknown-linux-musl"),
            (Os::Linux, Arch::Aarch64) => Some("aarch64-unknown-linux-gnu"),
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
            Os::Windows => "fd.exe",
            _ => "fd",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_download_url_linux_x64() {
        let platform = Platform::new(Os::Linux, Arch::X86_64);
        let url = FdUrlBuilder::download_url("10.2.0", &platform);
        assert_eq!(
            url,
            Some("https://github.com/sharkdp/fd/releases/download/v10.2.0/fd-v10.2.0-x86_64-unknown-linux-musl.tar.gz".to_string())
        );
    }

    #[test]
    fn test_download_url_windows_x64() {
        let platform = Platform::new(Os::Windows, Arch::X86_64);
        let url = FdUrlBuilder::download_url("10.2.0", &platform);
        assert_eq!(
            url,
            Some("https://github.com/sharkdp/fd/releases/download/v10.2.0/fd-v10.2.0-x86_64-pc-windows-msvc.zip".to_string())
        );
    }

    #[test]
    fn test_download_url_macos_arm64() {
        let platform = Platform::new(Os::MacOS, Arch::Aarch64);
        let url = FdUrlBuilder::download_url("10.2.0", &platform);
        assert_eq!(
            url,
            Some("https://github.com/sharkdp/fd/releases/download/v10.2.0/fd-v10.2.0-aarch64-apple-darwin.tar.gz".to_string())
        );
    }
}
