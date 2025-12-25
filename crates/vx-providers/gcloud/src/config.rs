//! Google Cloud CLI configuration

use serde::{Deserialize, Serialize};
use vx_runtime::Platform;

/// Google Cloud CLI configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GcloudConfig {
    /// Default gcloud version
    pub default_version: Option<String>,
    /// Install directory
    pub install_dir: Option<String>,
}

/// Google Cloud CLI URL builder for download URLs
pub struct GcloudUrlBuilder;

impl GcloudUrlBuilder {
    /// Generate download URL for Google Cloud SDK version
    /// Google Cloud SDK releases are available from dl.google.com
    pub fn download_url(version: &str, platform: &Platform) -> Option<String> {
        use vx_runtime::{Arch, Os};

        let (os_str, arch_str, ext) = match (&platform.os, &platform.arch) {
            // Linux x86_64
            (Os::Linux, Arch::X86_64) => ("linux", "x86_64", "tar.gz"),
            // Linux ARM64
            (Os::Linux, Arch::Aarch64) => ("linux", "arm", "tar.gz"),
            // macOS x86_64
            (Os::MacOS, Arch::X86_64) => ("darwin", "x86_64", "tar.gz"),
            // macOS ARM64
            (Os::MacOS, Arch::Aarch64) => ("darwin", "arm", "tar.gz"),
            // Windows x86_64
            (Os::Windows, Arch::X86_64) => ("windows", "x86_64", "zip"),
            // Windows ARM64 (bundled version)
            (Os::Windows, Arch::Aarch64) => ("windows", "x86_64", "zip"),
            _ => return None,
        };

        Some(format!(
            "https://dl.google.com/dl/cloudsdk/channels/rapid/downloads/google-cloud-cli-{}-{}-{}.{}",
            version, os_str, arch_str, ext
        ))
    }

    /// Get the archive type for the platform
    pub fn archive_type(platform: &Platform) -> &'static str {
        use vx_runtime::Os;

        match &platform.os {
            Os::Windows => "zip",
            _ => "tar.gz",
        }
    }
}
