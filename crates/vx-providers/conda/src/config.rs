//! Conda configuration and URL building

use vx_runtime::platform::Platform;
use vx_runtime::{Arch, Os};

/// Conda URL builder for consistent download URL generation
#[derive(Debug, Clone, Default)]
pub struct CondaUrlBuilder;

impl CondaUrlBuilder {
    /// Generate download URL for Miniforge (Conda distribution)
    ///
    /// Format: `https://github.com/conda-forge/miniforge/releases/download/{version}/Miniforge3-{version}-{os}-{arch}.{ext}`
    pub fn conda_download_url(version: &str, platform: &Platform) -> Option<String> {
        let (os_str, arch_str, ext) = match (&platform.os, &platform.arch) {
            (Os::Windows, Arch::X86_64) => ("Windows", "x86_64", "exe"),
            (Os::MacOS, Arch::X86_64) => ("MacOSX", "x86_64", "sh"),
            (Os::MacOS, Arch::Aarch64) => ("MacOSX", "arm64", "sh"),
            (Os::Linux, Arch::X86_64) => ("Linux", "x86_64", "sh"),
            (Os::Linux, Arch::Aarch64) => ("Linux", "aarch64", "sh"),
            _ => return None,
        };

        Some(format!(
            "https://github.com/conda-forge/miniforge/releases/download/{}/Miniforge3-{}-{}-{}.{}",
            version, version, os_str, arch_str, ext
        ))
    }

    /// Generate download URL for Micromamba
    ///
    /// Format: `https://github.com/mamba-org/micromamba-releases/releases/download/{version}/micromamba-{platform}.tar.bz2`
    pub fn micromamba_download_url(version: &str, platform: &Platform) -> Option<String> {
        let platform_str = match (&platform.os, &platform.arch) {
            (Os::Windows, Arch::X86_64) => "win-64",
            (Os::MacOS, Arch::X86_64) => "osx-64",
            (Os::MacOS, Arch::Aarch64) => "osx-arm64",
            (Os::Linux, Arch::X86_64) => "linux-64",
            (Os::Linux, Arch::Aarch64) => "linux-aarch64",
            _ => return None,
        };

        Some(format!(
            "https://github.com/mamba-org/micromamba-releases/releases/download/{}/micromamba-{}.tar.bz2",
            version, platform_str
        ))
    }
}
