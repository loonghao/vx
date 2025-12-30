//! Python configuration and URL building
//!
//! Uses python-build-standalone from Astral for portable Python distributions.
//! https://github.com/astral-sh/python-build-standalone
//!
//! Release naming format:
//! cpython-{python_version}+{release_date}-{platform}-{variant}.tar.gz
//!
//! Example:
//! cpython-3.12.8+20251217-x86_64-pc-windows-msvc-shared-install_only.tar.gz

use vx_runtime::Platform;

/// Python URL builder for consistent download URL generation
#[derive(Debug, Clone, Default)]
pub struct PythonUrlBuilder;

impl PythonUrlBuilder {
    pub fn new() -> Self {
        Self
    }

    /// Generate download URL for Python version with specific release date
    ///
    /// URL format:
    /// https://github.com/astral-sh/python-build-standalone/releases/download/{release_date}/{filename}
    ///
    /// Filename format:
    /// cpython-{version}+{release_date}-{platform}-{variant}-install_only.tar.gz
    pub fn download_url_with_date(
        version: &str,
        release_date: &str,
        platform: &Platform,
    ) -> Option<String> {
        let filename = Self::get_filename_with_date(version, release_date, platform)?;
        Some(format!(
            "https://github.com/astral-sh/python-build-standalone/releases/download/{}/{}",
            release_date, filename
        ))
    }

    /// Get platform-specific filename with release date
    ///
    /// Format: cpython-{version}+{release_date}-{platform}-{variant}-install_only.tar.gz
    pub fn get_filename_with_date(
        version: &str,
        release_date: &str,
        platform: &Platform,
    ) -> Option<String> {
        let platform_str = Self::get_platform_string(platform);
        let variant = Self::get_variant(platform);

        Some(format!(
            "cpython-{}+{}-{}-{}-install_only.tar.gz",
            version, release_date, platform_str, variant
        ))
    }

    /// Get platform string for downloads
    ///
    /// python-build-standalone uses standard Rust target triples
    pub fn get_platform_string(platform: &Platform) -> &'static str {
        use vx_runtime::{Arch, Os};

        match (&platform.os, &platform.arch) {
            (Os::Windows, Arch::X86_64) => "x86_64-pc-windows-msvc",
            (Os::Windows, Arch::X86) => "i686-pc-windows-msvc",
            (Os::MacOS, Arch::X86_64) => "x86_64-apple-darwin",
            (Os::MacOS, Arch::Aarch64) => "aarch64-apple-darwin",
            (Os::Linux, Arch::X86_64) => "x86_64-unknown-linux-gnu",
            (Os::Linux, Arch::Aarch64) => "aarch64-unknown-linux-gnu",
            _ => "x86_64-unknown-linux-gnu", // Default fallback
        }
    }

    /// Get build variant for the platform
    ///
    /// - Windows: shared (dynamic linking)
    /// - macOS/Linux: pgo+lto (optimized build)
    pub fn get_variant(platform: &Platform) -> &'static str {
        use vx_runtime::Os;

        match &platform.os {
            Os::Windows => "shared",
            _ => "pgo+lto",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vx_runtime::{Arch, Os};

    #[test]
    fn test_platform_string_windows() {
        let platform = Platform {
            os: Os::Windows,
            arch: Arch::X86_64,
        };
        assert_eq!(
            PythonUrlBuilder::get_platform_string(&platform),
            "x86_64-pc-windows-msvc"
        );
    }

    #[test]
    fn test_platform_string_macos_arm() {
        let platform = Platform {
            os: Os::MacOS,
            arch: Arch::Aarch64,
        };
        assert_eq!(
            PythonUrlBuilder::get_platform_string(&platform),
            "aarch64-apple-darwin"
        );
    }

    #[test]
    fn test_platform_string_linux() {
        let platform = Platform {
            os: Os::Linux,
            arch: Arch::X86_64,
        };
        assert_eq!(
            PythonUrlBuilder::get_platform_string(&platform),
            "x86_64-unknown-linux-gnu"
        );
    }

    #[test]
    fn test_filename_windows() {
        let platform = Platform {
            os: Os::Windows,
            arch: Arch::X86_64,
        };
        let filename =
            PythonUrlBuilder::get_filename_with_date("3.12.8", "20251217", &platform).unwrap();
        assert!(filename.contains("cpython-3.12.8"));
        assert!(filename.contains("20251217"));
        assert!(filename.contains("x86_64-pc-windows-msvc"));
        assert!(filename.contains("shared"));
        assert!(filename.ends_with(".tar.gz"));
    }

    #[test]
    fn test_filename_linux() {
        let platform = Platform {
            os: Os::Linux,
            arch: Arch::X86_64,
        };
        let filename =
            PythonUrlBuilder::get_filename_with_date("3.12.8", "20251217", &platform).unwrap();
        assert!(filename.contains("cpython-3.12.8"));
        assert!(filename.contains("x86_64-unknown-linux-gnu"));
        assert!(filename.contains("pgo+lto"));
    }

    #[test]
    fn test_download_url_with_date() {
        let platform = Platform {
            os: Os::Linux,
            arch: Arch::X86_64,
        };
        let url =
            PythonUrlBuilder::download_url_with_date("3.12.8", "20251217", &platform).unwrap();
        assert!(url.starts_with(
            "https://github.com/astral-sh/python-build-standalone/releases/download/20251217/"
        ));
        assert!(url.contains("cpython-3.12.8"));
    }

    #[test]
    fn test_download_url_python37() {
        let platform = Platform {
            os: Os::Linux,
            arch: Arch::X86_64,
        };
        // Python 3.7 uses older release dates
        let url =
            PythonUrlBuilder::download_url_with_date("3.7.17", "20230826", &platform).unwrap();
        assert!(url.contains("20230826"));
        assert!(url.contains("cpython-3.7.17"));
    }
}
