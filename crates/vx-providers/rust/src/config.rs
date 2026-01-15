//! URL builder and platform configuration for Rust/Rustup
//!
//! Rustup releases are available at: https://static.rust-lang.org/rustup/dist/
//! Download URL format:
//! - Windows x64: https://static.rust-lang.org/rustup/dist/x86_64-pc-windows-msvc/rustup-init.exe
//! - Windows x86: https://static.rust-lang.org/rustup/dist/i686-pc-windows-msvc/rustup-init.exe
//! - macOS x64: https://static.rust-lang.org/rustup/dist/x86_64-apple-darwin/rustup-init
//! - macOS ARM64: https://static.rust-lang.org/rustup/dist/aarch64-apple-darwin/rustup-init
//! - Linux x64: https://static.rust-lang.org/rustup/dist/x86_64-unknown-linux-gnu/rustup-init
//! - Linux ARM64: https://static.rust-lang.org/rustup/dist/aarch64-unknown-linux-gnu/rustup-init

use vx_runtime::{Arch, Os, Platform};

/// URL builder for Rustup downloads
pub struct RustupUrlBuilder;

impl RustupUrlBuilder {
    /// Rustup distribution base URL
    const BASE_URL: &'static str = "https://static.rust-lang.org/rustup/dist";

    /// Get the platform target triple for rustup downloads
    pub fn get_target_triple(platform: &Platform) -> Option<&'static str> {
        match (&platform.os, &platform.arch) {
            (Os::Windows, Arch::X86_64) => Some("x86_64-pc-windows-msvc"),
            (Os::Windows, Arch::X86) => Some("i686-pc-windows-msvc"),
            (Os::MacOS, Arch::X86_64) => Some("x86_64-apple-darwin"),
            (Os::MacOS, Arch::Aarch64) => Some("aarch64-apple-darwin"),
            (Os::Linux, Arch::X86_64) => Some("x86_64-unknown-linux-gnu"),
            (Os::Linux, Arch::Aarch64) => Some("aarch64-unknown-linux-gnu"),
            (Os::Linux, Arch::Arm) => Some("arm-unknown-linux-gnueabihf"),
            _ => None,
        }
    }

    /// Get the executable filename for the platform
    pub fn get_executable_name(platform: &Platform) -> &'static str {
        match platform.os {
            Os::Windows => "rustup-init.exe",
            _ => "rustup-init",
        }
    }

    /// Build the download URL for rustup-init
    /// Note: rustup downloads are not versioned in the URL, always downloads latest
    pub fn download_url(platform: &Platform) -> Option<String> {
        let target = Self::get_target_triple(platform)?;
        let filename = Self::get_executable_name(platform);
        Some(format!("{}/{}/{}", Self::BASE_URL, target, filename))
    }

    /// Get the final rustup executable name (after rename from rustup-init)
    pub fn get_final_executable_name(platform: &Platform) -> &'static str {
        match platform.os {
            Os::Windows => "rustup.exe",
            _ => "rustup",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_download_url_windows_x64() {
        let platform = Platform {
            os: Os::Windows,
            arch: Arch::X86_64,
        };
        let url = RustupUrlBuilder::download_url(&platform);
        assert_eq!(
            url,
            Some(
                "https://static.rust-lang.org/rustup/dist/x86_64-pc-windows-msvc/rustup-init.exe"
                    .to_string()
            )
        );
    }

    #[test]
    fn test_download_url_macos_arm64() {
        let platform = Platform {
            os: Os::MacOS,
            arch: Arch::Aarch64,
        };
        let url = RustupUrlBuilder::download_url(&platform);
        assert_eq!(
            url,
            Some(
                "https://static.rust-lang.org/rustup/dist/aarch64-apple-darwin/rustup-init"
                    .to_string()
            )
        );
    }

    #[test]
    fn test_download_url_linux_x64() {
        let platform = Platform {
            os: Os::Linux,
            arch: Arch::X86_64,
        };
        let url = RustupUrlBuilder::download_url(&platform);
        assert_eq!(
            url,
            Some(
                "https://static.rust-lang.org/rustup/dist/x86_64-unknown-linux-gnu/rustup-init"
                    .to_string()
            )
        );
    }

    #[test]
    fn test_get_target_triple() {
        let win64 = Platform {
            os: Os::Windows,
            arch: Arch::X86_64,
        };
        assert_eq!(
            RustupUrlBuilder::get_target_triple(&win64),
            Some("x86_64-pc-windows-msvc")
        );
    }
}
