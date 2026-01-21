//! URL builder and platform configuration for NASM
//!
//! NASM releases are available at: https://www.nasm.us/pub/nasm/releasebuilds/
//! Download URL format:
//! - Windows x64: nasm-{version}-win64.zip
//! - Windows x86: nasm-{version}-win32.zip
//! - macOS x64: nasm-{version}-macosx.zip (older versions)

use vx_runtime::{Arch, Os, Platform};

/// URL builder for NASM downloads
pub struct NasmUrlBuilder;

impl NasmUrlBuilder {
    /// NASM releases base URL
    const BASE_URL: &'static str = "https://www.nasm.us/pub/nasm/releasebuilds";

    /// Build the download URL for a specific version and platform
    pub fn download_url(version: &str, platform: &Platform) -> Option<String> {
        let filename = Self::get_filename(version, platform)?;
        Some(format!("{}/{}/{}", Self::BASE_URL, version, filename))
    }

    /// Get the download filename for the platform
    pub fn get_filename(version: &str, platform: &Platform) -> Option<String> {
        match (&platform.os, &platform.arch) {
            (Os::Windows, Arch::X86_64) => Some(format!("nasm-{}-win64.zip", version)),
            (Os::Windows, Arch::X86) => Some(format!("nasm-{}-win32.zip", version)),
            (Os::MacOS, Arch::X86_64) => Some(format!("nasm-{}-macosx.zip", version)),
            // Linux requires source compilation
            _ => None,
        }
    }

    /// Get the archive directory name after extraction
    pub fn get_archive_dir_name(version: &str, platform: &Platform) -> Option<String> {
        match (&platform.os, &platform.arch) {
            (Os::Windows, Arch::X86_64) => Some(format!("nasm-{}", version)),
            (Os::Windows, Arch::X86) => Some(format!("nasm-{}", version)),
            (Os::MacOS, _) => Some(format!("nasm-{}", version)),
            _ => None,
        }
    }

    /// Get the executable name for the platform
    pub fn get_executable_name(platform: &Platform) -> &'static str {
        match platform.os {
            Os::Windows => "nasm.exe",
            _ => "nasm",
        }
    }

    /// Check if the platform is supported for binary downloads
    pub fn is_binary_supported(platform: &Platform) -> bool {
        matches!(
            (&platform.os, &platform.arch),
            (Os::Windows, Arch::X86_64) | (Os::Windows, Arch::X86) | (Os::MacOS, Arch::X86_64)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_download_url_windows_x64() {
        let platform = Platform::new(Os::Windows, Arch::X86_64);
        let url = NasmUrlBuilder::download_url("2.16.01", &platform);
        assert_eq!(
            url,
            Some(
                "https://www.nasm.us/pub/nasm/releasebuilds/2.16.01/nasm-2.16.01-win64.zip"
                    .to_string()
            )
        );
    }

    #[test]
    fn test_download_url_windows_x86() {
        let platform = Platform::new(Os::Windows, Arch::X86);
        let url = NasmUrlBuilder::download_url("2.16.01", &platform);
        assert_eq!(
            url,
            Some(
                "https://www.nasm.us/pub/nasm/releasebuilds/2.16.01/nasm-2.16.01-win32.zip"
                    .to_string()
            )
        );
    }

    #[test]
    fn test_download_url_macos() {
        let platform = Platform::new(Os::MacOS, Arch::X86_64);
        let url = NasmUrlBuilder::download_url("2.16.01", &platform);
        assert_eq!(
            url,
            Some(
                "https://www.nasm.us/pub/nasm/releasebuilds/2.16.01/nasm-2.16.01-macosx.zip"
                    .to_string()
            )
        );
    }

    #[test]
    fn test_download_url_linux_not_supported() {
        let platform = Platform::new(Os::Linux, Arch::X86_64);
        let url = NasmUrlBuilder::download_url("2.16.01", &platform);
        assert_eq!(url, None);
    }

    #[test]
    fn test_get_archive_dir_name() {
        let platform = Platform::new(Os::Windows, Arch::X86_64);
        assert_eq!(
            NasmUrlBuilder::get_archive_dir_name("2.16.01", &platform),
            Some("nasm-2.16.01".to_string())
        );
    }

    #[test]
    fn test_is_binary_supported() {
        let win64 = Platform::new(Os::Windows, Arch::X86_64);
        let linux = Platform::new(Os::Linux, Arch::X86_64);
        assert!(NasmUrlBuilder::is_binary_supported(&win64));
        assert!(!NasmUrlBuilder::is_binary_supported(&linux));
    }
}
