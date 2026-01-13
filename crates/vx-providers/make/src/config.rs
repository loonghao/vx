//! URL builder and platform configuration for GNU Make
//!
//! On Windows, we use GNU Make for Windows from ezwinports or similar sources.
//! On Unix systems, make is typically pre-installed or available via system package manager.

use vx_runtime::{Arch, Os, Platform};

/// URL builder for Make downloads
pub struct MakeUrlBuilder;

impl MakeUrlBuilder {
    /// Build the download URL for a specific version and platform
    ///
    /// Windows: Uses pre-built binaries from GitHub releases (make-for-windows)
    /// Unix: Returns None (use system package manager)
    pub fn download_url(version: &str, platform: &Platform) -> Option<String> {
        match (&platform.os, &platform.arch) {
            // Windows - use GNU Make for Windows
            (Os::Windows, Arch::X86_64 | Arch::X86) => {
                // Use the make-for-windows project which provides pre-built binaries
                Some(format!(
                    "https://github.com/mbuilov/gnumake-windows/releases/download/{}/gnumake-{}-x64.zip",
                    version, version
                ))
            }
            // Unix systems should use system package manager
            _ => None,
        }
    }

    /// Get the executable name for the platform
    pub fn get_executable_name(platform: &Platform) -> &'static str {
        match platform.os {
            Os::Windows => "make.exe",
            _ => "make",
        }
    }

    /// Check if the platform is supported for binary downloads
    #[allow(dead_code)]
    pub fn is_binary_supported(platform: &Platform) -> bool {
        matches!(platform.os, Os::Windows)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_download_url_windows() {
        let platform = Platform {
            os: Os::Windows,
            arch: Arch::X86_64,
        };
        let url = MakeUrlBuilder::download_url("4.4.1", &platform);
        assert!(url.is_some());
        assert!(url.unwrap().contains("gnumake"));
    }

    #[test]
    fn test_download_url_linux_not_supported() {
        let platform = Platform {
            os: Os::Linux,
            arch: Arch::X86_64,
        };
        let url = MakeUrlBuilder::download_url("4.4.1", &platform);
        assert!(url.is_none());
    }
}
