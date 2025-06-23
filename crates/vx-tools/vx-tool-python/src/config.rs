//! Configuration for Python tool using vx-config system

use anyhow::Result;
use std::path::PathBuf;
use vx_installer::{ArchiveFormat, InstallConfig, InstallMethod, LifecycleHooks};

/// Python URL builder for different platforms
pub struct PythonUrlBuilder;

impl PythonUrlBuilder {
    /// Build download URL for Python Build Standalone
    pub fn build_url(version: &str) -> Result<String> {
        // For now, use a simple URL template
        // TODO: Use vx-config when the configuration structure is finalized
        let platform_pattern = Self::get_platform_pattern()?;

        // Python Build Standalone uses format: cpython-{python_version}+{build_date}-{platform}.tar.gz
        // We need to map the build date to the actual Python version
        let python_version = Self::get_python_version_for_build(version)?;

        let url = format!(
            "https://github.com/astral-sh/python-build-standalone/releases/download/{}/cpython-{}+{}-{}.tar.gz",
            version, python_version, version, platform_pattern
        );
        Ok(url)
    }

    /// Get executable name for current platform
    pub fn get_executable_name() -> Result<String> {
        if cfg!(windows) {
            Ok("python.exe".to_string())
        } else {
            Ok("python3".to_string())
        }
    }

    /// Get binary directory for current platform
    pub fn get_bin_dir() -> Result<String> {
        if cfg!(windows) {
            Ok("python".to_string())
        } else {
            Ok("python/bin".to_string())
        }
    }

    /// Get Python version for a given build date
    /// This maps build dates to the latest Python version available in that build
    pub fn get_python_version_for_build(build_date: &str) -> Result<String> {
        // For recent builds, we can use a reasonable default
        // In a real implementation, this should query the actual release info
        match build_date {
            "20250612" => Ok("3.13.5".to_string()),
            "20250610" => Ok("3.13.4".to_string()),
            "20250604" => Ok("3.13.4".to_string()),
            "20250529" => Ok("3.13.4".to_string()),
            "20250517" => Ok("3.13.4".to_string()),
            _ => {
                // Default to a recent stable version
                Ok("3.13.4".to_string())
            }
        }
    }

    /// Get platform pattern for Python Build Standalone
    fn get_platform_pattern() -> Result<String> {
        let pattern = if cfg!(target_os = "windows") && cfg!(target_arch = "x86_64") {
            "x86_64-pc-windows-msvc-install_only"
        } else if cfg!(target_os = "linux") && cfg!(target_arch = "x86_64") {
            "x86_64-unknown-linux-gnu-install_only"
        } else if cfg!(target_os = "macos") && cfg!(target_arch = "x86_64") {
            "x86_64-apple-darwin-install_only"
        } else if cfg!(target_os = "macos") && cfg!(target_arch = "aarch64") {
            "aarch64-apple-darwin-install_only"
        } else {
            return Err(anyhow::anyhow!(
                "Unsupported platform: {}-{}",
                std::env::consts::OS,
                std::env::consts::ARCH
            ));
        };

        Ok(pattern.to_string())
    }
}

/// Create installation configuration for Python
pub fn create_install_config(version: &str, install_dir: PathBuf, force: bool) -> InstallConfig {
    let download_url = PythonUrlBuilder::build_url(version).unwrap_or_default();

    // Create lifecycle hooks for Python
    let hooks = LifecycleHooks::default();

    InstallConfig::builder()
        .tool_name("python".to_string())
        .version(version)
        .download_url(download_url)
        .install_method(InstallMethod::Archive {
            format: if cfg!(windows) {
                ArchiveFormat::Zip
            } else {
                ArchiveFormat::TarGz
            },
        })
        .install_dir(install_dir)
        .force(force)
        .lifecycle_hooks(hooks)
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_create_install_config() {
        let temp_dir = TempDir::new().unwrap();
        let install_dir = temp_dir.path().to_path_buf();

        let config = create_install_config("3.12.1", install_dir.clone(), false);

        assert_eq!(config.tool_name, "python");
        assert_eq!(config.version, "3.12.1");
        assert_eq!(config.install_dir, install_dir);
        assert!(!config.force);

        // Check install method
        match config.install_method {
            InstallMethod::Archive { format } => {
                if cfg!(windows) {
                    assert!(matches!(format, ArchiveFormat::Zip));
                } else {
                    assert!(matches!(format, ArchiveFormat::TarGz));
                }
            }
            _ => panic!("Expected Archive install method"),
        }
    }

    #[test]
    fn test_create_install_config_with_force() {
        let temp_dir = TempDir::new().unwrap();
        let install_dir = temp_dir.path().to_path_buf();

        let config = create_install_config("3.11.7", install_dir, true);

        assert_eq!(config.version, "3.11.7");
        assert!(config.force);
    }
}
