//! Installation utilities and configuration

use crate::{
    downloader::Downloader,
    formats::ArchiveExtractor,
    progress::{ProgressContext, ProgressStyle},
    Error, Result,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Main installer for tools and packages
pub struct Installer {
    downloader: Downloader,
    extractor: ArchiveExtractor,
}

impl Installer {
    /// Create a new installer
    pub async fn new() -> Result<Self> {
        let downloader = Downloader::new()?;
        let extractor = ArchiveExtractor::new();

        Ok(Self {
            downloader,
            extractor,
        })
    }

    /// Create a new installer with custom timeout
    ///
    /// # Arguments
    /// * `timeout` - Download timeout duration
    /// * `cdn_enabled` - Whether to enable CDN acceleration
    pub async fn with_timeout(timeout: Duration, cdn_enabled: bool) -> Result<Self> {
        let downloader = Downloader::with_timeout(timeout, cdn_enabled)?;
        let extractor = ArchiveExtractor::new();

        Ok(Self {
            downloader,
            extractor,
        })
    }

    /// Install a tool using the provided configuration
    pub async fn install(&self, config: &InstallConfig) -> Result<PathBuf> {
        // Check if already installed and not forcing reinstall
        if !config.force && self.is_installed(config).await? {
            return Err(Error::AlreadyInstalled {
                tool_name: config.tool_name.clone(),
                version: config.version.clone(),
            });
        }

        // Create progress context
        let progress = ProgressContext::new(
            crate::progress::create_progress_reporter(ProgressStyle::default(), true),
            true,
        );

        match &config.install_method {
            InstallMethod::Archive { format: _ } => {
                self.install_from_archive(config, &progress).await
            }
            InstallMethod::Binary => self.install_binary(config, &progress).await,
            InstallMethod::Script { url } => self.install_from_script(config, url, &progress).await,
            InstallMethod::PackageManager { manager, package } => {
                self.install_from_package_manager(config, manager, package, &progress)
                    .await
            }
            InstallMethod::Custom { method } => {
                self.install_custom(config, method, &progress).await
            }
        }
    }

    /// Check if a tool version is already installed
    pub async fn is_installed(&self, config: &InstallConfig) -> Result<bool> {
        let install_dir = &config.install_dir;

        // Check if installation directory exists and contains executables
        if !install_dir.exists() {
            return Ok(false);
        }

        // Look for executable files
        let bin_dir = install_dir.join("bin");
        if bin_dir.exists() {
            let exe_name = if cfg!(windows) {
                format!("{}.exe", config.tool_name)
            } else {
                config.tool_name.clone()
            };

            let exe_path = bin_dir.join(&exe_name);
            Ok(exe_path.exists() && exe_path.is_file())
        } else {
            // Check if there are any executable files in the install directory
            self.has_executables(install_dir)
        }
    }

    /// Uninstall a tool
    pub async fn uninstall(&self, _tool_name: &str, install_dir: &Path) -> Result<()> {
        if install_dir.exists() {
            std::fs::remove_dir_all(install_dir)?;
        }
        Ok(())
    }

    /// Install from archive (ZIP, TAR, etc.)
    async fn install_from_archive(
        &self,
        config: &InstallConfig,
        progress: &ProgressContext,
    ) -> Result<PathBuf> {
        let download_url = config
            .download_url
            .as_ref()
            .ok_or_else(|| Error::InvalidConfig {
                message: "Download URL is required for archive installation".to_string(),
            })?;

        // Download the archive
        let temp_path = self
            .downloader
            .download_temp(download_url, progress)
            .await?;

        // Extract the archive
        let extracted_files = self
            .extractor
            .extract(&temp_path, &config.install_dir, progress)
            .await?;

        // Find the best executable
        let executable_path = self
            .extractor
            .find_best_executable(&extracted_files, &config.tool_name)?;

        // Clean up temporary file
        let _ = std::fs::remove_file(temp_path);

        Ok(executable_path)
    }

    /// Install binary file directly
    async fn install_binary(
        &self,
        config: &InstallConfig,
        progress: &ProgressContext,
    ) -> Result<PathBuf> {
        let download_url = config
            .download_url
            .as_ref()
            .ok_or_else(|| Error::InvalidConfig {
                message: "Download URL is required for binary installation".to_string(),
            })?;

        // Check metadata for target filename and directory (from layout config)
        let target_name = config
            .metadata
            .get("target_name")
            .cloned()
            .unwrap_or_else(|| {
                if cfg!(windows) {
                    format!("{}.exe", config.tool_name)
                } else {
                    config.tool_name.clone()
                }
            });

        let target_dir = config
            .metadata
            .get("target_dir")
            .map(|s| s.as_str())
            .unwrap_or("bin");

        // Create target directory
        let bin_dir = config.install_dir.join(target_dir);
        std::fs::create_dir_all(&bin_dir)?;

        let exe_path = bin_dir.join(target_name);

        // Download to temporary location first
        let temp_path = self
            .downloader
            .download_temp(download_url, progress)
            .await?;

        // Move/rename to final location
        std::fs::rename(&temp_path, &exe_path).or_else(|_| {
            // If rename fails (cross-device), copy instead
            std::fs::copy(&temp_path, &exe_path)?;
            std::fs::remove_file(&temp_path)?;
            Ok::<(), std::io::Error>(())
        })?;

        // Set executable permissions on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            let permissions_str = config
                .metadata
                .get("target_permissions")
                .map(|s| s.as_str())
                .unwrap_or("755");

            let mode = u32::from_str_radix(permissions_str, 8).unwrap_or(0o755);

            let mut permissions = std::fs::metadata(&exe_path)?.permissions();
            permissions.set_mode(mode);
            std::fs::set_permissions(&exe_path, permissions)?;
        }

        Ok(exe_path)
    }

    /// Install from script
    async fn install_from_script(
        &self,
        _config: &InstallConfig,
        _script_url: &str,
        _progress: &ProgressContext,
    ) -> Result<PathBuf> {
        // TODO: Implement script-based installation
        Err(Error::unsupported_format("script installation"))
    }

    /// Install using package manager
    async fn install_from_package_manager(
        &self,
        _config: &InstallConfig,
        _manager: &str,
        _package: &str,
        _progress: &ProgressContext,
    ) -> Result<PathBuf> {
        // TODO: Implement package manager installation
        Err(Error::unsupported_format("package manager installation"))
    }

    /// Install using custom method
    async fn install_custom(
        &self,
        _config: &InstallConfig,
        _method: &str,
        _progress: &ProgressContext,
    ) -> Result<PathBuf> {
        // TODO: Implement custom installation methods
        Err(Error::unsupported_format("custom installation"))
    }

    /// Check if directory contains executable files
    fn has_executables(&self, dir: &Path) -> Result<bool> {
        if !dir.exists() {
            return Ok(false);
        }

        for entry in walkdir::WalkDir::new(dir).max_depth(3) {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    if let Ok(metadata) = std::fs::metadata(path) {
                        let permissions = metadata.permissions();
                        if permissions.mode() & 0o111 != 0 {
                            return Ok(true);
                        }
                    }
                }

                #[cfg(windows)]
                {
                    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                        if matches!(ext.to_lowercase().as_str(), "exe" | "bat" | "cmd" | "com") {
                            return Ok(true);
                        }
                    }
                }
            }
        }

        Ok(false)
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

    /// Download timeout in milliseconds (default: 300000 = 5 minutes)
    #[serde(default = "default_download_timeout")]
    pub download_timeout_ms: u64,

    /// Maximum number of retry attempts (default: 3)
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,

    /// Additional configuration
    pub metadata: HashMap<String, String>,
}

fn default_download_timeout() -> u64 {
    300_000 // 5 minutes
}

fn default_max_retries() -> u32 {
    3
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

/// Builder for InstallConfig
pub struct InstallConfigBuilder {
    config: InstallConfig,
}

impl Default for InstallConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl InstallConfigBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            config: InstallConfig {
                tool_name: String::new(),
                version: String::new(),
                install_method: InstallMethod::Binary,
                download_url: None,
                install_dir: PathBuf::new(),
                force: false,
                checksum: None,
                download_timeout_ms: default_download_timeout(),
                max_retries: default_max_retries(),
                metadata: HashMap::new(),
            },
        }
    }

    /// Set the tool name
    pub fn tool_name(mut self, name: impl Into<String>) -> Self {
        self.config.tool_name = name.into();
        self
    }

    /// Set the version
    pub fn version(mut self, version: impl Into<String>) -> Self {
        self.config.version = version.into();
        self
    }

    /// Set the installation method
    pub fn install_method(mut self, method: InstallMethod) -> Self {
        self.config.install_method = method;
        self
    }

    /// Set the download URL
    pub fn download_url(mut self, url: impl Into<String>) -> Self {
        self.config.download_url = Some(url.into());
        self
    }

    /// Set the installation directory
    pub fn install_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.config.install_dir = dir.into();
        self
    }

    /// Set force reinstallation
    pub fn force(mut self, force: bool) -> Self {
        self.config.force = force;
        self
    }

    /// Set checksum
    pub fn checksum(mut self, checksum: impl Into<String>) -> Self {
        self.config.checksum = Some(checksum.into());
        self
    }

    /// Set download timeout in milliseconds
    pub fn download_timeout_ms(mut self, timeout_ms: u64) -> Self {
        self.config.download_timeout_ms = timeout_ms;
        self
    }

    /// Set download timeout from Duration
    pub fn download_timeout(mut self, timeout: Duration) -> Self {
        self.config.download_timeout_ms = timeout.as_millis() as u64;
        self
    }

    /// Set maximum retry attempts
    pub fn max_retries(mut self, retries: u32) -> Self {
        self.config.max_retries = retries;
        self
    }

    /// Add metadata
    pub fn metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.config.metadata.insert(key.into(), value.into());
        self
    }

    /// Build the configuration
    pub fn build(self) -> InstallConfig {
        self.config
    }
}

impl InstallConfig {
    /// Create a new builder
    pub fn builder() -> InstallConfigBuilder {
        InstallConfigBuilder::new()
    }
}
