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
        let mut executables = Vec::new();

        // Check store directory (~/.vx/store/<tool>/<version>)
        let store_versions = self.manager.list_store_versions(tool_name)?;
        for version in store_versions {
            let version_dir = self.manager.version_store_dir(tool_name, &version);
            if let Some(exe_path) = self.find_executable_in_dir(&version_dir, tool_name) {
                executables.push(exe_path);
            }
        }

        Ok(executables)
    }

    /// Find the latest executable for a tool
    pub fn find_latest_executable(&self, tool_name: &str) -> Result<Option<PathBuf>> {
        let store_versions = self.manager.list_store_versions(tool_name)?;
        if let Some(version) = store_versions.last() {
            let version_dir = self.manager.version_store_dir(tool_name, version);
            if let Some(exe_path) = self.find_executable_in_dir(&version_dir, tool_name) {
                return Ok(Some(exe_path));
            }
        }
        Ok(None)
    }

    /// Find executable for a specific tool version
    pub fn find_version_executable(&self, tool_name: &str, version: &str) -> Option<PathBuf> {
        let store_dir = self.manager.version_store_dir(tool_name, version);
        self.find_executable_in_dir(&store_dir, tool_name)
    }

    /// Check if a tool is installed (any version)
    pub fn is_tool_installed(&self, tool_name: &str) -> Result<bool> {
        let store_versions = self.manager.list_store_versions(tool_name)?;
        Ok(!store_versions.is_empty())
    }

    /// Get all installed tools with their versions
    pub fn get_installed_tools_with_versions(&self) -> Result<Vec<(String, Vec<String>)>> {
        let store_runtimes = self.manager.list_store_runtimes()?;
        let mut result = Vec::new();

        for tool in store_runtimes {
            let mut versions = self.manager.list_store_versions(&tool)?;
            versions.sort();
            result.push((tool, versions));
        }

        result.sort_by(|a, b| a.0.cmp(&b.0));
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
        if !dir.exists() {
            return None;
        }

        // Build list of possible executable names in priority order
        // On Windows, .exe and .cmd should be preferred over extensionless files
        // because extensionless files are typically shell scripts
        let possible_names: Vec<String> = if cfg!(windows) {
            vec![
                format!("{}.exe", exe_name),
                format!("{}.cmd", exe_name),
                exe_name.to_string(),
            ]
        } else {
            vec![exe_name.to_string()]
        };

        /// Helper to find the best matching executable from a list of candidates
        /// Returns the one with highest priority (lowest index in possible_names)
        fn find_best_match(candidates: &[PathBuf], possible_names: &[String]) -> Option<PathBuf> {
            for name in possible_names {
                for candidate in candidates {
                    if let Some(file_name) = candidate.file_name().and_then(|n| n.to_str()) {
                        if file_name == name {
                            return Some(candidate.clone());
                        }
                    }
                }
            }
            None
        }

        // Collect all matching files at each level, then pick the best one
        let mut all_candidates: Vec<PathBuf> = Vec::new();

        // Check direct children (level 1)
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();

                // Check if this is a matching executable
                if path.is_file() {
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        if possible_names.iter().any(|n| n == name) {
                            all_candidates.push(path.clone());
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
                                    if possible_names.iter().any(|n| n == name) {
                                        all_candidates.push(sub_path.clone());
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
                                                if possible_names.iter().any(|n| n == name) {
                                                    all_candidates.push(deep_path);
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

        find_best_match(&all_candidates, &possible_names)
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

        // Create a tool installation in store directory
        let version_dir = resolver.manager().version_store_dir("node", "18.17.0");
        std::fs::create_dir_all(&version_dir).unwrap();
        let exe_name = if cfg!(windows) { "node.exe" } else { "node" };
        let exe_path = version_dir.join(exe_name);
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
