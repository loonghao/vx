//! URL builder and platform configuration for protoc
//!
//! protoc releases are available at: https://github.com/protocolbuffers/protobuf/releases
//! Download URL format: protoc-{version}-{platform}.zip
//! Examples:
//! - protoc-29.2-win64.zip
//! - protoc-29.2-linux-x86_64.zip
//! - protoc-29.2-osx-universal_binary.zip

use vx_runtime::{Arch, Os, Platform};

/// URL builder for protoc downloads
pub struct ProtocUrlBuilder;

impl ProtocUrlBuilder {
    /// Base URL for protoc releases
    const BASE_URL: &'static str = "https://github.com/protocolbuffers/protobuf/releases/download";

    /// Build the download URL for a specific version and platform
    pub fn download_url(version: &str, platform: &Platform) -> Option<String> {
        let platform_suffix = Self::get_platform_suffix(platform)?;
        // protoc uses 'v' prefix in release tags
        let version_tag = if version.starts_with('v') {
            version.to_string()
        } else {
            format!("v{}", version)
        };
        // Version in filename doesn't have 'v' prefix
        let version_num = version.trim_start_matches('v');
        Some(format!(
            "{}/{}/protoc-{}-{}.zip",
            Self::BASE_URL,
            version_tag,
            version_num,
            platform_suffix
        ))
    }

    /// Get the platform suffix for protoc downloads
    pub fn get_platform_suffix(platform: &Platform) -> Option<&'static str> {
        match (&platform.os, &platform.arch) {
            // Windows
            (Os::Windows, Arch::X86_64) => Some("win64"),
            (Os::Windows, Arch::X86) => Some("win32"),
            // macOS - universal binary
            (Os::MacOS, Arch::X86_64 | Arch::Aarch64) => Some("osx-universal_binary"),
            // Linux
            (Os::Linux, Arch::X86_64) => Some("linux-x86_64"),
            (Os::Linux, Arch::Aarch64) => Some("linux-aarch_64"),
            (Os::Linux, Arch::X86) => Some("linux-x86_32"),
            _ => None,
        }
    }

    /// Get the executable name for the platform
    pub fn get_executable_name(platform: &Platform) -> &'static str {
        match platform.os {
            Os::Windows => "protoc.exe",
            _ => "protoc",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_download_url_linux_x64() {
        let platform = Platform::new(Os::Linux, Arch::X86_64);
        let url = ProtocUrlBuilder::download_url("29.2", &platform);
        assert_eq!(
            url,
            Some(
                "https://github.com/protocolbuffers/protobuf/releases/download/v29.2/protoc-29.2-linux-x86_64.zip"
                    .to_string()
            )
        );
    }

    #[test]
    fn test_download_url_windows_x64() {
        let platform = Platform::new(Os::Windows, Arch::X86_64);
        let url = ProtocUrlBuilder::download_url("29.2", &platform);
        assert_eq!(
            url,
            Some(
                "https://github.com/protocolbuffers/protobuf/releases/download/v29.2/protoc-29.2-win64.zip"
                    .to_string()
            )
        );
    }

    #[test]
    fn test_download_url_macos_arm64() {
        let platform = Platform::new(Os::MacOS, Arch::Aarch64);
        let url = ProtocUrlBuilder::download_url("29.2", &platform);
        assert_eq!(
            url,
            Some(
                "https://github.com/protocolbuffers/protobuf/releases/download/v29.2/protoc-29.2-osx-universal_binary.zip"
                    .to_string()
            )
        );
    }

    #[test]
    fn test_download_url_with_v_prefix() {
        let platform = Platform::new(Os::Linux, Arch::X86_64);
        let url = ProtocUrlBuilder::download_url("v29.2", &platform);
        assert_eq!(
            url,
            Some(
                "https://github.com/protocolbuffers/protobuf/releases/download/v29.2/protoc-29.2-linux-x86_64.zip"
                    .to_string()
            )
        );
    }
}
