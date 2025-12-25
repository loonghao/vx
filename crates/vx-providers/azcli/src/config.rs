//! Azure CLI configuration

use serde::{Deserialize, Serialize};
use vx_runtime::Platform;

/// Azure CLI configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AzCliConfig {
    /// Default Azure CLI version
    pub default_version: Option<String>,
    /// Install directory
    pub install_dir: Option<String>,
}

/// Azure CLI URL builder for download URLs
pub struct AzCliUrlBuilder;

impl AzCliUrlBuilder {
    /// Generate download URL for Azure CLI version
    /// Azure CLI releases are available from GitHub releases
    pub fn download_url(version: &str, platform: &Platform) -> Option<String> {
        use vx_runtime::{Arch, Os};

        match (&platform.os, &platform.arch) {
            // Linux - Azure CLI provides deb/rpm packages and pip install
            // We use the standalone zip from GitHub releases
            (Os::Linux, Arch::X86_64) => Some(format!(
                "https://github.com/Azure/azure-cli/releases/download/azure-cli-{}/azure-cli-{}-linux-x86_64.tar.gz",
                version, version
            )),
            (Os::Linux, Arch::Aarch64) => Some(format!(
                "https://github.com/Azure/azure-cli/releases/download/azure-cli-{}/azure-cli-{}-linux-aarch64.tar.gz",
                version, version
            )),
            // macOS
            (Os::MacOS, Arch::X86_64) => Some(format!(
                "https://github.com/Azure/azure-cli/releases/download/azure-cli-{}/azure-cli-{}-macos-x86_64.tar.gz",
                version, version
            )),
            (Os::MacOS, Arch::Aarch64) => Some(format!(
                "https://github.com/Azure/azure-cli/releases/download/azure-cli-{}/azure-cli-{}-macos-arm64.tar.gz",
                version, version
            )),
            // Windows - MSI installer
            (Os::Windows, Arch::X86_64) => Some(format!(
                "https://azcliprod.blob.core.windows.net/msi/azure-cli-{}-x64.msi",
                version
            )),
            (Os::Windows, Arch::Aarch64) => Some(format!(
                "https://azcliprod.blob.core.windows.net/msi/azure-cli-{}-arm64.msi",
                version
            )),
            _ => None,
        }
    }

    /// Get the archive/installer type for the platform
    pub fn archive_type(platform: &Platform) -> &'static str {
        use vx_runtime::Os;

        match &platform.os {
            Os::Windows => "msi",
            _ => "tar.gz",
        }
    }
}
