//! Deno configuration

use serde::{Deserialize, Serialize};
use vx_runtime::Platform;

/// Deno configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DenoConfig {
    /// Default Deno version
    pub default_version: Option<String>,
    /// Install directory
    pub install_dir: Option<String>,
}

/// Deno URL builder for download URLs
pub struct DenoUrlBuilder;

impl DenoUrlBuilder {
    /// Generate download URL for Deno version
    /// Deno releases are hosted on GitHub
    pub fn download_url(version: &str, platform: &Platform) -> Option<String> {
        let filename = Self::get_filename(platform);
        Some(format!(
            "https://github.com/denoland/deno/releases/download/v{}/{}",
            version, filename
        ))
    }

    /// Get platform-specific filename
    pub fn get_filename(platform: &Platform) -> String {
        use vx_runtime::{Arch, Os};

        match (&platform.os, &platform.arch) {
            (Os::Windows, Arch::X86_64) => "deno-x86_64-pc-windows-msvc.zip".to_string(),
            (Os::MacOS, Arch::X86_64) => "deno-x86_64-apple-darwin.zip".to_string(),
            (Os::MacOS, Arch::Aarch64) => "deno-aarch64-apple-darwin.zip".to_string(),
            (Os::Linux, Arch::X86_64) => "deno-x86_64-unknown-linux-gnu.zip".to_string(),
            (Os::Linux, Arch::Aarch64) => "deno-aarch64-unknown-linux-gnu.zip".to_string(),
            _ => "deno-x86_64-unknown-linux-gnu.zip".to_string(),
        }
    }

    /// Get the archive directory name (Deno extracts directly, no subdirectory)
    pub fn get_archive_dir_name(_platform: &Platform) -> &'static str {
        // Deno zip contains just the binary at root level
        ""
    }
}
