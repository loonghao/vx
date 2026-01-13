//! Rust configuration

use serde::{Deserialize, Serialize};

/// Rust configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RustConfig {
    /// Default Rust version/channel
    pub default_channel: Option<String>,
    /// Default target
    pub default_target: Option<String>,
    /// RUSTUP_HOME override
    pub rustup_home: Option<String>,
    /// CARGO_HOME override
    pub cargo_home: Option<String>,
}

impl Default for RustConfig {
    fn default() -> Self {
        Self {
            default_channel: Some("stable".to_string()),
            default_target: None,
            rustup_home: None,
            cargo_home: None,
        }
    }
}

/// Rust URL builder for download URLs
pub struct RustUrlBuilder;

impl RustUrlBuilder {
    /// Generate download URL for Rust version
    ///
    /// Uses `.tar.gz` format for all platforms (including Windows) as it's
    /// universally supported by our archive extraction logic.
    /// Note: Rust official releases provide both `.tar.gz` and `.tar.xz` for Windows.
    pub fn download_url(version: &str) -> Option<String> {
        let platform = Self::get_platform_string();
        // Use tar.gz for all platforms - Windows also has tar.gz downloads available
        // from static.rust-lang.org, and our extractor supports it natively
        Some(format!(
            "https://static.rust-lang.org/dist/rust-{}-{}.tar.gz",
            version, platform
        ))
    }

    /// Get rustup download URL for a specific version and platform
    pub fn rustup_url(version: &str) -> String {
        let platform = Self::get_platform_string();
        let exe_suffix = if cfg!(windows) { ".exe" } else { "" };
        // Use archive URL for specific versions
        format!(
            "https://static.rust-lang.org/rustup/archive/{}/{}/rustup-init{}",
            version, platform, exe_suffix
        )
    }

    /// Get platform string for downloads
    pub fn get_platform_string() -> String {
        if cfg!(target_os = "windows") {
            if cfg!(target_arch = "x86_64") {
                "x86_64-pc-windows-msvc".to_string()
            } else if cfg!(target_arch = "x86") {
                "i686-pc-windows-msvc".to_string()
            } else if cfg!(target_arch = "aarch64") {
                "aarch64-pc-windows-msvc".to_string()
            } else {
                "x86_64-pc-windows-msvc".to_string()
            }
        } else if cfg!(target_os = "macos") {
            if cfg!(target_arch = "x86_64") {
                "x86_64-apple-darwin".to_string()
            } else if cfg!(target_arch = "aarch64") {
                "aarch64-apple-darwin".to_string()
            } else {
                "x86_64-apple-darwin".to_string()
            }
        } else if cfg!(target_os = "linux") {
            if cfg!(target_arch = "x86_64") {
                "x86_64-unknown-linux-gnu".to_string()
            } else if cfg!(target_arch = "aarch64") {
                "aarch64-unknown-linux-gnu".to_string()
            } else {
                "x86_64-unknown-linux-gnu".to_string()
            }
        } else {
            "x86_64-unknown-linux-gnu".to_string()
        }
    }
}
