//! Deno configuration

use serde::{Deserialize, Serialize};
use vx_runtime::{Arch, Libc, Os, Platform};

/// Deno configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DenoConfig {
    /// Default Deno version
    pub default_version: Option<String>,
    /// Install directory
    pub install_dir: Option<String>,
}

/// Deno URL builder for download URLs
pub struct DenoUrlBuilder;

impl DenoUrlBuilder {
    /// Generate download URL for Deno version
    /// Deno releases are hosted on GitHub
    pub fn download_url(version: &str, platform: &Platform) -> Option<String> {
        let filename = Self::get_filename(platform)?;
        Some(format!(
            "https://github.com/denoland/deno/releases/download/v{}/{}",
            version, filename
        ))
    }

    /// Get platform-specific filename
    /// Deno provides binaries for major platforms
    pub fn get_filename(platform: &Platform) -> Option<String> {
        match (&platform.os, &platform.arch, &platform.libc) {
            // Windows
            (Os::Windows, Arch::X86_64, _) => Some("deno-x86_64-pc-windows-msvc.zip".to_string()),

            // macOS
            (Os::MacOS, Arch::X86_64, _) => Some("deno-x86_64-apple-darwin.zip".to_string()),
            (Os::MacOS, Arch::Aarch64, _) => Some("deno-aarch64-apple-darwin.zip".to_string()),

            // Linux GNU
            (Os::Linux, Arch::X86_64, Libc::Gnu) => {
                Some("deno-x86_64-unknown-linux-gnu.zip".to_string())
            }
            (Os::Linux, Arch::Aarch64, Libc::Gnu) => {
                Some("deno-aarch64-unknown-linux-gnu.zip".to_string())
            }

            // Linux MUSL - Deno doesn't provide musl builds, fall back to gnu
            // Users on musl systems should use compatibility layers
            (Os::Linux, Arch::X86_64, Libc::Musl) => {
                Some("deno-x86_64-unknown-linux-gnu.zip".to_string())
            }
            (Os::Linux, Arch::Aarch64, Libc::Musl) => {
                Some("deno-aarch64-unknown-linux-gnu.zip".to_string())
            }

            // Unsupported platforms
            _ => None,
        }
    }

    /// Get the archive directory name (Deno extracts directly, no subdirectory)
    pub fn get_archive_dir_name(_platform: &Platform) -> &'static str {
        // Deno zip contains just the binary at root level
        ""
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linux_platforms() {
        let platform = Platform::new(Os::Linux, Arch::X86_64);
        assert_eq!(
            DenoUrlBuilder::get_filename(&platform),
            Some("deno-x86_64-unknown-linux-gnu.zip".to_string())
        );

        let platform = Platform::new(Os::Linux, Arch::Aarch64);
        assert_eq!(
            DenoUrlBuilder::get_filename(&platform),
            Some("deno-aarch64-unknown-linux-gnu.zip".to_string())
        );
    }

    #[test]
    fn test_macos_platforms() {
        let platform = Platform::new(Os::MacOS, Arch::X86_64);
        assert_eq!(
            DenoUrlBuilder::get_filename(&platform),
            Some("deno-x86_64-apple-darwin.zip".to_string())
        );

        let platform = Platform::new(Os::MacOS, Arch::Aarch64);
        assert_eq!(
            DenoUrlBuilder::get_filename(&platform),
            Some("deno-aarch64-apple-darwin.zip".to_string())
        );
    }

    #[test]
    fn test_windows_platforms() {
        let platform = Platform::new(Os::Windows, Arch::X86_64);
        assert_eq!(
            DenoUrlBuilder::get_filename(&platform),
            Some("deno-x86_64-pc-windows-msvc.zip".to_string())
        );
    }
}
