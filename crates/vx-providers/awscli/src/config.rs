//! AWS CLI configuration

use serde::{Deserialize, Serialize};
use vx_runtime::Platform;

/// AWS CLI configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AwsCliConfig {
    /// Default AWS CLI version
    pub default_version: Option<String>,
    /// Install directory
    pub install_dir: Option<String>,
}

/// AWS CLI URL builder for download URLs
pub struct AwsCliUrlBuilder;

impl AwsCliUrlBuilder {
    /// Generate download URL for AWS CLI v2 version
    /// AWS CLI v2 releases are available from awscli.amazonaws.com
    pub fn download_url(version: &str, platform: &Platform) -> Option<String> {
        use vx_runtime::{Arch, Os};

        match (&platform.os, &platform.arch) {
            // Linux x86_64
            (Os::Linux, Arch::X86_64) => Some(format!(
                "https://awscli.amazonaws.com/awscli-exe-linux-x86_64-{}.zip",
                version
            )),
            // Linux ARM64
            (Os::Linux, Arch::Aarch64) => Some(format!(
                "https://awscli.amazonaws.com/awscli-exe-linux-aarch64-{}.zip",
                version
            )),
            // macOS - universal binary (works on both Intel and Apple Silicon)
            (Os::MacOS, Arch::X86_64 | Arch::Aarch64) => Some(format!(
                "https://awscli.amazonaws.com/AWSCLIV2-{}.pkg",
                version
            )),
            // Windows x86_64
            (Os::Windows, Arch::X86_64) => Some(format!(
                "https://awscli.amazonaws.com/AWSCLIV2-{}.msi",
                version
            )),
            _ => None,
        }
    }

    /// Get the archive/installer type for the platform
    pub fn archive_type(platform: &Platform) -> &'static str {
        use vx_runtime::Os;

        match &platform.os {
            Os::Linux => "zip",
            Os::MacOS => "pkg",
            Os::Windows => "msi",
            _ => "zip",
        }
    }
}
