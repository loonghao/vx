//! URL builder and platform configuration for Task
//!
//! Task releases are available at: https://github.com/go-task/task/releases
//! Download URL format: https://github.com/go-task/task/releases/download/v{version}/task_{os}_{arch}.{ext}

use vx_runtime::{Arch, Os, Platform};

/// URL builder for Task downloads
pub struct TaskUrlBuilder;

impl TaskUrlBuilder {
    /// Base URL for Task releases
    const BASE_URL: &'static str = "https://github.com/go-task/task/releases/download";

    /// Build the download URL for a specific version and platform
    pub fn download_url(version: &str, platform: &Platform) -> Option<String> {
        let (os_name, arch_name) = Self::get_platform_parts(platform)?;
        let ext = Self::get_archive_extension(platform);
        // Task uses 'v' prefix in release tags
        let version_tag = if version.starts_with('v') {
            version.to_string()
        } else {
            format!("v{}", version)
        };
        Some(format!(
            "{}/{}/task_{}_{}.{}",
            Self::BASE_URL,
            version_tag,
            os_name,
            arch_name,
            ext
        ))
    }

    /// Get the OS and arch parts for Task downloads
    pub fn get_platform_parts(platform: &Platform) -> Option<(&'static str, &'static str)> {
        let os_name = match &platform.os {
            Os::Windows => "windows",
            Os::MacOS => "darwin",
            Os::Linux => "linux",
            _ => return None,
        };

        let arch_name = match &platform.arch {
            Arch::X86_64 => "amd64",
            Arch::Aarch64 => "arm64",
            Arch::Arm => "arm",
            Arch::X86 => "386",
            _ => return None,
        };

        Some((os_name, arch_name))
    }

    /// Get the archive extension for the platform
    pub fn get_archive_extension(platform: &Platform) -> &'static str {
        match platform.os {
            Os::Windows => "zip",
            _ => "tar.gz",
        }
    }

    /// Get the executable name for the platform
    pub fn get_executable_name(platform: &Platform) -> &'static str {
        match platform.os {
            Os::Windows => "task.exe",
            _ => "task",
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
        let url = TaskUrlBuilder::download_url("3.40.1", &platform);
        assert_eq!(
            url,
            Some(
                "https://github.com/go-task/task/releases/download/v3.40.1/task_linux_amd64.tar.gz"
                    .to_string()
            )
        );
    }

    #[test]
    fn test_download_url_windows_x64() {
        let platform = Platform {
            os: Os::Windows,
            arch: Arch::X86_64,
        };
        let url = TaskUrlBuilder::download_url("3.40.1", &platform);
        assert_eq!(
            url,
            Some(
                "https://github.com/go-task/task/releases/download/v3.40.1/task_windows_amd64.zip"
                    .to_string()
            )
        );
    }

    #[test]
    fn test_download_url_macos_arm64() {
        let platform = Platform {
            os: Os::MacOS,
            arch: Arch::Aarch64,
        };
        let url = TaskUrlBuilder::download_url("3.40.1", &platform);
        assert_eq!(
            url,
            Some(
                "https://github.com/go-task/task/releases/download/v3.40.1/task_darwin_arm64.tar.gz"
                    .to_string()
            )
        );
    }

    #[test]
    fn test_download_url_with_v_prefix() {
        let platform = Platform {
            os: Os::Linux,
            arch: Arch::X86_64,
        };
        let url = TaskUrlBuilder::download_url("v3.40.1", &platform);
        assert_eq!(
            url,
            Some(
                "https://github.com/go-task/task/releases/download/v3.40.1/task_linux_amd64.tar.gz"
                    .to_string()
            )
        );
    }
}
