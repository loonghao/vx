//! Terraform configuration

use serde::{Deserialize, Serialize};
use vx_runtime::Platform;

/// Terraform configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TerraformConfig {
    /// Default Terraform version
    pub default_version: Option<String>,
    /// Install directory
    pub install_dir: Option<String>,
}

/// Terraform URL builder for download URLs
pub struct TerraformUrlBuilder;

impl TerraformUrlBuilder {
    /// Generate download URL for Terraform version
    /// Terraform releases are hosted on releases.hashicorp.com
    pub fn download_url(version: &str, platform: &Platform) -> Option<String> {
        let (os_str, arch_str) = Self::get_platform_strings(platform);

        Some(format!(
            "https://releases.hashicorp.com/terraform/{}/terraform_{}_{}_{}.zip",
            version, version, os_str, arch_str
        ))
    }

    /// Get platform strings for Terraform downloads
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
