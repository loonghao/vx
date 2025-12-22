//! Helm configuration

use serde::{Deserialize, Serialize};
use vx_runtime::Platform;

/// Helm configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HelmConfig {
    /// Default Helm version
    pub default_version: Option<String>,
    /// Install directory
    pub install_dir: Option<String>,
}

/// Helm URL builder for download URLs
pub struct HelmUrlBuilder;

impl HelmUrlBuilder {
    /// Generate download URL for Helm version
    /// Helm releases are hosted on get.helm.sh
    pub fn download_url(version: &str, platform: &Platform) -> Option<String> {
        let (os_str, arch_str) = Self::get_platform_strings(platform);
        let ext = if platform.os == vx_runtime::Os::Windows {
            "zip"
        } else {
            "tar.gz"
        };

        Some(format!(
            "https://get.helm.sh/helm-v{}-{}-{}.{}",
            version, os_str, arch_str, ext
        ))
    }

    /// Get platform strings for Helm downloads
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

    /// Get the archive directory name
    pub fn get_archive_dir_name(platform: &Platform) -> String {
        let (os_str, arch_str) = Self::get_platform_strings(platform);
        format!("{}-{}", os_str, arch_str)
    }
}
