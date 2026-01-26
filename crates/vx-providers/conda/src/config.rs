//! Conda configuration and URL building
//!
//! This module provides Conda-specific configuration,
//! including URL building and platform detection.

use vx_runtime::Platform;

/// Conda URL builder for consistent download URL generation
#[derive(Debug, Clone, Default)]
pub struct CondaUrlBuilder;

impl CondaUrlBuilder {
    pub fn new() -> Self {
        Self
    }

    /// Generate download URL for Miniforge (Conda distribution)
    ///
    /// Miniforge download URLs format:
    /// https://github.com/conda-forge/miniforge/releases/download/{version}/Miniforge3-{version}-{os}-{arch}.{ext}
    pub fn conda_download_url(version: &str, platform: &Platform) -> Option<String> {
        let filename = Self::get_miniforge_filename(version, platform)?;
        Some(format!(
            "https://github.com/conda-forge/miniforge/releases/download/{}/{}",
            version, filename
        ))
    }

    /// Generate download URL for Micromamba
    ///
    /// Micromamba download URLs format:
    /// - Linux/macOS: https://github.com/mamba-org/micromamba-releases/releases/download/{version}/micromamba-{platform}.tar.bz2
    /// - Windows: https://github.com/mamba-org/micromamba-releases/releases/download/{version}/micromamba-win-64.tar.bz2
    ///
    /// We use tar.bz2 format for all platforms as it extracts to a consistent structure.
    pub fn micromamba_download_url(version: &str, platform: &Platform) -> Option<String> {
        let platform_str = Self::get_micromamba_platform_string(platform)?;
        // Use tar.bz2 format which extracts to bin/micromamba (unix) or Library/bin/micromamba.exe (windows)
        Some(format!(
            "https://github.com/mamba-org/micromamba-releases/releases/download/{}/micromamba-{}.tar.bz2",
            version, platform_str
        ))
    }

    /// Get Miniforge filename for the platform
    fn get_miniforge_filename(version: &str, platform: &Platform) -> Option<String> {
        use vx_runtime::{Arch, Os};

        let (os_str, arch_str, ext) = match (&platform.os, &platform.arch) {
            (Os::Windows, Arch::X86_64) => ("Windows", "x86_64", "exe"),
            (Os::MacOS, Arch::X86_64) => ("MacOSX", "x86_64", "sh"),
            (Os::MacOS, Arch::Aarch64) => ("MacOSX", "arm64", "sh"),
            (Os::Linux, Arch::X86_64) => ("Linux", "x86_64", "sh"),
            (Os::Linux, Arch::Aarch64) => ("Linux", "aarch64", "sh"),
            _ => return None,
        };

        Some(format!(
            "Miniforge3-{}-{}-{}.{}",
            version, os_str, arch_str, ext
        ))
    }

    /// Get platform string for Micromamba downloads
    fn get_micromamba_platform_string(platform: &Platform) -> Option<String> {
        use vx_runtime::{Arch, Os};

        match (&platform.os, &platform.arch) {
            (Os::Windows, Arch::X86_64) => Some("win-64".to_string()),
            (Os::MacOS, Arch::X86_64) => Some("osx-64".to_string()),
            (Os::MacOS, Arch::Aarch64) => Some("osx-arm64".to_string()),
            (Os::Linux, Arch::X86_64) => Some("linux-64".to_string()),
            (Os::Linux, Arch::Aarch64) => Some("linux-aarch64".to_string()),
            _ => None,
        }
    }

    /// Get the conda executable path within the Miniforge installation
    pub fn get_conda_executable_path(platform: &Platform) -> String {
        use vx_runtime::Os;

        match &platform.os {
            Os::Windows => "Scripts\\conda.exe".to_string(),
            _ => "bin/conda".to_string(),
        }
    }

    /// Get the mamba executable path within the Miniforge installation
    pub fn get_mamba_executable_path(platform: &Platform) -> String {
        use vx_runtime::Os;

        match &platform.os {
            Os::Windows => "Scripts\\mamba.exe".to_string(),
            _ => "bin/mamba".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vx_runtime::{Arch, Os};

    #[test]
    fn test_conda_download_url_linux() {
        let platform = Platform {
            os: Os::Linux,
            arch: Arch::X86_64,
        };
        let url = CondaUrlBuilder::conda_download_url("24.3.0-0", &platform);
        assert!(url.is_some());
        assert!(url.unwrap().contains("Miniforge3-24.3.0-0-Linux-x86_64.sh"));
    }

    #[test]
    fn test_conda_download_url_windows() {
        let platform = Platform {
            os: Os::Windows,
            arch: Arch::X86_64,
        };
        let url = CondaUrlBuilder::conda_download_url("24.3.0-0", &platform);
        assert!(url.is_some());
        assert!(url.unwrap().contains("Miniforge3-24.3.0-0-Windows-x86_64.exe"));
    }

    #[test]
    fn test_conda_download_url_macos_arm() {
        let platform = Platform {
            os: Os::MacOS,
            arch: Arch::Aarch64,
        };
        let url = CondaUrlBuilder::conda_download_url("24.3.0-0", &platform);
        assert!(url.is_some());
        assert!(url.unwrap().contains("Miniforge3-24.3.0-0-MacOSX-arm64.sh"));
    }

    #[test]
    fn test_micromamba_download_url_linux() {
        let platform = Platform {
            os: Os::Linux,
            arch: Arch::X86_64,
        };
        let url = CondaUrlBuilder::micromamba_download_url("2.5.0-1", &platform);
        assert!(url.is_some());
        let url = url.unwrap();
        assert!(url.contains("micromamba-linux-64.tar.bz2"));
        assert!(url.contains("2.5.0-1"));
    }

    #[test]
    fn test_micromamba_download_url_windows() {
        let platform = Platform {
            os: Os::Windows,
            arch: Arch::X86_64,
        };
        let url = CondaUrlBuilder::micromamba_download_url("2.5.0-1", &platform);
        assert!(url.is_some());
        let url = url.unwrap();
        assert!(url.contains("micromamba-win-64.tar.bz2"));
    }

    #[test]
    fn test_micromamba_download_url_macos_arm() {
        let platform = Platform {
            os: Os::MacOS,
            arch: Arch::Aarch64,
        };
        let url = CondaUrlBuilder::micromamba_download_url("2.5.0-1", &platform);
        assert!(url.is_some());
        let url = url.unwrap();
        assert!(url.contains("micromamba-osx-arm64.tar.bz2"));
    }
}
