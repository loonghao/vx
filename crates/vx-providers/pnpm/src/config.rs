//! PNPM configuration

use serde::{Deserialize, Serialize};

/// PNPM configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PnpmConfig {
    /// Default PNPM version
    pub default_version: Option<String>,
    /// Store directory
    pub store_dir: Option<String>,
    /// Virtual store directory
    pub virtual_store_dir: Option<String>,
}

/// PNPM URL builder for download URLs
pub struct PnpmUrlBuilder;

impl PnpmUrlBuilder {
    /// Generate download URL for PNPM version
    pub fn download_url(version: &str) -> Option<String> {
        let filename = Self::get_filename(version);
        Some(format!(
            "https://github.com/pnpm/pnpm/releases/download/v{}/{}",
            version, filename
        ))
    }

    /// Get platform-specific filename
    pub fn get_filename(version: &str) -> String {
        let platform = Self::get_platform_string();
        if cfg!(windows) {
            format!("pnpm-{}-{}.exe", platform, version)
        } else {
            format!("pnpm-{}-{}", platform, version)
        }
    }

    /// Get platform string for PNPM downloads
    pub fn get_platform_string() -> String {
        match (std::env::consts::OS, std::env::consts::ARCH) {
            ("windows", "x86_64") => "win-x64".to_string(),
            ("windows", "aarch64") => "win-arm64".to_string(),
            ("macos", "x86_64") => "macos-x64".to_string(),
            ("macos", "aarch64") => "macos-arm64".to_string(),
            ("linux", "x86_64") => "linux-x64".to_string(),
            ("linux", "aarch64") => "linux-arm64".to_string(),
            _ => "linux-x64".to_string(),
        }
    }
}
