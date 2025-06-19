//! Path resolver for finding and resolving tool paths

use crate::PathManager;
use anyhow::Result;
use std::path::PathBuf;

/// Resolves tool paths and finds installed tools
#[derive(Debug, Clone)]
pub struct PathResolver {
    manager: PathManager,
}

impl PathResolver {
    /// Create a new PathResolver
    pub fn new(manager: PathManager) -> Self {
        Self { manager }
    }

    /// Find all executable paths for a tool (all versions)
    pub fn find_tool_executables(&self, tool_name: &str) -> Result<Vec<PathBuf>> {
        let versions = self.manager.list_tool_versions(tool_name)?;
        let mut executables = Vec::new();

        for version in versions {
            let exe_path = self.manager.tool_executable_path(tool_name, &version);
            if exe_path.exists() {
                executables.push(exe_path);
            }
        }

        Ok(executables)
    }

    /// Find the latest executable for a tool
    pub fn find_latest_executable(&self, tool_name: &str) -> Result<Option<PathBuf>> {
        if let Some(latest_version) = self.manager.get_latest_tool_version(tool_name)? {
            let exe_path = self
                .manager
                .tool_executable_path(tool_name, &latest_version);
            if exe_path.exists() {
                return Ok(Some(exe_path));
            }
        }
        Ok(None)
    }

    /// Find executable for a specific tool version
    pub fn find_version_executable(&self, tool_name: &str, version: &str) -> Option<PathBuf> {
        let exe_path = self.manager.tool_executable_path(tool_name, version);
        if exe_path.exists() {
            Some(exe_path)
        } else {
            None
        }
    }

    /// Check if a tool is installed (any version)
    pub fn is_tool_installed(&self, tool_name: &str) -> Result<bool> {
        let versions = self.manager.list_tool_versions(tool_name)?;
        Ok(!versions.is_empty())
    }

    /// Get all installed tools with their versions
    pub fn get_installed_tools_with_versions(&self) -> Result<Vec<(String, Vec<String>)>> {
        let tools = self.manager.list_installed_tools()?;
        let mut result = Vec::new();

        for tool in tools {
            let versions = self.manager.list_tool_versions(&tool)?;
            result.push((tool, versions));
        }

        Ok(result)
    }

    /// Resolve tool path with version preference
    /// If version is specified, try to find that specific version
    /// Otherwise, return the latest version
    pub fn resolve_tool_path(
        &self,
        tool_name: &str,
        version: Option<&str>,
    ) -> Result<Option<PathBuf>> {
        match version {
            Some(v) => Ok(self.find_version_executable(tool_name, v)),
            None => self.find_latest_executable(tool_name),
        }
    }

    /// Get the path manager
    pub fn manager(&self) -> &PathManager {
        &self.manager
    }
}

impl Default for PathResolver {
    fn default() -> Self {
        Self::new(PathManager::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_path_resolver() {
        let temp_dir = TempDir::new().unwrap();
        let manager = PathManager::with_base_dir(temp_dir.path()).unwrap();
        let resolver = PathResolver::new(manager);

        // Initially no tools
        assert!(!resolver.is_tool_installed("node").unwrap());
        assert_eq!(
            resolver.find_tool_executables("node").unwrap(),
            Vec::<PathBuf>::new()
        );
        assert_eq!(resolver.find_latest_executable("node").unwrap(), None);

        // Create a tool installation
        let _version_dir = resolver
            .manager()
            .create_tool_version_dir("node", "18.17.0")
            .unwrap();
        let exe_path = resolver.manager().tool_executable_path("node", "18.17.0");
        std::fs::write(&exe_path, "fake executable").unwrap();

        // Now it should be found
        assert!(resolver.is_tool_installed("node").unwrap());
        assert_eq!(
            resolver.find_tool_executables("node").unwrap(),
            vec![exe_path.clone()]
        );
        assert_eq!(
            resolver.find_latest_executable("node").unwrap(),
            Some(exe_path.clone())
        );
        assert_eq!(
            resolver.find_version_executable("node", "18.17.0"),
            Some(exe_path.clone())
        );
        assert_eq!(
            resolver.resolve_tool_path("node", None).unwrap(),
            Some(exe_path.clone())
        );
        assert_eq!(
            resolver.resolve_tool_path("node", Some("18.17.0")).unwrap(),
            Some(exe_path)
        );
    }
}
