//! Installer utilities and types
//!
//! This module provides common installation functionality that can be used
//! by tool implementations.

use crate::Result;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Trait for installing tools
#[async_trait::async_trait]
pub trait Installer: Send + Sync {
    /// Get the name of the tool this installer supports
    fn tool_name(&self) -> &str;
    
    /// Install a specific version of the tool
    async fn install(&self, config: &InstallConfig) -> Result<PathBuf>;
    
    /// Uninstall a specific version of the tool
    async fn uninstall(&self, tool_name: &str, version: &str) -> Result<()>;
    
    /// Check if a specific version is installed
    async fn is_version_installed(&self, tool_name: &str, version: &str) -> Result<bool>;
    
    /// Get the installation directory for a tool version
    fn get_install_dir(&self, tool_name: &str, version: &str) -> PathBuf;
    
    /// Get all installed versions of the tool
    async fn get_installed_versions(&self, tool_name: &str) -> Result<Vec<String>>;
    
    /// Verify installation integrity
    async fn verify_installation(&self, tool_name: &str, version: &str) -> Result<bool> {
        self.is_version_installed(tool_name, version).await
    }
}

/// Configuration for tool installation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallConfig {
    /// Name of the tool to install
    pub tool_name: String,
    
    /// Version to install
    pub version: String,
    
    /// Installation method
    pub install_method: InstallMethod,
    
    /// Download URL (if applicable)
    pub download_url: Option<String>,
    
    /// Installation directory
    pub install_dir: PathBuf,
    
    /// Whether to force reinstallation
    pub force: bool,
    
    /// Checksum for verification
    pub checksum: Option<String>,
    
    /// Additional configuration
    pub metadata: std::collections::HashMap<String, String>,
}

/// Different methods for installing tools
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InstallMethod {
    /// Download and extract archive
    Archive { format: ArchiveFormat },
    
    /// Use system package manager
    PackageManager { manager: String, package: String },
    
    /// Run installation script
    Script { url: String },
    
    /// Download single binary
    Binary,
    
    /// Custom installation method
    Custom { method: String },
}

/// Supported archive formats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ArchiveFormat {
    Zip,
    TarGz,
    TarXz,
    TarBz2,
    SevenZip,
}

/// Installation progress callback
pub type ProgressCallback = Box<dyn Fn(InstallProgress) + Send + Sync>;

/// Installation progress information
#[derive(Debug, Clone)]
pub struct InstallProgress {
    pub stage: InstallStage,
    pub progress: f64, // 0.0 to 1.0
    pub message: String,
}

/// Installation stages
#[derive(Debug, Clone, PartialEq)]
pub enum InstallStage {
    Downloading,
    Extracting,
    Installing,
    Verifying,
    Completed,
    Failed,
}

/// Installation result
#[derive(Debug)]
pub struct InstallResult {
    pub success: bool,
    pub installed_path: Option<PathBuf>,
    pub error_message: Option<String>,
    pub installation_time: std::time::Duration,
}

impl InstallConfig {
    /// Create a new install config with minimal information
    pub fn new(tool_name: String, version: String, install_dir: PathBuf) -> Self {
        Self {
            tool_name,
            version,
            install_method: InstallMethod::Binary,
            download_url: None,
            install_dir,
            force: false,
            checksum: None,
            metadata: std::collections::HashMap::new(),
        }
    }

    /// Set installation method
    pub fn with_method(mut self, method: InstallMethod) -> Self {
        self.install_method = method;
        self
    }

    /// Set download URL
    pub fn with_download_url(mut self, url: String) -> Self {
        self.download_url = Some(url);
        self
    }

    /// Set force reinstallation
    pub fn with_force(mut self, force: bool) -> Self {
        self.force = force;
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

/// Default installer implementation that provides common download and extraction functionality
pub struct DefaultInstaller {
    client: reqwest::Client,
}

impl DefaultInstaller {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    /// Download and extract a file to the target directory
    pub async fn download_and_extract(&self, url: &str, target_dir: &Path) -> Result<()> {
        // Create target directory
        std::fs::create_dir_all(target_dir)?;

        // Download the file
        let response = self.client.get(url).send().await?;
        let bytes = response.bytes().await?;

        // Determine file type and extract
        if url.ends_with(".zip") {
            self.extract_zip(&bytes, target_dir)?;
        } else if url.ends_with(".tar.gz") || url.ends_with(".tgz") {
            self.extract_tar_gz(&bytes, target_dir)?;
        } else if url.ends_with(".tar.xz") {
            self.extract_tar_xz(&bytes, target_dir)?;
        } else {
            // Assume it's a single binary
            let filename = url.split('/').last().unwrap_or("binary");
            let target_path = target_dir.join(filename);
            std::fs::write(target_path, &bytes)?;
        }

        Ok(())
    }

    /// Extract ZIP archive
    fn extract_zip(&self, data: &[u8], target_dir: &Path) -> Result<()> {
        // This would use the zip crate to extract
        // For now, just create a placeholder
        let _ = (data, target_dir);
        Ok(())
    }

    /// Extract tar.gz archive
    fn extract_tar_gz(&self, data: &[u8], target_dir: &Path) -> Result<()> {
        // This would use the tar and flate2 crates
        // For now, just create a placeholder
        let _ = (data, target_dir);
        Ok(())
    }

    /// Extract tar.xz archive
    fn extract_tar_xz(&self, data: &[u8], target_dir: &Path) -> Result<()> {
        // This would use the tar and xz2 crates
        // For now, just create a placeholder
        let _ = (data, target_dir);
        Ok(())
    }
}

impl Default for DefaultInstaller {
    fn default() -> Self {
        Self::new()
    }
}
