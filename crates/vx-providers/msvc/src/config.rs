//! Configuration types for MSVC Build Tools
//!
//! This module provides configuration types for MSVC installation.
//! The actual download and installation is handled by msvc-kit.

use vx_runtime::{Arch, Os, Platform};

/// MSVC installation configuration
#[derive(Debug, Clone)]
pub struct MsvcInstallConfig {
    /// MSVC toolset version (e.g., "14.40")
    pub msvc_version: String,
    /// Windows SDK version (e.g., "10.0.22621.0")
    pub sdk_version: Option<String>,
    /// Host architecture (x64, x86, arm64)
    pub host_arch: String,
    /// Target architecture (x64, x86, arm64)
    pub target_arch: String,
}

impl Default for MsvcInstallConfig {
    fn default() -> Self {
        Self {
            msvc_version: "14.42".to_string(),
            sdk_version: None,
            host_arch: "x64".to_string(),
            target_arch: "x64".to_string(),
        }
    }
}

impl MsvcInstallConfig {
    /// Create a new configuration with the specified MSVC version
    pub fn new(msvc_version: &str) -> Self {
        Self {
            msvc_version: msvc_version.to_string(),
            ..Default::default()
        }
    }

    /// Set the Windows SDK version
    pub fn with_sdk_version(mut self, version: &str) -> Self {
        self.sdk_version = Some(version.to_string());
        self
    }

    /// Set the host architecture
    pub fn with_host_arch(mut self, arch: &str) -> Self {
        self.host_arch = arch.to_string();
        self
    }

    /// Set the target architecture
    pub fn with_target_arch(mut self, arch: &str) -> Self {
        self.target_arch = arch.to_string();
        self
    }
}

/// Helper functions for platform detection
pub struct PlatformHelper;

impl PlatformHelper {
    /// Get the architecture string for MSVC packages
    pub fn get_arch_string(platform: &Platform) -> Option<&'static str> {
        match (&platform.os, &platform.arch) {
            (Os::Windows, Arch::X86_64) => Some("x64"),
            (Os::Windows, Arch::X86) => Some("x86"),
            (Os::Windows, Arch::Aarch64) => Some("arm64"),
            _ => None,
        }
    }

    /// Check if the platform is supported
    ///
    /// MSVC Build Tools only support Windows
    pub fn is_platform_supported(platform: &Platform) -> bool {
        matches!(
            (&platform.os, &platform.arch),
            (Os::Windows, Arch::X86_64) | (Os::Windows, Arch::X86) | (Os::Windows, Arch::Aarch64)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_install_config() {
        let config = MsvcInstallConfig::default();
        assert_eq!(config.msvc_version, "14.42");
        assert_eq!(config.host_arch, "x64");
        assert_eq!(config.target_arch, "x64");
        assert!(config.sdk_version.is_none());
    }

    #[test]
    fn test_install_config_builder() {
        let config = MsvcInstallConfig::new("14.40")
            .with_sdk_version("10.0.22621.0")
            .with_host_arch("x64")
            .with_target_arch("arm64");

        assert_eq!(config.msvc_version, "14.40");
        assert_eq!(config.sdk_version, Some("10.0.22621.0".to_string()));
        assert_eq!(config.target_arch, "arm64");
    }

    #[test]
    fn test_arch_string_x64() {
        let platform = Platform::new(Os::Windows, Arch::X86_64);
        assert_eq!(PlatformHelper::get_arch_string(&platform), Some("x64"));
    }

    #[test]
    fn test_arch_string_arm64() {
        let platform = Platform::new(Os::Windows, Arch::Aarch64);
        assert_eq!(PlatformHelper::get_arch_string(&platform), Some("arm64"));
    }

    #[test]
    fn test_platform_supported_windows() {
        let platform = Platform::new(Os::Windows, Arch::X86_64);
        assert!(PlatformHelper::is_platform_supported(&platform));
    }

    #[test]
    fn test_platform_unsupported_linux() {
        let platform = Platform::new(Os::Linux, Arch::X86_64);
        assert!(!PlatformHelper::is_platform_supported(&platform));
    }
}
