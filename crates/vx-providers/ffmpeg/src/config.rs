//! URL builder and platform configuration for FFmpeg
//!
//! FFmpeg official doesn't provide prebuilt binaries, so we use BtbN/FFmpeg-Builds:
//! - Source: https://github.com/BtbN/FFmpeg-Builds/releases
//! - Supports: Windows (x64, arm64), Linux (x64, arm64)
//! - macOS fallback: https://evermeet.cx/ffmpeg/
//!
//! # Build Types
//!
//! - **GPL**: Includes GPL-licensed codecs (x264, x265) - recommended
//! - **LGPL**: Only LGPL-licensed codecs

use vx_runtime::{Arch, Os, Platform};

/// FFmpeg build type
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum FfmpegBuild {
    /// GPL version (includes x264, x265) - recommended
    #[default]
    Gpl,
    /// LGPL version (no GPL codecs)
    Lgpl,
}

/// URL builder for FFmpeg downloads
pub struct FfmpegUrlBuilder;

impl FfmpegUrlBuilder {
    /// Build the download URL for a specific version and platform
    ///
    /// Uses BtbN/FFmpeg-Builds for Windows and Linux
    /// Falls back to evermeet.cx for macOS
    pub fn download_url(version: &str, platform: &Platform, build: FfmpegBuild) -> Option<String> {
        match &platform.os {
            Os::Windows => Self::btbn_url(version, platform, build),
            Os::Linux => Self::btbn_url(version, platform, build),
            Os::MacOS => Self::macos_url(platform),
            _ => None,
        }
    }

    /// Get download URL from BtbN/FFmpeg-Builds (Windows and Linux)
    ///
    /// Asset naming pattern:
    /// - Versioned: ffmpeg-n{version}-latest-{platform}-{license}-{version}.{ext}
    /// - Master: ffmpeg-master-latest-{platform}-{license}.{ext}
    ///
    /// Platform values:
    /// - win64, winarm64 (Windows)
    /// - linux64, linuxarm64 (Linux)
    fn btbn_url(version: &str, platform: &Platform, build: FfmpegBuild) -> Option<String> {
        let platform_str = match (&platform.os, &platform.arch) {
            (Os::Windows, Arch::X86_64) => "win64",
            (Os::Windows, Arch::Aarch64) => "winarm64",
            (Os::Linux, Arch::X86_64) => "linux64",
            (Os::Linux, Arch::Aarch64) => "linuxarm64",
            _ => return None,
        };

        let license = match build {
            FfmpegBuild::Gpl => "gpl",
            FfmpegBuild::Lgpl => "lgpl",
        };

        let ext = match &platform.os {
            Os::Windows => "zip",
            Os::Linux => "tar.xz",
            _ => "zip",
        };

        // For "latest" or "master", use the master build
        // For specific versions like "8.0", "7.1", use versioned builds
        let asset_name = if version == "latest" || version == "master" {
            format!("ffmpeg-master-latest-{}-{}.{}", platform_str, license, ext)
        } else {
            // Version format: "8.0" -> "n8.0"
            format!(
                "ffmpeg-n{}-latest-{}-{}-{}.{}",
                version, platform_str, license, version, ext
            )
        };

        Some(format!(
            "https://github.com/BtbN/FFmpeg-Builds/releases/download/latest/{}",
            asset_name
        ))
    }

    /// Get download URL for macOS (from evermeet.cx)
    fn macos_url(platform: &Platform) -> Option<String> {
        // evermeet.cx supports both x86_64 and arm64
        match &platform.arch {
            Arch::X86_64 | Arch::Aarch64 => {
                // evermeet.cx provides universal binaries
                Some("https://evermeet.cx/ffmpeg/getrelease/ffmpeg/zip".to_string())
            }
            _ => None,
        }
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
    /// BtbN builds have structure: ffmpeg-{version}-{platform}-{license}/bin/ffmpeg.exe
    /// After post_extract flattening: bin/ffmpeg.exe
    pub fn get_executable_relative_path(tool: &str, platform: &Platform) -> String {
        let exe_name = Self::get_executable_name(tool, platform);
        match &platform.os {
            Os::Windows | Os::Linux => format!("bin/{}", exe_name),
            Os::MacOS => exe_name,
            _ => exe_name,
        }
    }
}
