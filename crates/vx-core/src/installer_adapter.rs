//! Adapter for integrating vx-installer with vx-core
//!
//! This module provides a bridge between the legacy vx-core installation system
//! and the new vx-installer crate, allowing for gradual migration.

use crate::{Result, VxEnvironment, VxError};
use std::path::PathBuf;
use vx_installer::{
    ArchiveFormat as VxInstallerArchiveFormat, InstallConfig as VxInstallerConfig,
    InstallMethod as VxInstallerMethod, Installer as VxInstaller,
};

// Re-export types for compatibility with vx-core
pub use vx_installer::{ArchiveFormat, InstallConfig, InstallMethod};

// Legacy types for backward compatibility
use serde::{Deserialize, Serialize};

/// Installation progress information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallProgress {
    /// Current stage of installation
    pub stage: InstallStage,
    /// Progress percentage (0-100)
    pub progress: u8,
    /// Current message
    pub message: String,
}

/// Installation stages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InstallStage {
    /// Downloading the tool
    Downloading,
    /// Extracting archive
    Extracting,
    /// Installing files
    Installing,
    /// Configuring tool
    Configuring,
    /// Completed successfully
    Completed,
    /// Failed with error
    Failed,
}

/// Adapter that wraps vx-installer functionality for use in vx-core
pub struct InstallerAdapter {
    installer: VxInstaller,
    env: VxEnvironment,
}

impl InstallerAdapter {
    /// Create a new installer adapter
    pub async fn new() -> Result<Self> {
        let installer = VxInstaller::new()
            .await
            .map_err(|e| VxError::InstallationFailed {
                tool_name: "unknown".to_string(),
                version: "unknown".to_string(),
                message: format!("Failed to create installer: {}", e),
            })?;
        let env = VxEnvironment::new()?;

        Ok(Self { installer, env })
    }

    /// Download and install a tool from URL (compatible with ToolDownloader::download_and_install)
    pub async fn download_and_install(
        &self,
        tool_name: &str,
        version: &str,
        download_url: &str,
    ) -> Result<PathBuf> {
        // Create installation directory
        let install_dir = self.env.get_version_install_dir(tool_name, version);

        // Determine installation method based on URL
        let install_method = self.detect_install_method(download_url);

        // Create install configuration
        let config = VxInstallerConfig::builder()
            .tool_name(tool_name)
            .version(version)
            .download_url(download_url)
            .install_method(install_method)
            .install_dir(install_dir)
            .force(false)
            .build();

        // Perform installation
        let executable_path = self
            .installer
            .install(&config)
            .await
            .map_err(|e| self.convert_installer_error(e, tool_name, version))?;

        Ok(executable_path)
    }

    /// Check if a tool version is installed
    pub async fn is_version_installed(&self, tool_name: &str, version: &str) -> Result<bool> {
        let install_dir = self.env.get_version_install_dir(tool_name, version);

        let config = VxInstallerConfig::builder()
            .tool_name(tool_name)
            .version(version)
            .install_dir(install_dir)
            .build();

        let is_installed = self
            .installer
            .is_installed(&config)
            .await
            .map_err(|e| self.convert_installer_error(e, tool_name, version))?;

        Ok(is_installed)
    }

    /// Uninstall a tool version
    pub async fn uninstall(&self, tool_name: &str, version: &str) -> Result<()> {
        let install_dir = self.env.get_version_install_dir(tool_name, version);

        self.installer
            .uninstall(tool_name, &install_dir)
            .await
            .map_err(|e| self.convert_installer_error(e, tool_name, version))?;

        Ok(())
    }

    /// Detect installation method from download URL
    fn detect_install_method(&self, url: &str) -> VxInstallerMethod {
        let url_lower = url.to_lowercase();

        if url_lower.ends_with(".zip") {
            VxInstallerMethod::Archive {
                format: VxInstallerArchiveFormat::Zip,
            }
        } else if url_lower.ends_with(".tar.gz") || url_lower.ends_with(".tgz") {
            VxInstallerMethod::Archive {
                format: VxInstallerArchiveFormat::TarGz,
            }
        } else if url_lower.ends_with(".tar.xz") || url_lower.ends_with(".txz") {
            VxInstallerMethod::Archive {
                format: VxInstallerArchiveFormat::TarXz,
            }
        } else if url_lower.ends_with(".tar.bz2") || url_lower.ends_with(".tbz2") {
            VxInstallerMethod::Archive {
                format: VxInstallerArchiveFormat::TarBz2,
            }
        } else {
            // Default to binary for unknown formats
            VxInstallerMethod::Binary
        }
    }

    /// Convert vx-installer errors to vx-core errors
    fn convert_installer_error(
        &self,
        error: vx_installer::Error,
        tool_name: &str,
        version: &str,
    ) -> VxError {
        match error {
            vx_installer::Error::DownloadFailed { url, reason } => {
                VxError::DownloadFailed { url, reason }
            }
            vx_installer::Error::InstallationFailed { message, .. } => {
                VxError::InstallationFailed {
                    tool_name: tool_name.to_string(),
                    version: version.to_string(),
                    message,
                }
            }
            vx_installer::Error::ExtractionFailed { reason, .. } => VxError::InstallationFailed {
                tool_name: tool_name.to_string(),
                version: version.to_string(),
                message: format!("Extraction failed: {}", reason),
            },
            vx_installer::Error::ExecutableNotFound { .. } => VxError::InstallationFailed {
                tool_name: tool_name.to_string(),
                version: version.to_string(),
                message: "Executable not found after installation".to_string(),
            },
            vx_installer::Error::AlreadyInstalled { .. } => VxError::VersionAlreadyInstalled {
                tool_name: tool_name.to_string(),
                version: version.to_string(),
            },
            _ => VxError::InstallationFailed {
                tool_name: tool_name.to_string(),
                version: version.to_string(),
                message: format!("Installation error: {}", error),
            },
        }
    }
}

/// Legacy ToolDownloader that uses the new installer adapter
pub struct ToolDownloader {
    adapter: InstallerAdapter,
}

impl ToolDownloader {
    /// Create a new tool downloader
    pub async fn new() -> Result<Self> {
        let adapter = InstallerAdapter::new().await?;
        Ok(Self { adapter })
    }

    /// Download and install a tool from URL (legacy interface)
    pub async fn download_and_install(
        &self,
        tool_name: &str,
        version: &str,
        download_url: &str,
    ) -> Result<PathBuf> {
        self.adapter
            .download_and_install(tool_name, version, download_url)
            .await
    }
}

impl Default for ToolDownloader {
    fn default() -> Self {
        // Note: This will panic if the async new() fails
        // In practice, this should be replaced with proper async initialization
        tokio::runtime::Handle::current()
            .block_on(Self::new())
            .expect("Failed to create tool downloader")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_installer_adapter_creation() {
        let adapter = InstallerAdapter::new().await;
        assert!(adapter.is_ok());
    }

    #[test]
    fn test_detect_install_method() {
        let adapter = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(InstallerAdapter::new())
            .unwrap();

        // Test ZIP detection
        let method = adapter.detect_install_method("https://example.com/tool.zip");
        assert!(matches!(
            method,
            VxInstallerMethod::Archive {
                format: VxInstallerArchiveFormat::Zip
            }
        ));

        // Test TAR.GZ detection
        let method = adapter.detect_install_method("https://example.com/tool.tar.gz");
        assert!(matches!(
            method,
            VxInstallerMethod::Archive {
                format: VxInstallerArchiveFormat::TarGz
            }
        ));

        // Test binary detection
        let method = adapter.detect_install_method("https://example.com/tool");
        assert!(matches!(method, VxInstallerMethod::Binary));
    }

    #[tokio::test]
    async fn test_tool_downloader_creation() {
        let downloader = ToolDownloader::new().await;
        assert!(downloader.is_ok());
    }
}
