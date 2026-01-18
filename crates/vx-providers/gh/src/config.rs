//! GitHub CLI URL builder
//!
//! This module provides URL building functionality for GitHub CLI downloads.

use vx_runtime::Platform;

/// URL builder for GitHub CLI
pub struct GitHubUrlBuilder;

impl GitHubUrlBuilder {
    pub fn download_url(version: &str, platform: &Platform) -> String {
        let (platform_name, arch_name, extension) = match (&platform.os, &platform.arch) {
            (vx_runtime::Os::Windows, vx_runtime::Arch::X86_64) => ("windows", "amd64", "zip"),
            (vx_runtime::Os::Windows, vx_runtime::Arch::Aarch64) => ("windows", "arm64", "zip"),
            (vx_runtime::Os::MacOS, vx_runtime::Arch::X86_64) => ("macOS", "amd64", "zip"),
            (vx_runtime::Os::MacOS, vx_runtime::Arch::Aarch64) => ("macOS", "arm64", "zip"),
            (vx_runtime::Os::Linux, vx_runtime::Arch::X86_64) => ("linux", "amd64", "tar.gz"),
            (vx_runtime::Os::Linux, vx_runtime::Arch::Aarch64) => ("linux", "arm64", "tar.gz"),
            _ => return String::new(),
        };

        format!(
            "https://github.com/cli/cli/releases/download/v{}/gh_{}_{}_{}.{}",
            version, version, platform_name, arch_name, extension
        )
    }
}
