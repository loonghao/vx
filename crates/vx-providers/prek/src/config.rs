//! prek URL builder
//!
//! This module provides URL building functionality for prek downloads.
//! prek release assets follow the pattern:
//!   prek-{target_triple}.{ext}
//!
//! Where:
//!   - Linux/macOS: .tar.gz
//!   - Windows: .zip

use vx_runtime::Platform;

/// URL builder for prek
pub struct PrekUrlBuilder;

impl PrekUrlBuilder {
    pub fn download_url(version: &str, platform: &Platform) -> Option<String> {
        let (target_triple, ext) = match (&platform.os, &platform.arch) {
            (vx_runtime::Os::Windows, vx_runtime::Arch::X86_64) => {
                ("x86_64-pc-windows-msvc", "zip")
            }
            (vx_runtime::Os::Windows, vx_runtime::Arch::Aarch64) => {
                ("aarch64-pc-windows-msvc", "zip")
            }
            (vx_runtime::Os::MacOS, vx_runtime::Arch::X86_64) => ("x86_64-apple-darwin", "tar.gz"),
            (vx_runtime::Os::MacOS, vx_runtime::Arch::Aarch64) => {
                ("aarch64-apple-darwin", "tar.gz")
            }
            (vx_runtime::Os::Linux, vx_runtime::Arch::X86_64) => {
                ("x86_64-unknown-linux-gnu", "tar.gz")
            }
            (vx_runtime::Os::Linux, vx_runtime::Arch::Aarch64) => {
                ("aarch64-unknown-linux-gnu", "tar.gz")
            }
            _ => return None,
        };

        Some(format!(
            "https://github.com/j178/prek/releases/download/v{}/prek-{}.{}",
            version, target_triple, ext
        ))
    }
}
