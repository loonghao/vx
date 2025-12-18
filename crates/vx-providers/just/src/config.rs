//! URL builder and platform configuration for Just
//!
//! Just releases are available at: https://github.com/casey/just/releases
//! Download URL format: https://github.com/casey/just/releases/download/{version}/just-{version}-{target}.{ext}

use vx_runtime::{Arch, Os, Platform};

/// URL builder for Just downloads
pub struct JustUrlBuilder;

impl JustUrlBuilder {
    /// Base URL for Just releases
    const BASE_URL: &'static str = "https://github.com/casey/just/releases/download";

    /// Build the download URL for a specific version and platform
    pub fn download_url(version: &str, platform: &Platform) -> Option<String> {
        let target = Self::get_target_triple(platform)?;
        let ext = Self::get_archive_extension(platform);
        Some(format!(
            "{}/{}/just-{}-{}.{}",
            Self::BASE_URL,
            version,
            version,
            target,
            ext
        ))
    }

    /// Get the target triple for the platform
    pub fn get_target_triple(platform: &Platform) -> Option<String> {
        match (&platform.os, &platform.arch) {
            // Windows
            (Os::Windows, Arch::X86_64) => Some("x86_64-pc-windows-msvc".to_string()),
            (Os::Windows, Arch::Aarch64) => Some("aarch64-pc-windows-msvc".to_string()),

            // macOS
            (Os::MacOS, Arch::X86_64) => Some("x86_64-apple-darwin".to_string()),
            (Os::MacOS, Arch::Aarch64) => Some("aarch64-apple-darwin".to_string()),

            // Linux (using musl for better compatibility)
            (Os::Linux, Arch::X86_64) => Some("x86_64-unknown-linux-musl".to_string()),
            (Os::Linux, Arch::Aarch64) => Some("aarch64-unknown-linux-musl".to_string()),
            (Os::Linux, Arch::Arm) => Some("arm-unknown-linux-musleabihf".to_string()),

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
            Os::Windows => "just.exe",
            _ => "just",
        }
    }

    /// Get the archive directory name (Just extracts directly, no subdirectory)
    pub fn get_archive_dir_name(_version: &str, _platform: &Platform) -> String {
        // Just archives extract files directly without a subdirectory
        String::new()
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
        let url = JustUrlBuilder::download_url("1.45.0", &platform);
        assert_eq!(
            url,
            Some("https://github.com/casey/just/releases/download/1.45.0/just-1.45.0-x86_64-unknown-linux-musl.tar.gz".to_string())
        );
    }

    #[test]
    fn test_download_url_windows_x64() {
        let platform = Platform {
            os: Os::Windows,
            arch: Arch::X86_64,
        };
        let url = JustUrlBuilder::download_url("1.45.0", &platform);
        assert_eq!(
            url,
            Some("https://github.com/casey/just/releases/download/1.45.0/just-1.45.0-x86_64-pc-windows-msvc.zip".to_string())
        );
    }

    #[test]
    fn test_download_url_macos_arm64() {
        let platform = Platform {
            os: Os::MacOS,
            arch: Arch::Aarch64,
        };
        let url = JustUrlBuilder::download_url("1.45.0", &platform);
        assert_eq!(
            url,
            Some("https://github.com/casey/just/releases/download/1.45.0/just-1.45.0-aarch64-apple-darwin.tar.gz".to_string())
        );
    }
}
