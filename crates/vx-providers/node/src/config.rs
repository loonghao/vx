//! Node.js configuration and URL building
//!
//! This module provides Node.js-specific configuration,
//! including URL building and platform detection.

use vx_runtime::Platform;

/// Node.js URL builder for consistent download URL generation
pub struct NodeUrlBuilder;

impl NodeUrlBuilder {
    /// Generate download URL for Node.js version
    pub fn download_url(version: &str, platform: &Platform) -> Option<String> {
        let filename = Self::get_filename(version, platform);
        Some(format!("https://nodejs.org/dist/v{}/{}", version, filename))
    }

    /// Get platform-specific filename
    pub fn get_filename(version: &str, platform: &Platform) -> String {
        let platform_str = Self::get_platform_string(platform);
        if platform.os == vx_runtime::Os::Windows {
            format!("node-v{}-{}.zip", version, platform_str)
        } else {
            format!("node-v{}-{}.tar.gz", version, platform_str)
        }
    }

    /// Get platform string for Node.js downloads
    pub fn get_platform_string(platform: &Platform) -> String {
        use vx_runtime::{Arch, Os};

        match (&platform.os, &platform.arch) {
            (Os::Windows, Arch::X86_64) => "win-x64".to_string(),
            (Os::Windows, Arch::X86) => "win-x86".to_string(),
            (Os::Windows, Arch::Aarch64) => "win-arm64".to_string(),
            (Os::MacOS, Arch::X86_64) => "darwin-x64".to_string(),
            (Os::MacOS, Arch::Aarch64) => "darwin-arm64".to_string(),
            (Os::Linux, Arch::X86_64) => "linux-x64".to_string(),
            (Os::Linux, Arch::X86) => "linux-x86".to_string(),
            (Os::Linux, Arch::Aarch64) => "linux-arm64".to_string(),
            (Os::Linux, Arch::Arm) => "linux-armv7l".to_string(),
            _ => "linux-x64".to_string(), // Default fallback
        }
    }

    /// Get the executable name for the current platform
    pub fn executable_name(runtime_name: &str, platform: &Platform) -> String {
        if platform.os == vx_runtime::Os::Windows {
            format!("{}.exe", runtime_name)
        } else {
            runtime_name.to_string()
        }
    }
}
