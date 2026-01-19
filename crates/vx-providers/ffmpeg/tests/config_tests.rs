//! Tests for FFmpeg URL builder and configuration

use vx_provider_ffmpeg::{FfmpegBuild, FfmpegUrlBuilder};
use vx_runtime::{Arch, Os, Platform};

mod url_builder {
    use super::*;

    #[test]
    fn test_windows_x64_gpl() {
        let platform = Platform {
            os: Os::Windows,
            arch: Arch::X86_64,
        };
        let url = FfmpegUrlBuilder::download_url("7.0", &platform, FfmpegBuild::Gpl);
        assert_eq!(
            url,
            Some(
                "https://github.com/BtbN/FFmpeg-Builds/releases/download/latest/ffmpeg-n7.0-latest-win64-gpl-7.0.zip"
                    .to_string()
            )
        );
    }

    #[test]
    fn test_windows_x64_lgpl() {
        let platform = Platform {
            os: Os::Windows,
            arch: Arch::X86_64,
        };
        let url = FfmpegUrlBuilder::download_url("7.0", &platform, FfmpegBuild::Lgpl);
        assert_eq!(
            url,
            Some(
                "https://github.com/BtbN/FFmpeg-Builds/releases/download/latest/ffmpeg-n7.0-latest-win64-lgpl-7.0.zip"
                    .to_string()
            )
        );
    }

    #[test]
    fn test_windows_arm64() {
        let platform = Platform {
            os: Os::Windows,
            arch: Arch::Aarch64,
        };
        let url = FfmpegUrlBuilder::download_url("7.0", &platform, FfmpegBuild::Gpl);
        assert_eq!(
            url,
            Some(
                "https://github.com/BtbN/FFmpeg-Builds/releases/download/latest/ffmpeg-n7.0-latest-winarm64-gpl-7.0.zip"
                    .to_string()
            )
        );
    }

    #[test]
    fn test_macos_x64() {
        let platform = Platform {
            os: Os::MacOS,
            arch: Arch::X86_64,
        };
        let url = FfmpegUrlBuilder::download_url("7.0", &platform, FfmpegBuild::Gpl);
        assert_eq!(
            url,
            Some("https://evermeet.cx/ffmpeg/getrelease/ffmpeg/zip".to_string())
        );
    }

    #[test]
    fn test_macos_arm64() {
        let platform = Platform {
            os: Os::MacOS,
            arch: Arch::Aarch64,
        };
        let url = FfmpegUrlBuilder::download_url("7.0", &platform, FfmpegBuild::Gpl);
        assert_eq!(
            url,
            Some("https://evermeet.cx/ffmpeg/getrelease/ffmpeg/zip".to_string())
        );
    }

    #[test]
    fn test_linux_x64() {
        let platform = Platform {
            os: Os::Linux,
            arch: Arch::X86_64,
        };
        let url = FfmpegUrlBuilder::download_url("7.0", &platform, FfmpegBuild::Gpl);
        assert_eq!(
            url,
            Some(
                "https://github.com/BtbN/FFmpeg-Builds/releases/download/latest/ffmpeg-n7.0-latest-linux64-gpl-7.0.tar.xz"
                    .to_string()
            )
        );
    }

    #[test]
    fn test_linux_arm64() {
        let platform = Platform {
            os: Os::Linux,
            arch: Arch::Aarch64,
        };
        let url = FfmpegUrlBuilder::download_url("7.0", &platform, FfmpegBuild::Gpl);
        assert_eq!(
            url,
            Some(
                "https://github.com/BtbN/FFmpeg-Builds/releases/download/latest/ffmpeg-n7.0-latest-linuxarm64-gpl-7.0.tar.xz"
                    .to_string()
            )
        );
    }

    #[test]
    fn test_linux_arm_not_supported() {
        let platform = Platform {
            os: Os::Linux,
            arch: Arch::Arm,
        };
        let url = FfmpegUrlBuilder::download_url("7.0", &platform, FfmpegBuild::Gpl);
        // BtbN does not support armhf
        assert!(url.is_none());
    }

    #[test]
    fn test_master_version() {
        let platform = Platform {
            os: Os::Windows,
            arch: Arch::X86_64,
        };
        let url = FfmpegUrlBuilder::download_url("latest", &platform, FfmpegBuild::Gpl);
        assert_eq!(
            url,
            Some(
                "https://github.com/BtbN/FFmpeg-Builds/releases/download/latest/ffmpeg-master-latest-win64-gpl.zip"
                    .to_string()
            )
        );
    }
}

mod executable_name {
    use super::*;

    #[test]
    fn test_windows_executable_name() {
        let platform = Platform {
            os: Os::Windows,
            arch: Arch::X86_64,
        };
        assert_eq!(
            FfmpegUrlBuilder::get_executable_name("ffmpeg", &platform),
            "ffmpeg.exe"
        );
        assert_eq!(
            FfmpegUrlBuilder::get_executable_name("ffprobe", &platform),
            "ffprobe.exe"
        );
        assert_eq!(
            FfmpegUrlBuilder::get_executable_name("ffplay", &platform),
            "ffplay.exe"
        );
    }

    #[test]
    fn test_unix_executable_name() {
        let platform = Platform {
            os: Os::Linux,
            arch: Arch::X86_64,
        };
        assert_eq!(
            FfmpegUrlBuilder::get_executable_name("ffmpeg", &platform),
            "ffmpeg"
        );
        assert_eq!(
            FfmpegUrlBuilder::get_executable_name("ffprobe", &platform),
            "ffprobe"
        );
    }
}

mod executable_path {
    use super::*;

    #[test]
    fn test_windows_relative_path() {
        let platform = Platform {
            os: Os::Windows,
            arch: Arch::X86_64,
        };
        assert_eq!(
            FfmpegUrlBuilder::get_executable_relative_path("ffmpeg", &platform),
            "bin/ffmpeg.exe"
        );
    }

    #[test]
    fn test_linux_relative_path() {
        let platform = Platform {
            os: Os::Linux,
            arch: Arch::X86_64,
        };
        assert_eq!(
            FfmpegUrlBuilder::get_executable_relative_path("ffmpeg", &platform),
            "bin/ffmpeg"
        );
    }

    #[test]
    fn test_macos_relative_path() {
        let platform = Platform {
            os: Os::MacOS,
            arch: Arch::Aarch64,
        };
        // macOS uses evermeet.cx which puts ffmpeg at root
        assert_eq!(
            FfmpegUrlBuilder::get_executable_relative_path("ffmpeg", &platform),
            "ffmpeg"
        );
    }
}
