//! Tests for FFmpeg URL builder and configuration

use vx_provider_ffmpeg::{FfmpegBuild, FfmpegUrlBuilder};
use vx_runtime::{Arch, Os, Platform};

mod url_builder {
    use super::*;

    #[test]
    fn test_windows_x64_essentials() {
        let platform = Platform {
            os: Os::Windows,
            arch: Arch::X86_64,
        };
        let url = FfmpegUrlBuilder::download_url("7.0", &platform, FfmpegBuild::Essentials);
        assert_eq!(
            url,
            Some("https://www.gyan.dev/ffmpeg/builds/ffmpeg-release-essentials.zip".to_string())
        );
    }

    #[test]
    fn test_windows_x64_full() {
        let platform = Platform {
            os: Os::Windows,
            arch: Arch::X86_64,
        };
        let url = FfmpegUrlBuilder::download_url("7.0", &platform, FfmpegBuild::Full);
        assert_eq!(
            url,
            Some("https://www.gyan.dev/ffmpeg/builds/ffmpeg-release-full.zip".to_string())
        );
    }

    #[test]
    fn test_windows_arm64_not_supported() {
        let platform = Platform {
            os: Os::Windows,
            arch: Arch::Aarch64,
        };
        let url = FfmpegUrlBuilder::download_url("7.0", &platform, FfmpegBuild::Essentials);
        assert!(url.is_none());
    }

    #[test]
    fn test_macos_x64() {
        let platform = Platform {
            os: Os::MacOS,
            arch: Arch::X86_64,
        };
        let url = FfmpegUrlBuilder::download_url("7.0", &platform, FfmpegBuild::Essentials);
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
        let url = FfmpegUrlBuilder::download_url("7.0", &platform, FfmpegBuild::Essentials);
        assert_eq!(
            url,
            Some("https://evermeet.cx/ffmpeg/getrelease/ffmpeg/zip".to_string())
        );
    }

    #[test]
    fn test_linux_amd64() {
        let platform = Platform {
            os: Os::Linux,
            arch: Arch::X86_64,
        };
        let url = FfmpegUrlBuilder::download_url("7.0", &platform, FfmpegBuild::Essentials);
        assert_eq!(
            url,
            Some(
                "https://johnvansickle.com/ffmpeg/releases/ffmpeg-release-amd64-static.tar.xz"
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
        let url = FfmpegUrlBuilder::download_url("7.0", &platform, FfmpegBuild::Essentials);
        assert_eq!(
            url,
            Some(
                "https://johnvansickle.com/ffmpeg/releases/ffmpeg-release-arm64-static.tar.xz"
                    .to_string()
            )
        );
    }

    #[test]
    fn test_linux_armhf() {
        let platform = Platform {
            os: Os::Linux,
            arch: Arch::Arm,
        };
        let url = FfmpegUrlBuilder::download_url("7.0", &platform, FfmpegBuild::Essentials);
        assert_eq!(
            url,
            Some(
                "https://johnvansickle.com/ffmpeg/releases/ffmpeg-release-armhf-static.tar.xz"
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
