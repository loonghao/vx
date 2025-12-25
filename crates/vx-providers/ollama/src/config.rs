//! URL builder and platform configuration for Ollama
//!
//! Ollama releases are available at: https://github.com/ollama/ollama/releases
//!
//! Download URL formats:
//! - Windows: ollama-windows-amd64.zip, ollama-windows-arm64.zip
//! - macOS: ollama-darwin.tgz
//! - Linux: ollama-linux-amd64.tgz, ollama-linux-arm64.tgz

use vx_runtime::{Arch, Os, Platform};

/// URL builder for Ollama downloads
pub struct OllamaUrlBuilder;

impl OllamaUrlBuilder {
    /// Base URL for Ollama releases
    const BASE_URL: &'static str = "https://github.com/ollama/ollama/releases/download";

    /// Build the download URL for a specific version and platform
    ///
    /// # Arguments
    /// * `version` - Version string (with or without 'v' prefix)
    /// * `platform` - Target platform
    ///
    /// # Returns
    /// Download URL if the platform is supported, None otherwise
    pub fn download_url(version: &str, platform: &Platform) -> Option<String> {
        let target = Self::get_target_string(platform)?;
        let ext = Self::get_archive_extension(platform);

        // Ensure version has 'v' prefix for the URL
        let version_tag = if version.starts_with('v') {
            version.to_string()
        } else {
            format!("v{}", version)
        };

        Some(format!(
            "{}/{}/ollama-{}.{}",
            Self::BASE_URL,
            version_tag,
            target,
            ext
        ))
    }

    /// Get the target string for the platform
    ///
    /// Ollama uses simple platform names like:
    /// - darwin (macOS, universal binary)
    /// - linux-amd64, linux-arm64
    /// - windows-amd64, windows-arm64
    pub fn get_target_string(platform: &Platform) -> Option<&'static str> {
        match (&platform.os, &platform.arch) {
            // Windows
            (Os::Windows, Arch::X86_64) => Some("windows-amd64"),
            (Os::Windows, Arch::Aarch64) => Some("windows-arm64"),

            // macOS - universal binary
            (Os::MacOS, Arch::X86_64 | Arch::Aarch64) => Some("darwin"),

            // Linux
            (Os::Linux, Arch::X86_64) => Some("linux-amd64"),
            (Os::Linux, Arch::Aarch64) => Some("linux-arm64"),

            _ => None,
        }
    }

    /// Get the archive extension for the platform
    pub fn get_archive_extension(platform: &Platform) -> &'static str {
        match platform.os {
            Os::Windows => "zip",
            _ => "tgz",
        }
    }

    /// Get the executable name for the platform
    pub fn get_executable_name(platform: &Platform) -> &'static str {
        match platform.os {
            Os::Windows => "ollama.exe",
            _ => "ollama",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_download_url_linux_x64() {
        let platform = Platform {
            os: Os::Linux,
            arch: Arch::X86_64,
        };
        let url = OllamaUrlBuilder::download_url("0.13.5", &platform);
        assert!(url.is_some());
        let url = url.unwrap();
        assert_eq!(
            url,
            "https://github.com/ollama/ollama/releases/download/v0.13.5/ollama-linux-amd64.tgz"
        );
    }

    #[test]
    fn test_download_url_linux_arm64() {
        let platform = Platform {
            os: Os::Linux,
            arch: Arch::Aarch64,
        };
        let url = OllamaUrlBuilder::download_url("0.13.5", &platform);
        assert!(url.is_some());
        let url = url.unwrap();
        assert_eq!(
            url,
            "https://github.com/ollama/ollama/releases/download/v0.13.5/ollama-linux-arm64.tgz"
        );
    }

    #[test]
    fn test_download_url_windows_x64() {
        let platform = Platform {
            os: Os::Windows,
            arch: Arch::X86_64,
        };
        let url = OllamaUrlBuilder::download_url("0.13.5", &platform);
        assert!(url.is_some());
        let url = url.unwrap();
        assert_eq!(
            url,
            "https://github.com/ollama/ollama/releases/download/v0.13.5/ollama-windows-amd64.zip"
        );
    }

    #[test]
    fn test_download_url_windows_arm64() {
        let platform = Platform {
            os: Os::Windows,
            arch: Arch::Aarch64,
        };
        let url = OllamaUrlBuilder::download_url("0.13.5", &platform);
        assert!(url.is_some());
        let url = url.unwrap();
        assert_eq!(
            url,
            "https://github.com/ollama/ollama/releases/download/v0.13.5/ollama-windows-arm64.zip"
        );
    }

    #[test]
    fn test_download_url_macos_x64() {
        let platform = Platform {
            os: Os::MacOS,
            arch: Arch::X86_64,
        };
        let url = OllamaUrlBuilder::download_url("0.13.5", &platform);
        assert!(url.is_some());
        let url = url.unwrap();
        assert_eq!(
            url,
            "https://github.com/ollama/ollama/releases/download/v0.13.5/ollama-darwin.tgz"
        );
    }

    #[test]
    fn test_download_url_macos_arm64() {
        let platform = Platform {
            os: Os::MacOS,
            arch: Arch::Aarch64,
        };
        let url = OllamaUrlBuilder::download_url("0.13.5", &platform);
        assert!(url.is_some());
        let url = url.unwrap();
        // macOS uses universal binary
        assert_eq!(
            url,
            "https://github.com/ollama/ollama/releases/download/v0.13.5/ollama-darwin.tgz"
        );
    }

    #[test]
    fn test_download_url_with_v_prefix() {
        let platform = Platform {
            os: Os::Linux,
            arch: Arch::X86_64,
        };
        let url = OllamaUrlBuilder::download_url("v0.13.5", &platform);
        assert!(url.is_some());
        let url = url.unwrap();
        // Should not double the 'v' prefix
        assert_eq!(
            url,
            "https://github.com/ollama/ollama/releases/download/v0.13.5/ollama-linux-amd64.tgz"
        );
    }

    #[test]
    fn test_archive_extension() {
        assert_eq!(
            OllamaUrlBuilder::get_archive_extension(&Platform {
                os: Os::Windows,
                arch: Arch::X86_64
            }),
            "zip"
        );
        assert_eq!(
            OllamaUrlBuilder::get_archive_extension(&Platform {
                os: Os::Linux,
                arch: Arch::X86_64
            }),
            "tgz"
        );
        assert_eq!(
            OllamaUrlBuilder::get_archive_extension(&Platform {
                os: Os::MacOS,
                arch: Arch::Aarch64
            }),
            "tgz"
        );
    }

    #[test]
    fn test_executable_name() {
        assert_eq!(
            OllamaUrlBuilder::get_executable_name(&Platform {
                os: Os::Windows,
                arch: Arch::X86_64
            }),
            "ollama.exe"
        );
        assert_eq!(
            OllamaUrlBuilder::get_executable_name(&Platform {
                os: Os::Linux,
                arch: Arch::X86_64
            }),
            "ollama"
        );
        assert_eq!(
            OllamaUrlBuilder::get_executable_name(&Platform {
                os: Os::MacOS,
                arch: Arch::Aarch64
            }),
            "ollama"
        );
    }
}
