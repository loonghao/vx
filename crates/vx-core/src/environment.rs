//! Environment management for VX tool manager
//!
//! This module handles:
//! - Tool installation directories
//! - Version management
//! - Environment isolation
//! - PATH management

use crate::{Result, VxError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// VX environment manager
#[derive(Debug, Clone)]
pub struct VxEnvironment {
    /// Base directory for all VX installations
    base_dir: PathBuf,
    /// Configuration directory
    config_dir: PathBuf,
    /// Cache directory
    cache_dir: PathBuf,
}

/// Tool installation information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInstallation {
    /// Tool name
    pub tool_name: String,
    /// Installed version
    pub version: String,
    /// Installation directory
    pub install_dir: PathBuf,
    /// Executable path
    pub executable_path: PathBuf,
    /// Installation timestamp
    pub installed_at: chrono::DateTime<chrono::Utc>,
    /// Whether this is the active version
    pub is_active: bool,
}

/// Environment configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentConfig {
    /// Active versions for each tool
    pub active_versions: HashMap<String, String>,
    /// Global settings
    pub global_settings: HashMap<String, String>,
}

impl VxEnvironment {
    /// Create a new VX environment
    pub fn new() -> Result<Self> {
        let base_dir = Self::get_vx_home()?;
        let config_dir = base_dir.join("config");
        let cache_dir = base_dir.join("cache");

        // Ensure directories exist
        std::fs::create_dir_all(&base_dir)?;
        std::fs::create_dir_all(&config_dir)?;
        std::fs::create_dir_all(&cache_dir)?;

        Ok(Self {
            base_dir,
            config_dir,
            cache_dir,
        })
    }

    /// Create a new VX environment with custom base directory (for testing)
    pub fn new_with_base_dir<P: AsRef<Path>>(base_dir: P) -> Result<Self> {
        let base_dir = base_dir.as_ref().to_path_buf();
        let config_dir = base_dir.join("config");
        let cache_dir = base_dir.join("cache");

        // Ensure directories exist
        std::fs::create_dir_all(&base_dir)?;
        std::fs::create_dir_all(&config_dir)?;
        std::fs::create_dir_all(&cache_dir)?;

        Ok(Self {
            base_dir,
            config_dir,
            cache_dir,
        })
    }

    /// Get VX home directory
    pub fn get_vx_home() -> Result<PathBuf> {
        if let Ok(vx_home) = std::env::var("VX_HOME") {
            Ok(PathBuf::from(vx_home))
        } else if let Some(home) = dirs::home_dir() {
            Ok(home.join(".vx"))
        } else {
            Err(VxError::ConfigurationError {
                message: "Cannot determine VX home directory".to_string(),
            })
        }
    }

    /// Get base installation directory
    pub fn get_base_install_dir(&self) -> PathBuf {
        self.base_dir.join("tools")
    }

    /// Get tool installation directory
    pub fn get_tool_install_dir(&self, tool_name: &str) -> PathBuf {
        self.get_base_install_dir().join(tool_name)
    }

    /// Get version installation directory
    pub fn get_version_install_dir(&self, tool_name: &str, version: &str) -> PathBuf {
        self.get_tool_install_dir(tool_name).join(version)
    }

    /// Get cache directory for downloads
    pub fn get_cache_dir(&self) -> PathBuf {
        self.cache_dir.clone()
    }

    /// Get download cache directory for a tool
    pub fn get_tool_cache_dir(&self, tool_name: &str) -> PathBuf {
        self.cache_dir.join("downloads").join(tool_name)
    }

    /// Get shim directory for tool proxies
    pub fn shim_dir(&self) -> Result<PathBuf> {
        let shim_dir = self.base_dir.join("shims");
        std::fs::create_dir_all(&shim_dir)?;
        Ok(shim_dir)
    }

    /// Get bin directory for vx executables
    pub fn bin_dir(&self) -> Result<PathBuf> {
        let bin_dir = self.base_dir.join("bin");
        std::fs::create_dir_all(&bin_dir)?;
        Ok(bin_dir)
    }

    /// Get configuration file path
    pub fn get_config_file(&self) -> PathBuf {
        self.config_dir.join("environment.toml")
    }

    /// Load environment configuration
    pub fn load_config(&self) -> Result<EnvironmentConfig> {
        let config_file = self.get_config_file();

        if !config_file.exists() {
            return Ok(EnvironmentConfig {
                active_versions: HashMap::new(),
                global_settings: HashMap::new(),
            });
        }

        let content = std::fs::read_to_string(&config_file)?;
        let config: EnvironmentConfig =
            toml::from_str(&content).map_err(|e| VxError::ConfigurationError {
                message: format!("Failed to parse environment config: {}", e),
            })?;

        Ok(config)
    }

    /// Save environment configuration
    pub fn save_config(&self, config: &EnvironmentConfig) -> Result<()> {
        let config_file = self.get_config_file();
        let content = toml::to_string_pretty(config).map_err(|e| VxError::ConfigurationError {
            message: format!("Failed to serialize environment config: {}", e),
        })?;

        std::fs::write(&config_file, content)?;
        Ok(())
    }

    /// Get active version for a tool
    pub fn get_active_version(&self, tool_name: &str) -> Result<Option<String>> {
        let config = self.load_config()?;
        Ok(config.active_versions.get(tool_name).cloned())
    }

    /// Set active version for a tool
    pub fn set_active_version(&self, tool_name: &str, version: &str) -> Result<()> {
        let mut config = self.load_config()?;
        config
            .active_versions
            .insert(tool_name.to_string(), version.to_string());
        self.save_config(&config)?;
        Ok(())
    }

    /// List all installed versions for a tool
    pub fn list_installed_versions(&self, tool_name: &str) -> Result<Vec<String>> {
        let tool_dir = self.get_tool_install_dir(tool_name);

        if !tool_dir.exists() {
            return Ok(Vec::new());
        }

        let mut versions = Vec::new();
        for entry in std::fs::read_dir(&tool_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                if let Some(version) = entry.file_name().to_str() {
                    versions.push(version.to_string());
                }
            }
        }

        // Sort versions (simple string sort for now, could be improved with semver)
        versions.sort();
        Ok(versions)
    }

    /// Check if a version is installed
    pub fn is_version_installed(&self, tool_name: &str, version: &str) -> bool {
        let version_dir = self.get_version_install_dir(tool_name, version);
        version_dir.exists()
    }

    /// Get installation info for a tool version
    pub fn get_installation_info(
        &self,
        tool_name: &str,
        version: &str,
    ) -> Result<Option<ToolInstallation>> {
        if !self.is_version_installed(tool_name, version) {
            return Ok(None);
        }

        let install_dir = self.get_version_install_dir(tool_name, version);
        let config = self.load_config()?;
        let is_active = config.active_versions.get(tool_name) == Some(&version.to_string());

        // Try to find the executable
        let executable_path = self.find_executable_in_dir(&install_dir, tool_name)?;

        Ok(Some(ToolInstallation {
            tool_name: tool_name.to_string(),
            version: version.to_string(),
            install_dir,
            executable_path,
            installed_at: chrono::Utc::now(), // TODO: Get actual installation time
            is_active,
        }))
    }

    /// Find executable in installation directory
    pub fn find_executable_in_dir(&self, dir: &Path, tool_name: &str) -> Result<PathBuf> {
        // Common executable patterns (including nested directories)
        // On Windows, prioritize .cmd and .exe files over files without extensions
        let mut patterns = vec![];

        // Add Windows-specific patterns first (higher priority)
        #[cfg(windows)]
        {
            patterns.extend(vec![
                format!("{}.cmd", tool_name),
                format!("{}.bat", tool_name),
                format!("{}.exe", tool_name),
                format!("{}.ps1", tool_name),
            ]);
        }

        // Add generic patterns
        #[cfg(not(windows))]
        {
            patterns.extend(vec![format!("{}.exe", tool_name), tool_name.to_string()]);
        }

        // On Windows, add the no-extension pattern last (lowest priority)
        #[cfg(windows)]
        {
            patterns.push(tool_name.to_string());
        }

        // Add bin subdirectory patterns
        #[cfg(windows)]
        {
            patterns.extend(vec![
                format!("bin/{}.cmd", tool_name),
                format!("bin/{}.bat", tool_name),
                format!("bin/{}.exe", tool_name),
                format!("bin/{}.ps1", tool_name),
                format!("bin/{}", tool_name),
            ]);
        }

        #[cfg(not(windows))]
        {
            patterns.extend(vec![
                format!("bin/{}.exe", tool_name),
                format!("bin/{}", tool_name),
            ]);
        }

        // First try direct patterns
        for pattern in &patterns {
            let exe_path = dir.join(pattern);
            if self.is_executable(&exe_path) {
                return Ok(exe_path);
            }
        }

        // If not found, search recursively in subdirectories (common for archives)
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                if entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
                    let subdir = entry.path();

                    // Try patterns in subdirectory
                    for pattern in &patterns {
                        let exe_path = subdir.join(pattern);
                        if self.is_executable(&exe_path) {
                            return Ok(exe_path);
                        }
                    }
                }
            }
        }

        Err(VxError::ExecutableNotFound {
            tool_name: tool_name.to_string(),
            install_dir: dir.to_path_buf(),
        })
    }

    /// Check if a path is an executable (handles symlinks)
    fn is_executable(&self, path: &Path) -> bool {
        if !path.exists() {
            return false;
        }

        // Check if it's a regular file or symlink
        if let Ok(metadata) = std::fs::metadata(path) {
            if metadata.is_file() {
                return true;
            }
        }

        // Check if it's a symlink that points to an executable
        if let Ok(metadata) = std::fs::symlink_metadata(path) {
            if metadata.file_type().is_symlink() {
                // For symlinks, we consider them executable if they exist
                // (the target might be executable even if we can't check it directly)
                return true;
            }
        }

        false
    }

    /// Clean up unused installations
    pub fn cleanup_unused(&self, keep_latest: usize) -> Result<Vec<String>> {
        let mut cleaned = Vec::new();
        let tools_dir = self.get_base_install_dir();

        if !tools_dir.exists() {
            return Ok(cleaned);
        }

        for entry in std::fs::read_dir(&tools_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                if let Some(tool_name) = entry.file_name().to_str() {
                    let removed = self.cleanup_tool_versions(tool_name, keep_latest)?;
                    cleaned.extend(removed);
                }
            }
        }

        Ok(cleaned)
    }

    /// Clean up old versions of a specific tool
    pub fn cleanup_tool_versions(
        &self,
        tool_name: &str,
        keep_latest: usize,
    ) -> Result<Vec<String>> {
        let mut versions = self.list_installed_versions(tool_name)?;
        let config = self.load_config()?;
        let active_version = config.active_versions.get(tool_name);

        if versions.len() <= keep_latest {
            return Ok(Vec::new());
        }

        // Sort versions (newest first)
        versions.sort();
        versions.reverse();

        let mut removed = Vec::new();
        let mut kept_count = 0;

        for version in versions {
            // Always keep the active version
            if Some(&version) == active_version {
                continue;
            }

            if kept_count < keep_latest {
                kept_count += 1;
                continue;
            }

            // Remove this version
            let version_dir = self.get_version_install_dir(tool_name, &version);
            if version_dir.exists() {
                std::fs::remove_dir_all(&version_dir)?;
                removed.push(format!("{}@{}", tool_name, version));
            }
        }

        Ok(removed)
    }
}

impl Default for VxEnvironment {
    fn default() -> Self {
        Self::new().expect("Failed to create VX environment")
    }
}
