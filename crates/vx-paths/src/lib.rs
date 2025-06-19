//! Cross-platform path management for vx tool installations
//!
//! This crate provides a unified interface for managing tool installation paths
//! across different platforms, ensuring consistent directory structures and
//! proper handling of executable file extensions.

use anyhow::Result;
use std::path::{Path, PathBuf};

pub mod config;
pub mod manager;
pub mod resolver;

pub use config::PathConfig;
pub use manager::PathManager;
pub use resolver::PathResolver;

/// Standard vx directory structure
#[derive(Debug, Clone)]
pub struct VxPaths {
    /// Base vx directory (~/.vx)
    pub base_dir: PathBuf,
    /// Tools installation directory (~/.vx/tools)
    pub tools_dir: PathBuf,
    /// Cache directory (~/.vx/cache)
    pub cache_dir: PathBuf,
    /// Configuration directory (~/.vx/config)
    pub config_dir: PathBuf,
    /// Temporary directory (~/.vx/tmp)
    pub tmp_dir: PathBuf,
}

impl VxPaths {
    /// Create VxPaths with default locations
    pub fn new() -> Result<Self> {
        let home_dir = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?;

        let base_dir = home_dir.join(".vx");

        Ok(Self {
            tools_dir: base_dir.join("tools"),
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
            tools_dir: base_dir.join("tools"),
            cache_dir: base_dir.join("cache"),
            config_dir: base_dir.join("config"),
            tmp_dir: base_dir.join("tmp"),
            base_dir,
        }
    }

    /// Ensure all directories exist
    pub fn ensure_dirs(&self) -> Result<()> {
        std::fs::create_dir_all(&self.base_dir)?;
        std::fs::create_dir_all(&self.tools_dir)?;
        std::fs::create_dir_all(&self.cache_dir)?;
        std::fs::create_dir_all(&self.config_dir)?;
        std::fs::create_dir_all(&self.tmp_dir)?;
        Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vx_paths_creation() {
        let paths = VxPaths::with_base_dir("/tmp/test-vx");

        assert_eq!(paths.base_dir, PathBuf::from("/tmp/test-vx"));
        assert_eq!(paths.tools_dir, PathBuf::from("/tmp/test-vx/tools"));
        assert_eq!(paths.cache_dir, PathBuf::from("/tmp/test-vx/cache"));
        assert_eq!(paths.config_dir, PathBuf::from("/tmp/test-vx/config"));
        assert_eq!(paths.tmp_dir, PathBuf::from("/tmp/test-vx/tmp"));
    }

    #[test]
    fn test_executable_extension() {
        if cfg!(target_os = "windows") {
            assert_eq!(executable_extension(), ".exe");
            assert_eq!(with_executable_extension("node"), "node.exe");
        } else {
            assert_eq!(executable_extension(), "");
            assert_eq!(with_executable_extension("node"), "node");
        }
    }
}
