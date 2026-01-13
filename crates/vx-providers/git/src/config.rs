//! URL builder and platform configuration for Git downloads.

use vx_runtime::{Arch, Os, Platform};

/// URL builder for Git downloads from GitHub releases.
pub struct GitUrlBuilder;

impl GitUrlBuilder {
    /// Build the download URL for a specific Git version and platform.
    ///
    /// Git for Windows provides portable versions via GitHub releases.
    /// For other platforms, Git is typically installed via system package managers.
    pub fn download_url(version: &str, platform: &Platform) -> Option<String> {
        // Git for Windows uses a different versioning scheme
        // e.g., v2.43.0.windows.1 -> MinGit-2.43.0-64-bit.zip
        let base_version = Self::extract_base_version(version);
        let filename = Self::get_filename(&base_version, platform)?;

        // Construct the full tag: most releases use .windows.1
        let full_tag = if version.contains(".windows.") {
            version.to_string()
        } else {
            format!("{}.windows.1", version)
        };

        // Git for Windows releases are hosted on GitHub
        Some(format!(
            "https://github.com/git-for-windows/git/releases/download/v{}/{}",
            full_tag, filename
        ))
    }

    /// Extract base version from Git for Windows version string.
    /// e.g., "2.43.0.windows.1" -> "2.43.0"
    fn extract_base_version(version: &str) -> String {
        if let Some(pos) = version.find(".windows") {
            version[..pos].to_string()
        } else {
            version.to_string()
        }
    }

    /// Get the filename for the Git archive.
    pub fn get_filename(version: &str, platform: &Platform) -> Option<String> {
        match (&platform.os, &platform.arch) {
            // Git for Windows - MinGit portable version
            (Os::Windows, Arch::X86_64) => Some(format!("MinGit-{}-64-bit.zip", version)),
            (Os::Windows, Arch::X86) => Some(format!("MinGit-{}-32-bit.zip", version)),
            // For non-Windows platforms, Git should be installed via system package manager
            // Return None to indicate no direct download available
            _ => None,
        }
    }

    /// Get the target triple for the platform.
    pub fn get_target_triple(platform: &Platform) -> &'static str {
        match (&platform.os, &platform.arch) {
            (Os::Windows, Arch::X86_64) => "x86_64-pc-windows-msvc",
            (Os::Windows, Arch::X86) => "i686-pc-windows-msvc",
            (Os::Windows, Arch::Aarch64) => "aarch64-pc-windows-msvc",
            (Os::MacOS, Arch::X86_64) => "x86_64-apple-darwin",
            (Os::MacOS, Arch::Aarch64) => "aarch64-apple-darwin",
            (Os::Linux, Arch::X86_64) => "x86_64-unknown-linux-gnu",
            (Os::Linux, Arch::Aarch64) => "aarch64-unknown-linux-gnu",
            (Os::Linux, Arch::Arm) => "arm-unknown-linux-gnueabihf",
            _ => "unknown",
        }
    }

    /// Get the archive extension for the platform.
    pub fn get_archive_extension(platform: &Platform) -> &'static str {
        match platform.os {
            Os::Windows => "zip",
            _ => "tar.gz",
        }
    }

    /// Get the executable name for the platform.
    pub fn get_executable_name(platform: &Platform) -> &'static str {
        match platform.os {
            Os::Windows => "git.exe",
            _ => "git",
        }
    }

    /// Get the platform string used in Git for Windows releases.
    pub fn get_platform_string(platform: &Platform) -> Option<&'static str> {
        match (&platform.os, &platform.arch) {
            (Os::Windows, Arch::X86_64) => Some("64-bit"),
            (Os::Windows, Arch::X86) => Some("32-bit"),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_base_version() {
        assert_eq!(
            GitUrlBuilder::extract_base_version("2.43.0.windows.1"),
            "2.43.0"
        );
        assert_eq!(GitUrlBuilder::extract_base_version("2.43.0"), "2.43.0");
    }

    #[test]
    fn test_get_filename_windows() {
        let platform = Platform {
            os: Os::Windows,
            arch: Arch::X86_64,
        };
        assert_eq!(
            GitUrlBuilder::get_filename("2.43.0", &platform),
            Some("MinGit-2.43.0-64-bit.zip".to_string())
        );
    }

    #[test]
    fn test_get_filename_linux() {
        let platform = Platform {
            os: Os::Linux,
            arch: Arch::X86_64,
        };
        assert_eq!(GitUrlBuilder::get_filename("2.43.0", &platform), None);
    }
}
