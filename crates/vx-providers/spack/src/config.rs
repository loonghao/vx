//! URL builder and platform configuration for Spack
//!
//! Spack releases are available at: https://github.com/spack/spack/releases
//!
//! Download URL format:
//! - Source tarball: https://github.com/spack/spack/releases/download/v{version}/spack-{version}.tar.gz
//!
//! Note: Spack is a Python-based tool that uses Unix shell scripts.
//! It only supports Linux and macOS. Windows users should use WSL.

use vx_runtime::{Os, Platform};

/// URL builder for Spack downloads
pub struct SpackUrlBuilder;

impl SpackUrlBuilder {
    /// Base URL for Spack releases
    const BASE_URL: &'static str = "https://github.com/spack/spack/releases/download";

    /// Build the download URL for a specific version
    ///
    /// # Arguments
    /// * `version` - Version string (with or without 'v' prefix)
    ///
    /// # Returns
    /// Download URL for the source tarball
    pub fn download_url(version: &str) -> Option<String> {
        // Ensure version has 'v' prefix for the URL
        let version_tag = if version.starts_with('v') {
            version.to_string()
        } else {
            format!("v{}", version)
        };

        // Strip 'v' prefix for the filename
        let version_num = version.strip_prefix('v').unwrap_or(version);

        Some(format!(
            "{}/{}/spack-{}.tar.gz",
            Self::BASE_URL,
            version_tag,
            version_num
        ))
    }

    /// Get the archive extension
    ///
    /// Spack always uses tar.gz format
    pub fn get_archive_extension() -> &'static str {
        "tar.gz"
    }

    /// Get the executable name for the platform
    ///
    /// On Windows, the spack script doesn't have an extension,
    /// but it needs to be run with Python or through WSL.
    pub fn get_executable_name(platform: &Platform) -> &'static str {
        match platform.os {
            Os::Windows => "spack",
            _ => "spack",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vx_runtime::Arch;

    #[test]
    fn test_download_url_without_v_prefix() {
        let url = SpackUrlBuilder::download_url("1.1.0");
        assert!(url.is_some());
        let url = url.unwrap();
        assert_eq!(
            url,
            "https://github.com/spack/spack/releases/download/v1.1.0/spack-1.1.0.tar.gz"
        );
    }

    #[test]
    fn test_download_url_with_v_prefix() {
        let url = SpackUrlBuilder::download_url("v1.1.0");
        assert!(url.is_some());
        let url = url.unwrap();
        // Should not double the 'v' prefix
        assert_eq!(
            url,
            "https://github.com/spack/spack/releases/download/v1.1.0/spack-1.1.0.tar.gz"
        );
    }

    #[test]
    fn test_download_url_older_version() {
        let url = SpackUrlBuilder::download_url("0.23.0");
        assert!(url.is_some());
        let url = url.unwrap();
        assert_eq!(
            url,
            "https://github.com/spack/spack/releases/download/v0.23.0/spack-0.23.0.tar.gz"
        );
    }

    #[test]
    fn test_archive_extension() {
        assert_eq!(SpackUrlBuilder::get_archive_extension(), "tar.gz");
    }

    #[test]
    fn test_executable_name_linux() {
        let platform = Platform {
            os: Os::Linux,
            arch: Arch::X86_64,
        };
        assert_eq!(SpackUrlBuilder::get_executable_name(&platform), "spack");
    }

    #[test]
    fn test_executable_name_macos() {
        let platform = Platform {
            os: Os::MacOS,
            arch: Arch::Aarch64,
        };
        assert_eq!(SpackUrlBuilder::get_executable_name(&platform), "spack");
    }

    #[test]
    fn test_executable_name_windows() {
        let platform = Platform {
            os: Os::Windows,
            arch: Arch::X86_64,
        };
        assert_eq!(SpackUrlBuilder::get_executable_name(&platform), "spack");
    }
}
