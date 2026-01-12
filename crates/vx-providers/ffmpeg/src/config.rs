//! URL builder and platform configuration for FFmpeg
//!
//! FFmpeg official doesn't provide prebuilt binaries, so we use trusted third-party sources:
//! - Windows: https://www.gyan.dev/ffmpeg/builds/
//! - macOS: https://evermeet.cx/ffmpeg/
//! - Linux: https://johnvansickle.com/ffmpeg/
//!
//! # Build Types
//!
//! - **Full**: Complete build with all codecs
//! - **Essentials**: Common codecs only (smaller download)
//! - **GPL**: Includes GPL-licensed codecs (x264, x265)
//! - **LGPL**: Only LGPL-licensed codecs

use vx_runtime::{Arch, Os, Platform};

/// FFmpeg build type
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum FfmpegBuild {
    /// Complete build with all codecs
    Full,
    /// Essential codecs only (smaller)
    #[default]
    Essentials,
    /// GPL version (includes x264, x265)
    Gpl,
    /// LGPL version (no GPL codecs)
    Lgpl,
}

/// URL builder for FFmpeg downloads
pub struct FfmpegUrlBuilder;

impl FfmpegUrlBuilder {
    /// Build the download URL for a specific version and platform
    ///
    /// Note: FFmpeg third-party sources have different versioning approaches:
    /// - gyan.dev (Windows): Uses "release" for latest stable
    /// - evermeet.cx (macOS): Uses "getrelease" for latest
    /// - johnvansickle.com (Linux): Uses "release" for latest stable
    pub fn download_url(version: &str, platform: &Platform, build: FfmpegBuild) -> Option<String> {
        match &platform.os {
            Os::Windows => Self::windows_url(version, platform, build),
            Os::MacOS => Self::macos_url(version, platform),
            Os::Linux => Self::linux_url(version, platform),
            _ => None,
        }
    }

    /// Get download URL for Windows (from gyan.dev)
    fn windows_url(_version: &str, platform: &Platform, build: FfmpegBuild) -> Option<String> {
        // gyan.dev only supports x86_64
        if platform.arch != Arch::X86_64 {
            return None;
        }

        let build_type = match build {
            FfmpegBuild::Full => "full",
            FfmpegBuild::Essentials => "essentials",
            FfmpegBuild::Gpl => "full",
            FfmpegBuild::Lgpl => "essentials",
        };

        // gyan.dev provides "release" builds (latest stable)
        // Format: ffmpeg-release-essentials.zip or ffmpeg-release-full.zip
        Some(format!(
            "https://www.gyan.dev/ffmpeg/builds/ffmpeg-release-{}.zip",
            build_type
        ))
    }

    /// Get download URL for macOS (from evermeet.cx)
    fn macos_url(_version: &str, platform: &Platform) -> Option<String> {
        // evermeet.cx supports both x86_64 and arm64
        match &platform.arch {
            Arch::X86_64 | Arch::Aarch64 => {
                // evermeet.cx provides universal binaries
                Some("https://evermeet.cx/ffmpeg/getrelease/ffmpeg/zip".to_string())
            }
            _ => None,
        }
    }

    /// Get download URL for Linux (from johnvansickle.com)
    fn linux_url(_version: &str, platform: &Platform) -> Option<String> {
        let arch = match &platform.arch {
            Arch::X86_64 => "amd64",
            Arch::Aarch64 => "arm64",
            Arch::Arm => "armhf",
            Arch::X86 => "i686",
            _ => return None,
        };

        // johnvansickle.com provides static builds
        Some(format!(
            "https://johnvansickle.com/ffmpeg/releases/ffmpeg-release-{}-static.tar.xz",
            arch
        ))
    }

    /// Get the archive extension for the platform
    pub fn get_archive_extension(platform: &Platform) -> &'static str {
        match &platform.os {
            Os::Windows => "zip",
            Os::Linux => "tar.xz",
            _ => "zip",
        }
    }

    /// Get the executable name for a given tool and platform
    pub fn get_executable_name(tool: &str, platform: &Platform) -> String {
        match &platform.os {
            Os::Windows => format!("{}.exe", tool),
            _ => tool.to_string(),
        }
    }

    /// Get the relative path to executable within the extracted archive
    ///
    /// Different platforms have different archive structures:
    /// - Windows (gyan.dev): ffmpeg-{version}-{build}/bin/ffmpeg.exe
    /// - macOS (evermeet.cx): ffmpeg (directly in archive)
    /// - Linux (johnvansickle.com): ffmpeg-{version}-{arch}-static/ffmpeg
    pub fn get_executable_relative_path(tool: &str, platform: &Platform) -> String {
        let exe_name = Self::get_executable_name(tool, platform);
        match &platform.os {
            Os::Windows => format!("bin/{}", exe_name),
            Os::MacOS => exe_name,
            Os::Linux => exe_name,
            _ => exe_name,
        }
    }
}
