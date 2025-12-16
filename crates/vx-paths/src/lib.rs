//! Cross-platform path management for vx tool installations
//!
//! This crate provides a unified interface for managing tool installation paths
//! across different platforms, ensuring consistent directory structures and
//! proper handling of executable file extensions.
//!
//! # Directory Structure
//!
//! ```text
//! ~/.vx/
//! ├── store/                      # Global storage (Content-Addressable)
//! │   ├── node/20.0.0/           # Complete installation
//! │   ├── go/1.21.0/
//! │   └── uv/0.5.0/
//! │
//! ├── envs/                       # Virtual environments (links to store)
//! │   ├── default/               # Default environment
//! │   │   └── node -> ../../store/node/20.0.0
//! │   └── project-abc/           # Project-specific environment
//! │       └── node -> ../../store/node/18.0.0
//! │
//! ├── bin/                        # Global shims
//! ├── cache/                      # Download cache
//! ├── config/                     # Configuration
//! └── tmp/                        # Temporary files
//! ```

use anyhow::Result;
use std::path::{Path, PathBuf};

pub mod config;
pub mod link;
pub mod manager;
pub mod resolver;

pub use config::PathConfig;
pub use link::{LinkResult, LinkStrategy};
pub use manager::PathManager;
pub use resolver::PathResolver;

/// Standard vx directory structure
#[derive(Debug, Clone)]
pub struct VxPaths {
    /// Base vx directory (~/.vx)
    pub base_dir: PathBuf,
    /// Global store directory (~/.vx/store) - Content-Addressable Storage
    pub store_dir: PathBuf,
    /// Virtual environments directory (~/.vx/envs)
    pub envs_dir: PathBuf,
    /// Global shims directory (~/.vx/bin)
    pub bin_dir: PathBuf,
    /// Cache directory (~/.vx/cache)
    pub cache_dir: PathBuf,
    /// Configuration directory (~/.vx/config)
    pub config_dir: PathBuf,
    /// Temporary directory (~/.vx/tmp)
    pub tmp_dir: PathBuf,
}

impl VxPaths {
    /// Create VxPaths with default locations
    ///
    /// Uses VX_HOME environment variable if set, otherwise defaults to ~/.vx
    pub fn new() -> Result<Self> {
        // Check for VX_HOME environment variable first
        if let Ok(vx_home) = std::env::var("VX_HOME") {
            return Ok(Self::with_base_dir(vx_home));
        }

        let home_dir = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?;

        let base_dir = home_dir.join(".vx");

        Ok(Self {
            store_dir: base_dir.join("store"),
            envs_dir: base_dir.join("envs"),
            bin_dir: base_dir.join("bin"),
            cache_dir: base_dir.join("cache"),
            config_dir: base_dir.join("config"),
            tmp_dir: base_dir.join("tmp"),
            base_dir,
        })
    }

    /// Create VxPaths with custom base directory
    pub fn with_base_dir<P: AsRef<Path>>(base_dir: P) -> Self {
        let base_dir = base_dir.as_ref().to_path_buf();

        Self {
            store_dir: base_dir.join("store"),
            envs_dir: base_dir.join("envs"),
            bin_dir: base_dir.join("bin"),
            cache_dir: base_dir.join("cache"),
            config_dir: base_dir.join("config"),
            tmp_dir: base_dir.join("tmp"),
            base_dir,
        }
    }

    /// Ensure all directories exist
    pub fn ensure_dirs(&self) -> Result<()> {
        std::fs::create_dir_all(&self.base_dir)?;
        std::fs::create_dir_all(&self.store_dir)?;
        std::fs::create_dir_all(&self.envs_dir)?;
        std::fs::create_dir_all(&self.bin_dir)?;
        std::fs::create_dir_all(&self.cache_dir)?;
        std::fs::create_dir_all(&self.config_dir)?;
        std::fs::create_dir_all(&self.tmp_dir)?;
        Ok(())
    }

    /// Get the store directory for a specific runtime
    pub fn runtime_store_dir(&self, runtime_name: &str) -> PathBuf {
        self.store_dir.join(runtime_name)
    }

    /// Get the store directory for a specific runtime version
    pub fn version_store_dir(&self, runtime_name: &str, version: &str) -> PathBuf {
        self.runtime_store_dir(runtime_name).join(version)
    }

    /// Get the environment directory
    pub fn env_dir(&self, env_name: &str) -> PathBuf {
        self.envs_dir.join(env_name)
    }

    /// Get the default environment directory
    pub fn default_env_dir(&self) -> PathBuf {
        self.envs_dir.join("default")
    }
}

impl Default for VxPaths {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| {
            // Fallback to current directory if home directory is not available
            Self::with_base_dir(".vx")
        })
    }
}

/// Get the executable file extension for the current platform
pub fn executable_extension() -> &'static str {
    if cfg!(target_os = "windows") {
        ".exe"
    } else {
        ""
    }
}

/// Add executable extension to a tool name if needed
pub fn with_executable_extension(tool_name: &str) -> String {
    format!("{}{}", tool_name, executable_extension())
}
