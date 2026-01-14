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

    /// Generate download URL for VSCode version.
    ///
    /// VS Code uses platform-specific identifiers. Only Windows uses the `-archive` suffix.
    /// For other platforms the base platform id already points to an archive download.
    ///
    /// URL format:
    /// - Windows: https://update.code.visualstudio.com/{version}/{platform}-archive/stable
    /// - macOS/Linux: https://update.code.visualstudio.com/{version}/{platform}/stable
    ///
    /// Note: We add a URL fragment as an extension hint for our downloader. Fragments are not
    /// sent to the server, but they help us classify archive types when the URL has no suffix.
    pub fn download_url(version: &str, platform: &Platform) -> Option<String> {
        let platform_str = Self::get_platform_string(platform);

        let (path, hint) = match platform.os {
            vx_runtime::Os::Windows => (format!("{}-archive", platform_str), "#.zip"),
            vx_runtime::Os::MacOS => (platform_str, "#.zip"),
            _ => (platform_str, "#.tar.gz"),
        };

        Some(format!(
            "https://update.code.visualstudio.com/{}/{}/stable{}",
            version, path, hint
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
