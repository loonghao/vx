//! VSCode configuration and URL building
//!
//! This module provides VSCode-specific configuration,
//! including URL building and platform detection.
//!
//! VSCode download URLs redirect to the actual file:
//! - Archive: https://update.code.visualstudio.com/{version}/{platform}-archive/stable
//!   -> redirects to: https://vscode.download.prss.microsoft.com/.../VSCode-{platform}-{version}.zip

use vx_runtime::Platform;

/// VSCode URL builder for consistent download URL generation
#[derive(Debug, Clone, Default)]
pub struct VscodeUrlBuilder;

impl VscodeUrlBuilder {
    pub fn new() -> Self {
        Self
    }

    /// Generate download URL for VSCode version
    /// URL format: https://update.code.visualstudio.com/{version}/{platform}-archive/stable
    /// This URL redirects to a .zip or .tar.gz file
    pub fn download_url(version: &str, platform: &Platform) -> Option<String> {
        let platform_str = Self::get_platform_string(platform);
        // Use archive format for portable installation
        // Add .zip extension hint for the downloader to recognize it as an archive
        Some(format!(
            "https://update.code.visualstudio.com/{}/{}-archive/stable#.zip",
            version, platform_str
        ))
    }

    /// Get platform string for VSCode downloads
    pub fn get_platform_string(platform: &Platform) -> String {
        use vx_runtime::{Arch, Os};

        match (&platform.os, &platform.arch) {
            // Windows
            (Os::Windows, Arch::X86_64) => "win32-x64".to_string(),
            (Os::Windows, Arch::X86) => "win32".to_string(),
            (Os::Windows, Arch::Aarch64) => "win32-arm64".to_string(),
            // macOS
            (Os::MacOS, Arch::X86_64) => "darwin-x64".to_string(),
            (Os::MacOS, Arch::Aarch64) => "darwin-arm64".to_string(),
            // Linux
            (Os::Linux, Arch::X86_64) => "linux-x64".to_string(),
            (Os::Linux, Arch::Aarch64) => "linux-arm64".to_string(),
            (Os::Linux, Arch::Arm) => "linux-armhf".to_string(),
            // Default fallback
            _ => "linux-x64".to_string(),
        }
    }

    /// Get file extension based on platform
    pub fn get_extension(platform: &Platform) -> &'static str {
        use vx_runtime::Os;

        match platform.os {
            Os::Windows => "zip",
            Os::MacOS => "zip",
            _ => "tar.gz",
        }
    }

    /// Get the expected archive directory name after extraction
    pub fn get_archive_dir_name(platform: &Platform) -> String {
        use vx_runtime::Os;

        match platform.os {
            Os::Windows => "VSCode".to_string(),
            Os::MacOS => "Visual Studio Code.app".to_string(),
            _ => "VSCode-linux-x64".to_string(), // Linux extracts to this
        }
    }
}
