//! Path manager for vx tool installations

use crate::{with_executable_extension, VxPaths};
use anyhow::Result;
use std::path::{Path, PathBuf};

/// Manages paths for vx tool installations with standardized structure
#[derive(Debug, Clone)]
pub struct PathManager {
    paths: VxPaths,
}

impl PathManager {
    /// Create a new PathManager with default paths
    pub fn new() -> Result<Self> {
        let paths = VxPaths::new()?;
        paths.ensure_dirs()?;
        Ok(Self { paths })
    }

    /// Create a new PathManager with custom base directory
    pub fn with_base_dir<P: AsRef<Path>>(base_dir: P) -> Result<Self> {
        let paths = VxPaths::with_base_dir(base_dir);
        paths.ensure_dirs()?;
        Ok(Self { paths })
    }

    /// Create a PathManager from existing VxPaths
    pub fn from_paths(paths: VxPaths) -> Self {
        Self { paths }
    }

    // ========== Base Directories ==========

    /// Get the base vx directory
    pub fn base_dir(&self) -> &Path {
        &self.paths.base_dir
    }

    /// Get the global store directory
    pub fn store_dir(&self) -> &Path {
        &self.paths.store_dir
    }

    /// Get the environments directory
    pub fn envs_dir(&self) -> &Path {
        &self.paths.envs_dir
    }

    /// Get the bin directory (for shims)
    pub fn bin_dir(&self) -> &Path {
        &self.paths.bin_dir
    }

    /// Get the tools directory (legacy)
    pub fn tools_dir(&self) -> &Path {
        &self.paths.tools_dir
    }

    /// Get the cache directory
    pub fn cache_dir(&self) -> &Path {
        &self.paths.cache_dir
    }

    /// Get the config directory
    pub fn config_dir(&self) -> &Path {
        &self.paths.config_dir
    }

    /// Get the temporary directory
    pub fn tmp_dir(&self) -> &Path {
        &self.paths.tmp_dir
    }

    // ========== Store Paths (Content-Addressable Storage) ==========

    /// Get the store directory for a specific runtime
    /// Returns: ~/.vx/store/<runtime>
    pub fn runtime_store_dir(&self, runtime_name: &str) -> PathBuf {
        self.paths.store_dir.join(runtime_name)
    }

    /// Get the store directory for a specific runtime version
    /// Returns: ~/.vx/store/<runtime>/<version>
    pub fn version_store_dir(&self, runtime_name: &str, version: &str) -> PathBuf {
        self.runtime_store_dir(runtime_name).join(version)
    }

    /// Get the executable path in the store for a specific runtime version
    /// Returns: ~/.vx/store/<runtime>/<version>/bin/<runtime>.exe (Windows)
    pub fn store_executable_path(&self, runtime_name: &str, version: &str) -> PathBuf {
        let version_dir = self.version_store_dir(runtime_name, version);
        let executable_name = with_executable_extension(runtime_name);
        version_dir.join("bin").join(executable_name)
    }

    /// Check if a runtime version is installed in the store
    pub fn is_version_in_store(&self, runtime_name: &str, version: &str) -> bool {
        let version_dir = self.version_store_dir(runtime_name, version);
        version_dir.exists()
    }

    /// List all installed versions of a runtime in the store
    pub fn list_store_versions(&self, runtime_name: &str) -> Result<Vec<String>> {
        let runtime_dir = self.runtime_store_dir(runtime_name);

        if !runtime_dir.exists() {
            return Ok(Vec::new());
        }

        let mut versions = Vec::new();
        for entry in std::fs::read_dir(&runtime_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                if let Some(version) = entry.file_name().to_str() {
                    versions.push(version.to_string());
                }
            }
        }

        versions.sort();
        Ok(versions)
    }

    /// List all runtimes in the store
    pub fn list_store_runtimes(&self) -> Result<Vec<String>> {
        if !self.paths.store_dir.exists() {
            return Ok(Vec::new());
        }

        let mut runtimes = Vec::new();
        for entry in std::fs::read_dir(&self.paths.store_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                if let Some(name) = entry.file_name().to_str() {
                    runtimes.push(name.to_string());
                }
            }
        }

        runtimes.sort();
        Ok(runtimes)
    }

    // ========== Environment Paths ==========

    /// Get the directory for a specific environment
    /// Returns: ~/.vx/envs/<env_name>
    pub fn env_dir(&self, env_name: &str) -> PathBuf {
        self.paths.envs_dir.join(env_name)
    }

    /// Get the default environment directory
    /// Returns: ~/.vx/envs/default
    pub fn default_env_dir(&self) -> PathBuf {
        self.paths.envs_dir.join("default")
    }

    /// Get the runtime link path in an environment
    /// Returns: ~/.vx/envs/<env_name>/<runtime>
    pub fn env_runtime_path(&self, env_name: &str, runtime_name: &str) -> PathBuf {
        self.env_dir(env_name).join(runtime_name)
    }

    /// List all environments
    pub fn list_envs(&self) -> Result<Vec<String>> {
        if !self.paths.envs_dir.exists() {
            return Ok(Vec::new());
        }

        let mut envs = Vec::new();
        for entry in std::fs::read_dir(&self.paths.envs_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                if let Some(name) = entry.file_name().to_str() {
                    envs.push(name.to_string());
                }
            }
        }

        envs.sort();
        Ok(envs)
    }

    /// Check if an environment exists
    pub fn env_exists(&self, env_name: &str) -> bool {
        self.env_dir(env_name).exists()
    }

    /// Create an environment directory
    pub fn create_env(&self, env_name: &str) -> Result<PathBuf> {
        let env_dir = self.env_dir(env_name);
        std::fs::create_dir_all(&env_dir)?;
        Ok(env_dir)
    }

    /// Remove an environment
    pub fn remove_env(&self, env_name: &str) -> Result<()> {
        let env_dir = self.env_dir(env_name);
        if env_dir.exists() {
            std::fs::remove_dir_all(&env_dir)?;
        }
        Ok(())
    }

    // ========== Legacy Tool Paths (for backward compatibility) ==========

    /// Get the installation directory for a specific tool (legacy)
    /// Returns: ~/.vx/tools/<tool>
    pub fn tool_dir(&self, tool_name: &str) -> PathBuf {
        self.paths.tools_dir.join(tool_name)
    }

    /// Get the installation directory for a specific tool version (legacy)
    /// Returns: ~/.vx/tools/<tool>/<version>
    pub fn tool_version_dir(&self, tool_name: &str, version: &str) -> PathBuf {
        self.tool_dir(tool_name).join(version)
    }

    /// Get the executable path for a specific tool version (legacy)
    pub fn tool_executable_path(&self, tool_name: &str, version: &str) -> PathBuf {
        let version_dir = self.tool_version_dir(tool_name, version);
        let executable_name = with_executable_extension(tool_name);
        version_dir.join(executable_name)
    }

    /// Check if a tool version is installed (legacy)
    /// This checks both the simple path and searches subdirectories
    pub fn is_tool_version_installed(&self, tool_name: &str, version: &str) -> bool {
        // First check the simple path
        let exe_path = self.tool_executable_path(tool_name, version);
        if exe_path.exists() {
            return true;
        }

        // If not found, search in subdirectories (for archives that extract to subdirs)
        let version_dir = self.tool_version_dir(tool_name, version);
        self.find_executable_in_dir(&version_dir, tool_name)
            .is_some()
    }

    /// Search for an executable in a directory (recursively, up to 2 levels)
    fn find_executable_in_dir(&self, dir: &std::path::Path, exe_name: &str) -> Option<PathBuf> {
        if !dir.exists() {
            return None;
        }

        let exe_name_with_ext = with_executable_extension(exe_name);

        // Check direct children
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();

                // Check if this is the executable
                if path.is_file() {
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        if name == exe_name_with_ext || name == exe_name {
                            return Some(path);
                        }
                    }
                }

                // Check one level deeper (for archives that extract to subdirectories)
                if path.is_dir() {
                    if let Ok(sub_entries) = std::fs::read_dir(&path) {
                        for sub_entry in sub_entries.filter_map(|e| e.ok()) {
                            let sub_path = sub_entry.path();
                            if sub_path.is_file() {
                                if let Some(name) = sub_path.file_name().and_then(|n| n.to_str()) {
                                    if name == exe_name_with_ext || name == exe_name {
                                        return Some(sub_path);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        None
    }

    /// List all installed versions of a tool (legacy)
    pub fn list_tool_versions(&self, tool_name: &str) -> Result<Vec<String>> {
        let tool_dir = self.tool_dir(tool_name);

        if !tool_dir.exists() {
            return Ok(Vec::new());
        }

        let mut versions = Vec::new();

        for entry in std::fs::read_dir(&tool_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                if let Some(version) = entry.file_name().to_str() {
                    if self.is_tool_version_installed(tool_name, version) {
                        versions.push(version.to_string());
                    }
                }
            }
        }

        versions.sort();
        Ok(versions)
    }

    /// Get the latest installed version of a tool (legacy)
    pub fn get_latest_tool_version(&self, tool_name: &str) -> Result<Option<String>> {
        let versions = self.list_tool_versions(tool_name)?;

        if versions.is_empty() {
            return Ok(None);
        }

        let latest = versions.into_iter().max();
        Ok(latest)
    }

    /// List all installed tools (legacy)
    pub fn list_installed_tools(&self) -> Result<Vec<String>> {
        let tools_dir = &self.paths.tools_dir;

        if !tools_dir.exists() {
            return Ok(Vec::new());
        }

        let mut tools = Vec::new();

        for entry in std::fs::read_dir(tools_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                if let Some(tool_name) = entry.file_name().to_str() {
                    let versions = self.list_tool_versions(tool_name)?;
                    if !versions.is_empty() {
                        tools.push(tool_name.to_string());
                    }
                }
            }
        }

        tools.sort();
        Ok(tools)
    }

    /// Create the directory structure for a tool version (legacy)
    pub fn create_tool_version_dir(&self, tool_name: &str, version: &str) -> Result<PathBuf> {
        let version_dir = self.tool_version_dir(tool_name, version);
        std::fs::create_dir_all(&version_dir)?;
        Ok(version_dir)
    }

    /// Remove a specific tool version (legacy)
    pub fn remove_tool_version(&self, tool_name: &str, version: &str) -> Result<()> {
        let version_dir = self.tool_version_dir(tool_name, version);

        if version_dir.exists() {
            std::fs::remove_dir_all(&version_dir)?;
        }

        let tool_dir = self.tool_dir(tool_name);
        if tool_dir.exists() {
            let remaining_versions = self.list_tool_versions(tool_name)?;
            if remaining_versions.is_empty() {
                std::fs::remove_dir_all(&tool_dir)?;
            }
        }

        Ok(())
    }

    /// Get cache path for a tool
    pub fn tool_cache_dir(&self, tool_name: &str) -> PathBuf {
        self.paths.cache_dir.join(tool_name)
    }

    /// Get temporary path for a tool installation
    pub fn tool_tmp_dir(&self, tool_name: &str, version: &str) -> PathBuf {
        self.paths
            .tmp_dir
            .join(format!("{}-{}", tool_name, version))
    }
}

impl Default for PathManager {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| {
            Self::with_base_dir(".vx")
                .expect("Failed to create PathManager with fallback directory")
        })
    }
}
