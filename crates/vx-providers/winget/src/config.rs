//! Windows Package Manager configuration

use serde::{Deserialize, Serialize};
use vx_runtime::{Os, Platform};

/// winget configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WingetConfig {
    /// Default sources to use
    pub default_sources: Option<Vec<String>>,
}

impl WingetConfig {
    /// Get the executable name for the platform
    pub fn get_executable_name(platform: &Platform) -> &'static str {
        match &platform.os {
            Os::Windows => "winget.exe",
            _ => "winget", // Won't be used, but provide a fallback
        }
    }

    /// Check if the platform is supported
    pub fn is_platform_supported(platform: &Platform) -> bool {
        matches!(&platform.os, Os::Windows)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vx_runtime::Arch;

    #[test]
    fn test_executable_name_windows() {
        let platform = Platform::new(Os::Windows, Arch::X86_64);
        assert_eq!(WingetConfig::get_executable_name(&platform), "winget.exe");
    }

    #[test]
    fn test_platform_support() {
        let windows = Platform::new(Os::Windows, Arch::X86_64);
        let linux = Platform::new(Os::Linux, Arch::X86_64);
        let macos = Platform::new(Os::MacOs, Arch::Aarch64);

        assert!(WingetConfig::is_platform_supported(&windows));
        assert!(!WingetConfig::is_platform_supported(&linux));
        assert!(!WingetConfig::is_platform_supported(&macos));
    }
}
