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
    /// Custom store directory
    pub store_dir: Option<PathBuf>,
    /// Custom environments directory
    pub envs_dir: Option<PathBuf>,
    /// Custom bin directory
    pub bin_dir: Option<PathBuf>,
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
            store_dir: None,
            envs_dir: None,
            bin_dir: None,
            cache_dir: None,
            config_dir: None,
            tmp_dir: None,
        }
    }

    /// Create PathConfig with custom base directory
    pub fn with_base_dir<P: AsRef<Path>>(base_dir: P) -> Self {
        Self {
            base_dir: Some(base_dir.as_ref().to_path_buf()),
            store_dir: None,
            envs_dir: None,
            bin_dir: None,
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
            store_dir: self.store_dir.clone().unwrap_or(default_paths.store_dir),
            npm_tools_dir: default_paths.npm_tools_dir,
            pip_tools_dir: default_paths.pip_tools_dir,
            envs_dir: self.envs_dir.clone().unwrap_or(default_paths.envs_dir),
            bin_dir: self.bin_dir.clone().unwrap_or(default_paths.bin_dir),
            cache_dir: self.cache_dir.clone().unwrap_or(default_paths.cache_dir),
            config_dir: self.config_dir.clone().unwrap_or(default_paths.config_dir),
            tmp_dir: self.tmp_dir.clone().unwrap_or(default_paths.tmp_dir),
            providers_dir: default_paths.providers_dir,
            // RFC 0025: Global packages CAS
            packages_dir: default_paths.packages_dir,
            shims_dir: default_paths.shims_dir,
        })
    }

    /// Load configuration from environment variables
    pub fn from_env() -> Self {
        Self {
            base_dir: std::env::var("VX_HOME").ok().map(PathBuf::from),
            store_dir: std::env::var("VX_STORE_DIR").ok().map(PathBuf::from),
            envs_dir: std::env::var("VX_ENVS_DIR").ok().map(PathBuf::from),
            bin_dir: std::env::var("VX_BIN_DIR").ok().map(PathBuf::from),
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
        if other.store_dir.is_some() {
            self.store_dir = other.store_dir.clone();
        }
        if other.envs_dir.is_some() {
            self.envs_dir = other.envs_dir.clone();
        }
        if other.bin_dir.is_some() {
            self.bin_dir = other.bin_dir.clone();
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
