//! Zig configuration

use serde::{Deserialize, Serialize};
use vx_runtime::Platform;

/// Zig configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ZigConfig {
    /// Default Zig version
    pub default_version: Option<String>,
    /// Install directory
    pub install_dir: Option<String>,
}

/// Zig URL builder for download URLs
pub struct ZigUrlBuilder;

impl ZigUrlBuilder {
    /// Generate download URL for Zig version
    /// Zig releases are hosted on ziglang.org
    /// URL format: https://ziglang.org/download/{version}/zig-{arch}-{os}-{version}.{ext}
    /// Example: https://ziglang.org/download/0.15.2/zig-x86_64-windows-0.15.2.zip
    pub fn download_url(version: &str, platform: &Platform) -> Option<String> {
        let (os_str, arch_str) = Self::get_platform_strings(platform);
        let ext = if platform.os == vx_runtime::Os::Windows {
            "zip"
        } else {
            "tar.xz"
        };

        // Format: zig-{arch}-{os}-{version}.{ext}
        // Note: Zig uses {arch}-{os} order, NOT {os}-{arch}
        Some(format!(
            "https://ziglang.org/download/{}/zig-{}-{}-{}.{}",
            version, arch_str, os_str, version, ext
        ))
    }

    /// Get platform strings for Zig downloads
    fn get_platform_strings(platform: &Platform) -> (&'static str, &'static str) {
        use vx_runtime::{Arch, Os};

        let os_str = match &platform.os {
            Os::Windows => "windows",
            Os::MacOS => "macos",
            Os::Linux => "linux",
            _ => "linux",
        };

        let arch_str = match &platform.arch {
            Arch::X86_64 => "x86_64",
            Arch::Aarch64 => "aarch64",
            Arch::X86 => "x86",
            Arch::Arm => "armv7a",
            _ => "x86_64",
        };

        (os_str, arch_str)
    }

    /// Get the archive directory name
    /// Format: zig-{arch}-{os}-{version}
    /// Example: zig-x86_64-linux-0.15.2
    pub fn get_archive_dir_name(version: &str, platform: &Platform) -> String {
        let (os_str, arch_str) = Self::get_platform_strings(platform);
        // Format: zig-{arch}-{os}-{version}
        format!("zig-{}-{}-{}", arch_str, os_str, version)
    }
}
