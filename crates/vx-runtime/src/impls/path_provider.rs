//! Real path provider implementation

use crate::traits::PathProvider;
use anyhow::Result;
use std::path::{Path, PathBuf};
use vx_paths::VxPaths;

/// Real path provider using VxPaths
pub struct RealPathProvider {
    paths: VxPaths,
}

impl RealPathProvider {
    /// Create a new real path provider
    pub fn new() -> Result<Self> {
        Ok(Self {
            paths: VxPaths::new()?,
        })
    }

    /// Create with custom base directory
    pub fn with_base_dir(base_dir: impl AsRef<Path>) -> Self {
        Self {
            paths: VxPaths::with_base_dir(base_dir),
        }
    }
}

impl Default for RealPathProvider {
    fn default() -> Self {
        Self::new().expect("Failed to create RealPathProvider")
    }
}

impl PathProvider for RealPathProvider {
    fn vx_home(&self) -> PathBuf {
        self.paths.base_dir.clone()
    }

    fn store_dir(&self) -> PathBuf {
        self.paths.store_dir.clone()
    }

    fn envs_dir(&self) -> PathBuf {
        self.paths.envs_dir.clone()
    }

    fn bin_dir(&self) -> PathBuf {
        self.paths.bin_dir.clone()
    }

    fn cache_dir(&self) -> PathBuf {
        self.paths.cache_dir.clone()
    }

    fn config_dir(&self) -> PathBuf {
        self.paths.config_dir.clone()
    }

    fn runtime_store_dir(&self, name: &str) -> PathBuf {
        self.paths.runtime_store_dir(name)
    }

    fn version_store_dir(&self, name: &str, version: &str) -> PathBuf {
        self.paths.version_store_dir(name, version)
    }

    fn executable_path(&self, name: &str, version: &str) -> PathBuf {
        let exe_name = vx_paths::with_executable_extension(name);
        self.version_store_dir(name, version)
            .join("bin")
            .join(exe_name)
    }

    fn env_dir(&self, env_name: &str) -> PathBuf {
        self.paths.env_dir(env_name)
    }

    // ========== npm-tools paths ==========

    fn npm_tools_dir(&self) -> PathBuf {
        self.paths.npm_tools_dir.clone()
    }

    fn npm_tool_dir(&self, package_name: &str) -> PathBuf {
        self.paths.npm_tool_dir(package_name)
    }

    fn npm_tool_version_dir(&self, package_name: &str, version: &str) -> PathBuf {
        self.paths.npm_tool_version_dir(package_name, version)
    }

    fn npm_tool_bin_dir(&self, package_name: &str, version: &str) -> PathBuf {
        self.paths.npm_tool_bin_dir(package_name, version)
    }

    // ========== pip-tools paths ==========

    fn pip_tools_dir(&self) -> PathBuf {
        self.paths.pip_tools_dir.clone()
    }

    fn pip_tool_dir(&self, package_name: &str) -> PathBuf {
        self.paths.pip_tool_dir(package_name)
    }

    fn pip_tool_version_dir(&self, package_name: &str, version: &str) -> PathBuf {
        self.paths.pip_tool_version_dir(package_name, version)
    }

    fn pip_tool_venv_dir(&self, package_name: &str, version: &str) -> PathBuf {
        self.paths.pip_tool_venv_dir(package_name, version)
    }

    fn pip_tool_bin_dir(&self, package_name: &str, version: &str) -> PathBuf {
        self.paths.pip_tool_bin_dir(package_name, version)
    }

    // ========== conda-tools paths ==========

    fn conda_tools_dir(&self) -> PathBuf {
        self.paths.conda_tools_dir.clone()
    }

    fn conda_tool_dir(&self, package_name: &str) -> PathBuf {
        self.paths.conda_tool_dir(package_name)
    }

    fn conda_tool_version_dir(&self, package_name: &str, version: &str) -> PathBuf {
        self.paths.conda_tool_version_dir(package_name, version)
    }

    fn conda_tool_env_dir(&self, package_name: &str, version: &str) -> PathBuf {
        self.paths.conda_tool_env_dir(package_name, version)
    }

    fn conda_tool_bin_dir(&self, package_name: &str, version: &str) -> PathBuf {
        self.paths.conda_tool_bin_dir(package_name, version)
    }
}
