//! Dagu URL builder
//!
//! This module provides URL building functionality for Dagu downloads.
//! Dagu release assets follow the pattern:
//!   dagu_{version}_{os}_{arch}.tar.gz
//!
//! All platforms use .tar.gz format.

use vx_runtime::Platform;

/// URL builder for Dagu
pub struct DaguUrlBuilder;

impl DaguUrlBuilder {
    pub fn download_url(version: &str, platform: &Platform) -> Option<String> {
        let (os_name, arch_name) = match (&platform.os, &platform.arch) {
            (vx_runtime::Os::Windows, vx_runtime::Arch::X86_64) => ("windows", "amd64"),
            (vx_runtime::Os::Windows, vx_runtime::Arch::Aarch64) => ("windows", "arm64"),
            (vx_runtime::Os::MacOS, vx_runtime::Arch::X86_64) => ("darwin", "amd64"),
            (vx_runtime::Os::MacOS, vx_runtime::Arch::Aarch64) => ("darwin", "arm64"),
            (vx_runtime::Os::Linux, vx_runtime::Arch::X86_64) => ("linux", "amd64"),
            (vx_runtime::Os::Linux, vx_runtime::Arch::Aarch64) => ("linux", "arm64"),
            _ => return None,
        };

        Some(format!(
            "https://github.com/dagu-org/dagu/releases/download/v{}/dagu_{}_{}_{}.tar.gz",
            version, version, os_name, arch_name
        ))
    }
}
