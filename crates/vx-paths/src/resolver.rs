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
    /// Checks both store and legacy tools directories
    pub fn find_tool_executables(&self, tool_name: &str) -> Result<Vec<PathBuf>> {
        let mut executables = Vec::new();

        // Check store directory first (~/.vx/store/<tool>/<version>)
        let store_versions = self.manager.list_store_versions(tool_name)?;
        for version in store_versions {
            let version_dir = self.manager.version_store_dir(tool_name, &version);
            if let Some(exe_path) = self.find_executable_in_dir(&version_dir, tool_name) {
                executables.push(exe_path);
            }
        }

        // Also check legacy tools directory (~/.vx/tools/<tool>/<version>)
        let tool_versions = self.manager.list_tool_versions(tool_name)?;
        for version in tool_versions {
            let exe_path = self.manager.tool_executable_path(tool_name, &version);
            if exe_path.exists() && !executables.contains(&exe_path) {
                executables.push(exe_path);
            } else {
                // Search in subdirectories
                let version_dir = self.manager.tool_version_dir(tool_name, &version);
                if let Some(found_path) = self.find_executable_in_dir(&version_dir, tool_name) {
                    if !executables.contains(&found_path) {
                        executables.push(found_path);
                    }
                }
            }
        }

        Ok(executables)
    }

    /// Find the latest executable for a tool
    /// Checks both store and legacy tools directories
    pub fn find_latest_executable(&self, tool_name: &str) -> Result<Option<PathBuf>> {
        // Check store directory first
        let store_versions = self.manager.list_store_versions(tool_name)?;
        if let Some(version) = store_versions.last() {
            let version_dir = self.manager.version_store_dir(tool_name, version);
            if let Some(exe_path) = self.find_executable_in_dir(&version_dir, tool_name) {
                return Ok(Some(exe_path));
            }
        }

        // Fall back to legacy tools directory
        if let Some(latest_version) = self.manager.get_latest_tool_version(tool_name)? {
            let exe_path = self
                .manager
                .tool_executable_path(tool_name, &latest_version);
            if exe_path.exists() {
                return Ok(Some(exe_path));
            }

            // Search in subdirectories
            let version_dir = self.manager.tool_version_dir(tool_name, &latest_version);
            if let Some(found_path) = self.find_executable_in_dir(&version_dir, tool_name) {
                return Ok(Some(found_path));
            }
        }
        Ok(None)
    }

    /// Find executable for a specific tool version
    /// Checks both store and legacy tools directories
    pub fn find_version_executable(&self, tool_name: &str, version: &str) -> Option<PathBuf> {
        // Check store directory first
        let store_dir = self.manager.version_store_dir(tool_name, version);
        if let Some(exe_path) = self.find_executable_in_dir(&store_dir, tool_name) {
            return Some(exe_path);
        }

        // Fall back to legacy tools directory
        let exe_path = self.manager.tool_executable_path(tool_name, version);
        if exe_path.exists() {
            return Some(exe_path);
        }

        // Search in subdirectories
        let version_dir = self.manager.tool_version_dir(tool_name, version);
        self.find_executable_in_dir(&version_dir, tool_name)
    }

    /// Check if a tool is installed (any version)
    /// Checks both store and legacy tools directories
    pub fn is_tool_installed(&self, tool_name: &str) -> Result<bool> {
        // Check store directory
        let store_versions = self.manager.list_store_versions(tool_name)?;
        if !store_versions.is_empty() {
            return Ok(true);
        }

        // Check legacy tools directory
        let tool_versions = self.manager.list_tool_versions(tool_name)?;
        Ok(!tool_versions.is_empty())
    }

    /// Get all installed tools with their versions
    /// Includes tools from both store and legacy directories
    pub fn get_installed_tools_with_versions(&self) -> Result<Vec<(String, Vec<String>)>> {
        use std::collections::HashMap;

        let mut tool_versions: HashMap<String, Vec<String>> = HashMap::new();

        // Get tools from store directory
        let store_runtimes = self.manager.list_store_runtimes()?;
        for tool in store_runtimes {
            let versions = self.manager.list_store_versions(&tool)?;
            tool_versions.entry(tool).or_default().extend(versions);
        }

        // Get tools from legacy directory
        let legacy_tools = self.manager.list_installed_tools()?;
        for tool in legacy_tools {
            let versions = self.manager.list_tool_versions(&tool)?;
            let entry = tool_versions.entry(tool).or_default();
            for v in versions {
                if !entry.contains(&v) {
                    entry.push(v);
                }
            }
        }

        // Convert to sorted vec
        let mut result: Vec<_> = tool_versions.into_iter().collect();
        result.sort_by(|a, b| a.0.cmp(&b.0));
        for (_, versions) in &mut result {
            versions.sort();
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

    /// Search for an executable in a directory (recursively, up to 3 levels)
    /// This handles various archive structures:
    /// - Direct: ~/.vx/store/uv/0.9.17/uv
    /// - One level: ~/.vx/store/uv/0.9.17/uv-platform/uv
    /// - Two levels: ~/.vx/store/go/1.25.5/go/bin/go
    fn find_executable_in_dir(&self, dir: &std::path::Path, exe_name: &str) -> Option<PathBuf> {
        use crate::with_executable_extension;

        if !dir.exists() {
            return None;
        }

        let exe_name_with_ext = with_executable_extension(exe_name);

        // Check direct children (level 1)
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

                // Check one level deeper (level 2)
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

                            // Check two levels deeper (level 3) - for go/bin/go structure
                            if sub_path.is_dir() {
                                if let Ok(deep_entries) = std::fs::read_dir(&sub_path) {
                                    for deep_entry in deep_entries.filter_map(|e| e.ok()) {
                                        let deep_path = deep_entry.path();
                                        if deep_path.is_file() {
                                            if let Some(name) =
                                                deep_path.file_name().and_then(|n| n.to_str())
                                            {
                                                if name == exe_name_with_ext || name == exe_name {
                                                    return Some(deep_path);
                                                }
                                            }
                                        }
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
