//! Java configuration

use serde::{Deserialize, Serialize};
use vx_runtime::Platform;

/// Java configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct JavaConfig {
    /// Default Java version
    pub default_version: Option<String>,
    /// Install directory
    pub install_dir: Option<String>,
}

/// Java URL builder for download URLs
/// Uses Eclipse Temurin (Adoptium) API
pub struct JavaUrlBuilder;

impl JavaUrlBuilder {
    /// Generate download URL for Java version using Adoptium API
    /// The version should be the major version (e.g., "21", "17", "11")
    pub fn download_url(version: &str, platform: &Platform) -> Option<String> {
        let (os_str, arch_str) = Self::get_platform_strings(platform);

        // Use Adoptium API v3 for binary downloads
        Some(format!(
            "https://api.adoptium.net/v3/binary/latest/{}/ga/{}/{}/jdk/hotspot/normal/eclipse?project=jdk",
            version, os_str, arch_str
        ))
    }

    /// Get platform strings for Adoptium downloads
    fn get_platform_strings(platform: &Platform) -> (&'static str, &'static str) {
        use vx_runtime::{Arch, Os};

        let os_str = match &platform.os {
            Os::Windows => "windows",
            Os::MacOS => "mac",
            Os::Linux => "linux",
            _ => "linux",
        };

        let arch_str = match &platform.arch {
            Arch::X86_64 => "x64",
            Arch::Aarch64 => "aarch64",
            Arch::X86 => "x86",
            Arch::Arm => "arm",
            _ => "x64",
        };

        (os_str, arch_str)
    }

    /// Get the archive directory pattern
    /// Temurin archives extract to jdk-<version>+<build> or similar
    pub fn get_archive_pattern() -> &'static str {
        "jdk-*"
    }
}
