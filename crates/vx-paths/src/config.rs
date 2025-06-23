//! Configuration for path management

use crate::{PathManager, VxPaths};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Configuration for path management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathConfig {
    /// Custom base directory for vx installations
    pub base_dir: Option<PathBuf>,
    /// Custom tools directory
    pub tools_dir: Option<PathBuf>,
    /// Custom cache directory
    pub cache_dir: Option<PathBuf>,
    /// Custom config directory
    pub config_dir: Option<PathBuf>,
    /// Custom temporary directory
    pub tmp_dir: Option<PathBuf>,
}

impl PathConfig {
    /// Create a new PathConfig with default values
    pub fn new() -> Self {
        Self {
            base_dir: None,
            tools_dir: None,
            cache_dir: None,
            config_dir: None,
            tmp_dir: None,
        }
    }

    /// Create PathConfig with custom base directory
    pub fn with_base_dir<P: AsRef<Path>>(base_dir: P) -> Self {
        Self {
            base_dir: Some(base_dir.as_ref().to_path_buf()),
            tools_dir: None,
            cache_dir: None,
            config_dir: None,
            tmp_dir: None,
        }
    }

    /// Create a PathManager from this configuration
    pub fn create_path_manager(&self) -> Result<PathManager> {
        let paths = self.create_vx_paths()?;
        paths.ensure_dirs()?;
        Ok(PathManager::from_paths(paths))
    }

    /// Create VxPaths from this configuration
    pub fn create_vx_paths(&self) -> Result<VxPaths> {
        let default_paths = if let Some(base_dir) = &self.base_dir {
            VxPaths::with_base_dir(base_dir)
        } else {
            VxPaths::new()?
        };

        Ok(VxPaths {
            base_dir: self.base_dir.clone().unwrap_or(default_paths.base_dir),
            tools_dir: self.tools_dir.clone().unwrap_or(default_paths.tools_dir),
            cache_dir: self.cache_dir.clone().unwrap_or(default_paths.cache_dir),
            config_dir: self.config_dir.clone().unwrap_or(default_paths.config_dir),
            tmp_dir: self.tmp_dir.clone().unwrap_or(default_paths.tmp_dir),
            turbo_cdn_cache_dir: default_paths.turbo_cdn_cache_dir,
            turbo_cdn_logs_dir: default_paths.turbo_cdn_logs_dir,
        })
    }

    /// Load configuration from environment variables
    pub fn from_env() -> Self {
        Self {
            base_dir: std::env::var("VX_BASE_DIR").ok().map(PathBuf::from),
            tools_dir: std::env::var("VX_TOOLS_DIR").ok().map(PathBuf::from),
            cache_dir: std::env::var("VX_CACHE_DIR").ok().map(PathBuf::from),
            config_dir: std::env::var("VX_CONFIG_DIR").ok().map(PathBuf::from),
            tmp_dir: std::env::var("VX_TMP_DIR").ok().map(PathBuf::from),
        }
    }

    /// Merge with another PathConfig, preferring values from other
    pub fn merge(&mut self, other: &PathConfig) {
        if other.base_dir.is_some() {
            self.base_dir = other.base_dir.clone();
        }
        if other.tools_dir.is_some() {
            self.tools_dir = other.tools_dir.clone();
        }
        if other.cache_dir.is_some() {
            self.cache_dir = other.cache_dir.clone();
        }
        if other.config_dir.is_some() {
            self.config_dir = other.config_dir.clone();
        }
        if other.tmp_dir.is_some() {
            self.tmp_dir = other.tmp_dir.clone();
        }
    }
}
impl Default for PathConfig {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_path_config_creation() {
        let config = PathConfig::new();
        assert!(config.base_dir.is_none());
        assert!(config.tools_dir.is_none());

        let temp_dir = TempDir::new().unwrap();
        let config = PathConfig::with_base_dir(temp_dir.path());
        assert_eq!(config.base_dir, Some(temp_dir.path().to_path_buf()));
    }

    #[test]
    fn test_path_config_merge() {
        let mut config1 = PathConfig::new();
        let config2 = PathConfig::with_base_dir("/custom/path");

        config1.merge(&config2);
        assert_eq!(config1.base_dir, Some(PathBuf::from("/custom/path")));
    }

    #[test]
    fn test_create_path_manager() {
        let temp_dir = TempDir::new().unwrap();
        let config = PathConfig::with_base_dir(temp_dir.path());
        let manager = config.create_path_manager().unwrap();

        assert!(manager.tools_dir().exists());
        assert!(manager.cache_dir().exists());
    }

    #[test]
    fn test_from_env() {
        std::env::set_var("VX_BASE_DIR", "/test/base");
        std::env::set_var("VX_TOOLS_DIR", "/test/tools");

        let config = PathConfig::from_env();
        assert_eq!(config.base_dir, Some(PathBuf::from("/test/base")));
        assert_eq!(config.tools_dir, Some(PathBuf::from("/test/tools")));

        std::env::remove_var("VX_BASE_DIR");
        std::env::remove_var("VX_TOOLS_DIR");
    }
}
