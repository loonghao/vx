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

    /// Get the base vx directory
    pub fn base_dir(&self) -> &Path {
        &self.paths.base_dir
    }

    /// Get the tools directory
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
    /// Get the installation directory for a specific tool
    /// Returns: ~/.vx/tools/<tool>
    pub fn tool_dir(&self, tool_name: &str) -> PathBuf {
        self.paths.tools_dir.join(tool_name)
    }

    /// Get the installation directory for a specific tool version
    /// Returns: ~/.vx/tools/<tool>/<version>
    pub fn tool_version_dir(&self, tool_name: &str, version: &str) -> PathBuf {
        self.tool_dir(tool_name).join(version)
    }

    /// Get the executable path for a specific tool version
    /// Returns: ~/.vx/tools/<tool>/<version>/<tool>.exe (Windows) or ~/.vx/tools/<tool>/<version>/<tool> (Unix)
    pub fn tool_executable_path(&self, tool_name: &str, version: &str) -> PathBuf {
        let version_dir = self.tool_version_dir(tool_name, version);
        let executable_name = with_executable_extension(tool_name);
        version_dir.join(executable_name)
    }

    /// Check if a tool version is installed
    pub fn is_tool_version_installed(&self, tool_name: &str, version: &str) -> bool {
        let exe_path = self.tool_executable_path(tool_name, version);
        exe_path.exists()
    }

    /// List all installed versions of a tool
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
                    // Verify that the executable exists in this version directory
                    if self.is_tool_version_installed(tool_name, version) {
                        versions.push(version.to_string());
                    }
                }
            }
        }

        // Sort versions (simple string sort for now, could be improved with semver)
        versions.sort();
        Ok(versions)
    }

    /// Get the latest installed version of a tool
    pub fn get_latest_tool_version(&self, tool_name: &str) -> Result<Option<String>> {
        let versions = self.list_tool_versions(tool_name)?;

        if versions.is_empty() {
            return Ok(None);
        }

        // For now, use simple string comparison
        // TODO: Implement proper semantic version comparison
        let latest = versions.into_iter().max();
        Ok(latest)
    }
    /// List all installed tools
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
                    // Check if this tool has any valid versions installed
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

    /// Create the directory structure for a tool version
    pub fn create_tool_version_dir(&self, tool_name: &str, version: &str) -> Result<PathBuf> {
        let version_dir = self.tool_version_dir(tool_name, version);
        std::fs::create_dir_all(&version_dir)?;
        Ok(version_dir)
    }

    /// Remove a specific tool version
    pub fn remove_tool_version(&self, tool_name: &str, version: &str) -> Result<()> {
        let version_dir = self.tool_version_dir(tool_name, version);

        if version_dir.exists() {
            std::fs::remove_dir_all(&version_dir)?;
        }

        // If this was the last version, remove the tool directory
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
            // Fallback to current directory if home directory is not available
            Self::with_base_dir(".vx")
                .expect("Failed to create PathManager with fallback directory")
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_path_manager_creation() {
        let temp_dir = TempDir::new().unwrap();
        let base_dir = temp_dir.path().join(".vx");
        let manager = PathManager::with_base_dir(&base_dir).unwrap();

        assert!(manager.tools_dir().exists());
        assert!(manager.cache_dir().exists());
        assert!(manager.config_dir().exists());
        assert!(manager.tmp_dir().exists());
    }

    #[test]
    fn test_tool_paths() {
        let temp_dir = TempDir::new().unwrap();
        let base_dir = temp_dir.path().join(".vx");
        let manager = PathManager::with_base_dir(&base_dir).unwrap();

        let tool_dir = manager.tool_dir("node");
        let version_dir = manager.tool_version_dir("node", "18.17.0");
        let exe_path = manager.tool_executable_path("node", "18.17.0");

        assert_eq!(tool_dir, base_dir.join("tools/node"));
        assert_eq!(version_dir, base_dir.join("tools/node/18.17.0"));

        if cfg!(target_os = "windows") {
            assert_eq!(exe_path, base_dir.join("tools/node/18.17.0/node.exe"));
        } else {
            assert_eq!(exe_path, base_dir.join("tools/node/18.17.0/node"));
        }
    }

    #[test]
    fn test_tool_version_management() {
        let temp_dir = TempDir::new().unwrap();
        let base_dir = temp_dir.path().join(".vx");
        let manager = PathManager::with_base_dir(&base_dir).unwrap();

        // Initially no versions
        assert!(!manager.is_tool_version_installed("node", "18.17.0"));
        assert_eq!(
            manager.list_tool_versions("node").unwrap(),
            Vec::<String>::new()
        );

        // Create version directory and executable
        let _version_dir = manager.create_tool_version_dir("node", "18.17.0").unwrap();
        let exe_path = manager.tool_executable_path("node", "18.17.0");
        std::fs::write(&exe_path, "fake executable").unwrap();

        // Now it should be detected
        assert!(manager.is_tool_version_installed("node", "18.17.0"));
        assert_eq!(manager.list_tool_versions("node").unwrap(), vec!["18.17.0"]);
        assert_eq!(
            manager.get_latest_tool_version("node").unwrap(),
            Some("18.17.0".to_string())
        );
        assert_eq!(manager.list_installed_tools().unwrap(), vec!["node"]);
    }
}
