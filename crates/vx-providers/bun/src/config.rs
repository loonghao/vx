//! Bun configuration

use serde::{Deserialize, Serialize};

/// Bun configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BunConfig {
    /// Default Bun version
    pub default_version: Option<String>,
    /// Install directory
    pub install_dir: Option<String>,
}

/// Bun URL builder for download URLs
pub struct BunUrlBuilder;

impl BunUrlBuilder {
    /// Generate download URL for Bun version
    pub fn download_url(version: &str, platform: &str, arch: &str) -> Option<String> {
        let filename = Self::get_filename(platform, arch);
        Some(format!(
            "https://github.com/oven-sh/bun/releases/download/bun-v{}/{}",
            version, filename
        ))
    }

    /// Get platform-specific filename
    pub fn get_filename(platform: &str, arch: &str) -> String {
        match (platform, arch) {
            ("windows", "x86_64") => "bun-windows-x64.zip".to_string(),
            ("macos", "x86_64") => "bun-darwin-x64.zip".to_string(),
            ("macos", "aarch64") => "bun-darwin-aarch64.zip".to_string(),
            ("linux", "x86_64") => "bun-linux-x64.zip".to_string(),
            ("linux", "aarch64") => "bun-linux-aarch64.zip".to_string(),
            _ => "bun-linux-x64.zip".to_string(),
        }
    }

    /// Get current platform string
    pub fn get_platform_string() -> (&'static str, &'static str) {
        let platform = if cfg!(target_os = "windows") {
            "windows"
        } else if cfg!(target_os = "macos") {
            "macos"
        } else {
            "linux"
        };

        let arch = if cfg!(target_arch = "x86_64") {
            "x86_64"
        } else if cfg!(target_arch = "aarch64") {
            "aarch64"
        } else {
            "x86_64"
        };

        (platform, arch)
    }
}
