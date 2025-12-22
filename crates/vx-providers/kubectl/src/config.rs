//! kubectl configuration

use serde::{Deserialize, Serialize};
use vx_runtime::Platform;

/// kubectl configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct KubectlConfig {
    /// Default kubectl version
    pub default_version: Option<String>,
    /// Install directory
    pub install_dir: Option<String>,
}

/// kubectl URL builder for download URLs
pub struct KubectlUrlBuilder;

impl KubectlUrlBuilder {
    /// Generate download URL for kubectl version
    /// kubectl releases are hosted on dl.k8s.io
    pub fn download_url(version: &str, platform: &Platform) -> Option<String> {
        let (os_str, arch_str) = Self::get_platform_strings(platform);
        let ext = if platform.os == vx_runtime::Os::Windows {
            ".exe"
        } else {
            ""
        };

        Some(format!(
            "https://dl.k8s.io/release/v{}/bin/{}/{}/kubectl{}",
            version, os_str, arch_str, ext
        ))
    }

    /// Get platform strings for kubectl downloads
    fn get_platform_strings(platform: &Platform) -> (&'static str, &'static str) {
        use vx_runtime::{Arch, Os};

        let os_str = match &platform.os {
            Os::Windows => "windows",
            Os::MacOS => "darwin",
            Os::Linux => "linux",
            _ => "linux",
        };

        let arch_str = match &platform.arch {
            Arch::X86_64 => "amd64",
            Arch::Aarch64 => "arm64",
            Arch::X86 => "386",
            Arch::Arm => "arm",
            _ => "amd64",
        };

        (os_str, arch_str)
    }
}
