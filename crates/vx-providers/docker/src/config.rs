//! Docker configuration

use serde::{Deserialize, Serialize};
use vx_runtime::Platform;

/// Docker configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DockerConfig {
    /// Default Docker version
    pub default_version: Option<String>,
    /// Install directory
    pub install_dir: Option<String>,
}

/// Docker URL builder for download URLs
pub struct DockerUrlBuilder;

impl DockerUrlBuilder {
    /// Generate download URL for Docker CLI version
    /// Docker CLI releases are available from download.docker.com
    ///
    /// Note: Docker Desktop requires separate installation on Windows.
    /// This provides the standalone Docker CLI binary for Linux and macOS only.
    pub fn download_url(version: &str, platform: &Platform) -> Option<String> {
        use vx_runtime::{Arch, Os};

        match (&platform.os, &platform.arch) {
            // Linux - Docker provides static binaries
            (Os::Linux, Arch::X86_64) => Some(format!(
                "https://download.docker.com/linux/static/stable/x86_64/docker-{}.tgz",
                version
            )),
            (Os::Linux, Arch::Aarch64) => Some(format!(
                "https://download.docker.com/linux/static/stable/aarch64/docker-{}.tgz",
                version
            )),
            // macOS - Docker CLI from download.docker.com
            (Os::MacOS, Arch::X86_64) => Some(format!(
                "https://download.docker.com/mac/static/stable/x86_64/docker-{}.tgz",
                version
            )),
            (Os::MacOS, Arch::Aarch64) => Some(format!(
                "https://download.docker.com/mac/static/stable/aarch64/docker-{}.tgz",
                version
            )),
            // Windows - Docker CLI is not available as standalone binary
            // Requires Docker Desktop installation
            (Os::Windows, _) => None,
            _ => None,
        }
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
