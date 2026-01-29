//! Path resolver for finding and resolving tool paths
//!
//! This module provides a unified interface for finding tool executables
//! across all vx-managed directories (store, npm-tools, pip-tools).

use crate::PathManager;
use anyhow::Result;
use std::path::{Path, PathBuf};

/// Result of finding a tool in vx-managed directories
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolLocation {
    /// The path to the executable
    pub path: PathBuf,
    /// The version of the tool
    pub version: String,
    /// The source of the tool (store, npm-tools, pip-tools)
    pub source: ToolSource,
}

/// Source of a tool installation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolSource {
    /// Tool installed in ~/.vx/store
    Store,
    /// Tool installed in ~/.vx/npm-tools
    NpmTools,
    /// Tool installed in ~/.vx/pip-tools
    PipTools,
    /// Tool installed in ~/.vx/conda-tools
    CondaTools,
}

impl std::fmt::Display for ToolSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ToolSource::Store => write!(f, "store"),
            ToolSource::NpmTools => write!(f, "npm-tools"),
            ToolSource::PipTools => write!(f, "pip-tools"),
            ToolSource::CondaTools => write!(f, "conda-tools"),
        }
    }
}

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

    /// Create a PathResolver with default paths
    pub fn default_paths() -> Result<Self> {
        Ok(Self::new(PathManager::new()?))
    }

    /// Get the path manager
    pub fn manager(&self) -> &PathManager {
        &self.manager
    }

    // ========== Unified Tool Finding API ==========

    /// Find a tool in any vx-managed directory
    /// Returns the first found location with version info
    pub fn find_tool(&self, tool_name: &str) -> Result<Option<ToolLocation>> {
        // Check store directory first
        if let Some(loc) = self.find_in_store(tool_name)? {
            return Ok(Some(loc));
        }

        // Check npm-tools directory
        if let Some(loc) = self.find_in_npm_tools(tool_name)? {
            return Ok(Some(loc));
        }

        // Check pip-tools directory
        if let Some(loc) = self.find_in_pip_tools(tool_name)? {
            return Ok(Some(loc));
        }

        // Check conda-tools directory
        if let Some(loc) = self.find_in_conda_tools(tool_name)? {
            return Ok(Some(loc));
        }

        Ok(None)
    }

    /// Find a tool in any vx-managed directory using a specific executable name
    /// This is useful for runtimes whose store directory differs from the executable name (e.g., msvc -> cl.exe)
    pub fn find_tool_with_executable(
        &self,
        tool_name: &str,
        exe_name: &str,
    ) -> Result<Option<ToolLocation>> {
        // Check store directory first
        if let Some(loc) = self.find_in_store_with_exe(tool_name, exe_name)? {
            return Ok(Some(loc));
        }

        // Check npm-tools directory
        if let Some(loc) = self.find_in_npm_tools(tool_name)? {
            return Ok(Some(loc));
        }

        // Check pip-tools directory
        if let Some(loc) = self.find_in_pip_tools(tool_name)? {
            return Ok(Some(loc));
        }

        // Check conda-tools directory
        if let Some(loc) = self.find_in_conda_tools(tool_name)? {
            return Ok(Some(loc));
        }

        Ok(None)
    }

    /// Find all installations of a tool across all directories
    pub fn find_all_tool_installations(&self, tool_name: &str) -> Result<Vec<ToolLocation>> {
        self.find_all_tool_installations_with_exe(tool_name, tool_name)
    }

    /// Find all installations of a tool across all directories with a specific executable name
    ///
    /// # Arguments
    /// * `tool_name` - The runtime/tool name (used for directory lookup)
    /// * `exe_name` - The executable name to search for
    pub fn find_all_tool_installations_with_exe(
        &self,
        tool_name: &str,
        exe_name: &str,
    ) -> Result<Vec<ToolLocation>> {
        let mut locations = Vec::new();

        // Collect from store
        locations.extend(self.find_all_in_store_with_exe(tool_name, exe_name)?);

        // Collect from npm-tools
        locations.extend(self.find_all_in_npm_tools(tool_name)?);

        // Collect from pip-tools
        locations.extend(self.find_all_in_pip_tools(tool_name)?);

        // Collect from conda-tools
        locations.extend(self.find_all_in_conda_tools(tool_name)?);

        Ok(locations)
    }

    /// Find the latest version of a tool
    pub fn find_latest_tool(&self, tool_name: &str) -> Result<Option<ToolLocation>> {
        let all = self.find_all_tool_installations(tool_name)?;
        // Return the last one (highest version due to sorting)
        Ok(all.into_iter().last())
    }

    /// Find a specific version of a tool
    pub fn find_tool_version(&self, tool_name: &str, version: &str) -> Option<ToolLocation> {
        self.find_tool_version_with_executable(tool_name, version, tool_name)
    }

    /// Find a specific version of a tool with a specific executable name
    ///
    /// # Arguments
    /// * `tool_name` - The runtime/tool name (used for directory lookup)
    /// * `version` - The version to find
    /// * `exe_name` - The executable name to search for
    pub fn find_tool_version_with_executable(
        &self,
        tool_name: &str,
        version: &str,
        exe_name: &str,
    ) -> Option<ToolLocation> {
        // Check store with platform redirection
        let platform_store_dir = self.manager.platform_store_dir(tool_name, version);
        if let Some(path) = self.find_executable_in_dir(&platform_store_dir, exe_name) {
            return Some(ToolLocation {
                path,
                version: version.to_string(),
                source: ToolSource::Store,
            });
        }

        // Check npm-tools
        let npm_bin = self.manager.npm_tool_bin_dir(tool_name, version);
        if let Some(path) = self.find_npm_executable(&npm_bin, tool_name) {
            return Some(ToolLocation {
                path,
                version: version.to_string(),
                source: ToolSource::NpmTools,
            });
        }

        // Check pip-tools
        let pip_bin = self.manager.pip_tool_bin_dir(tool_name, version);
        if let Some(path) = self.find_pip_executable(&pip_bin, tool_name) {
            return Some(ToolLocation {
                path,
                version: version.to_string(),
                source: ToolSource::PipTools,
            });
        }

        // Check conda-tools
        let conda_bin = self.manager.conda_tool_bin_dir(tool_name, version);
        if let Some(path) = self.find_conda_executable(&conda_bin, tool_name) {
            return Some(ToolLocation {
                path,
                version: version.to_string(),
                source: ToolSource::CondaTools,
            });
        }

        None
    }

    /// Check if a tool is installed (any version, any source)
    pub fn is_tool_installed(&self, tool_name: &str) -> Result<bool> {
        Ok(self.find_tool(tool_name)?.is_some())
    }

    // ========== Store Directory Methods ==========

    /// Find a tool in the store directory
    ///
    /// # Arguments
    /// * `tool_name` - The runtime/tool name (used for directory lookup)
    /// * `exe_name` - Optional executable name to search for (defaults to tool_name)
    pub fn find_in_store(&self, tool_name: &str) -> Result<Option<ToolLocation>> {
        self.find_in_store_with_exe(tool_name, tool_name)
    }

    /// Find a tool in the store directory with a specific executable name
    ///
    /// This method uses the new directory structure:
    /// - New (post-platform-redirection): <provider>/<version>/<platform>/
    ///
    /// # Arguments
    /// * `tool_name` - The runtime/tool name (used for directory lookup)
    /// * `exe_name` - The executable name to search for
    pub fn find_in_store_with_exe(
        &self,
        tool_name: &str,
        exe_name: &str,
    ) -> Result<Option<ToolLocation>> {
        let versions = self.manager.list_store_versions(tool_name)?;
        // Return the latest version (last after sort)
        for version in versions.iter().rev() {
            // Try platform-specific directory (new structure)
            let platform_dir = self.manager.platform_store_dir(tool_name, version);
            if let Some(path) = self.find_executable_in_dir(&platform_dir, exe_name) {
                return Ok(Some(ToolLocation {
                    path,
                    version: version.clone(),
                    source: ToolSource::Store,
                }));
            }
        }
        Ok(None)
    }

    /// Find all versions of a tool in the store directory
    pub fn find_all_in_store(&self, tool_name: &str) -> Result<Vec<ToolLocation>> {
        self.find_all_in_store_with_exe(tool_name, tool_name)
    }

    /// Find all versions of a tool in the store directory with a specific executable name
    ///
    /// This method uses the new directory structure:
    /// - New (post-platform-redirection): <provider>/<version>/<platform>/
    pub fn find_all_in_store_with_exe(
        &self,
        tool_name: &str,
        exe_name: &str,
    ) -> Result<Vec<ToolLocation>> {
        let mut locations = Vec::new();
        let versions = self.manager.list_store_versions(tool_name)?;

        for version in &versions {
            // Try platform-specific directory (new structure)
            let platform_dir = self.manager.platform_store_dir(tool_name, version);
            if let Some(path) = self.find_executable_in_dir(&platform_dir, exe_name) {
                locations.push(ToolLocation {
                    path,
                    version: version.clone(),
                    source: ToolSource::Store,
                });
            }
        }

        Ok(locations)
    }

    // ========== npm-tools Directory Methods ==========

    /// Find a tool in the npm-tools directory
    pub fn find_in_npm_tools(&self, tool_name: &str) -> Result<Option<ToolLocation>> {
        let versions = self.manager.list_npm_tool_versions(tool_name)?;
        // Return the latest version
        for version in versions.iter().rev() {
            let bin_dir = self.manager.npm_tool_bin_dir(tool_name, version);
            if let Some(path) = self.find_npm_executable(&bin_dir, tool_name) {
                return Ok(Some(ToolLocation {
                    path,
                    version: version.clone(),
                    source: ToolSource::NpmTools,
                }));
            }
        }
        Ok(None)
    }

    /// Find all versions of a tool in the npm-tools directory
    pub fn find_all_in_npm_tools(&self, tool_name: &str) -> Result<Vec<ToolLocation>> {
        let mut locations = Vec::new();
        let versions = self.manager.list_npm_tool_versions(tool_name)?;

        for version in versions {
            let bin_dir = self.manager.npm_tool_bin_dir(tool_name, &version);
            if let Some(path) = self.find_npm_executable(&bin_dir, tool_name) {
                locations.push(ToolLocation {
                    path,
                    version,
                    source: ToolSource::NpmTools,
                });
            }
        }

        Ok(locations)
    }

    // ========== pip-tools Directory Methods ==========

    /// Find a tool in the pip-tools directory
    pub fn find_in_pip_tools(&self, tool_name: &str) -> Result<Option<ToolLocation>> {
        let versions = self.manager.list_pip_tool_versions(tool_name)?;
        // Return the latest version
        for version in versions.iter().rev() {
            let bin_dir = self.manager.pip_tool_bin_dir(tool_name, version);
            if let Some(path) = self.find_pip_executable(&bin_dir, tool_name) {
                return Ok(Some(ToolLocation {
                    path,
                    version: version.clone(),
                    source: ToolSource::PipTools,
                }));
            }
        }
        Ok(None)
    }

    /// Find all versions of a tool in the pip-tools directory
    pub fn find_all_in_pip_tools(&self, tool_name: &str) -> Result<Vec<ToolLocation>> {
        let mut locations = Vec::new();
        let versions = self.manager.list_pip_tool_versions(tool_name)?;

        for version in versions {
            let bin_dir = self.manager.pip_tool_bin_dir(tool_name, &version);
            if let Some(path) = self.find_pip_executable(&bin_dir, tool_name) {
                locations.push(ToolLocation {
                    path,
                    version,
                    source: ToolSource::PipTools,
                });
            }
        }

        Ok(locations)
    }

    // ========== conda-tools Directory Methods ==========

    /// Find a tool in the conda-tools directory
    pub fn find_in_conda_tools(&self, tool_name: &str) -> Result<Option<ToolLocation>> {
        let versions = self.manager.list_conda_tool_versions(tool_name)?;
        // Return the latest version
        for version in versions.iter().rev() {
            let bin_dir = self.manager.conda_tool_bin_dir(tool_name, version);
            if let Some(path) = self.find_conda_executable(&bin_dir, tool_name) {
                return Ok(Some(ToolLocation {
                    path,
                    version: version.clone(),
                    source: ToolSource::CondaTools,
                }));
            }
        }
        Ok(None)
    }

    /// Find all versions of a tool in the conda-tools directory
    pub fn find_all_in_conda_tools(&self, tool_name: &str) -> Result<Vec<ToolLocation>> {
        let mut locations = Vec::new();
        let versions = self.manager.list_conda_tool_versions(tool_name)?;

        for version in versions {
            let bin_dir = self.manager.conda_tool_bin_dir(tool_name, &version);
            if let Some(path) = self.find_conda_executable(&bin_dir, tool_name) {
                locations.push(ToolLocation {
                    path,
                    version,
                    source: ToolSource::CondaTools,
                });
            }
        }

        Ok(locations)
    }

    // ========== Legacy API (for backward compatibility) ==========

    /// Find all executable paths for a tool (all versions)
    pub fn find_tool_executables(&self, tool_name: &str) -> Result<Vec<PathBuf>> {
        let locations = self.find_all_tool_installations(tool_name)?;
        Ok(locations.into_iter().map(|loc| loc.path).collect())
    }

    /// Find all executables for a tool with a specific executable name
    ///
    /// # Arguments
    /// * `tool_name` - The runtime/tool name (used for directory lookup)
    /// * `exe_name` - The executable name to search for
    pub fn find_tool_executables_with_exe(
        &self,
        tool_name: &str,
        exe_name: &str,
    ) -> Result<Vec<PathBuf>> {
        let locations = self.find_all_tool_installations_with_exe(tool_name, exe_name)?;
        Ok(locations.into_iter().map(|loc| loc.path).collect())
    }

    /// Find the latest executable for a tool
    pub fn find_latest_executable(&self, tool_name: &str) -> Result<Option<PathBuf>> {
        Ok(self.find_latest_tool(tool_name)?.map(|loc| loc.path))
    }

    /// Find the latest executable for a tool when the executable name differs
    pub fn find_latest_executable_with_exe(
        &self,
        tool_name: &str,
        exe_name: &str,
    ) -> Result<Option<PathBuf>> {
        let locations = self.find_all_tool_installations_with_exe(tool_name, exe_name)?;
        Ok(locations.into_iter().last().map(|loc| loc.path))
    }

    /// Find executable for a specific tool version
    pub fn find_version_executable(&self, tool_name: &str, version: &str) -> Option<PathBuf> {
        self.find_tool_version(tool_name, version)
            .map(|loc| loc.path)
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

    // ========== Internal Helper Methods ==========

    /// Find npm executable in a bin directory
    fn find_npm_executable(&self, bin_dir: &Path, tool_name: &str) -> Option<PathBuf> {
        let exe_name = if cfg!(windows) {
            format!("{}.cmd", tool_name)
        } else {
            tool_name.to_string()
        };
        let exe_path = bin_dir.join(&exe_name);
        if exe_path.exists() {
            Some(exe_path)
        } else {
            None
        }
    }

    /// Find pip executable in a bin directory
    fn find_pip_executable(&self, bin_dir: &Path, tool_name: &str) -> Option<PathBuf> {
        let exe_name = if cfg!(windows) {
            format!("{}.exe", tool_name)
        } else {
            tool_name.to_string()
        };
        let exe_path = bin_dir.join(&exe_name);
        if exe_path.exists() {
            Some(exe_path)
        } else {
            None
        }
    }

    /// Find conda executable in a bin directory
    fn find_conda_executable(&self, bin_dir: &Path, tool_name: &str) -> Option<PathBuf> {
        // Conda environments have similar structure to pip venvs
        let exe_name = if cfg!(windows) {
            format!("{}.exe", tool_name)
        } else {
            tool_name.to_string()
        };
        let exe_path = bin_dir.join(&exe_name);
        if exe_path.exists() {
            Some(exe_path)
        } else {
            None
        }
    }

    /// Search for an executable in a directory (recursively, up to 8 levels)
    /// This handles various archive structures:
    /// - Direct: ~/.vx/store/uv/0.9.17/uv
    /// - One level: ~/.vx/store/uv/0.9.17/uv-platform/uv
    /// - Two levels: ~/.vx/store/go/1.25.5/go/bin/go
    /// - Deep: ~/.vx/store/msvc/14.42/VC/Tools/MSVC/14.42.34433/bin/Hostx64/x64/cl.exe
    /// - Platform-suffixed: ~/.vx/store/rcedit/2.0.0/rcedit-x64.exe
    pub fn find_executable_in_dir(&self, dir: &Path, exe_name: &str) -> Option<PathBuf> {
        if !dir.exists() {
            tracing::trace!(
                "find_executable_in_dir: directory does not exist: {}",
                dir.display()
            );
            return None;
        }

        tracing::trace!(
            "find_executable_in_dir: searching for '{}' in {}",
            exe_name,
            dir.display()
        );

        // Build list of possible executable names in priority order
        // On Windows, .exe and .cmd should be preferred over extensionless files
        // because extensionless files are typically shell scripts
        //
        // Also include platform-suffixed variants (e.g., rcedit-x64.exe for rcedit)
        let possible_names: Vec<String> = if cfg!(windows) {
            vec![
                format!("{}.exe", exe_name),
                format!("{}.cmd", exe_name),
                exe_name.to_string(),
            ]
        } else {
            vec![exe_name.to_string()]
        };

        // Platform-suffixed patterns (e.g., rcedit-x64, rcedit-arm64)
        let platform_suffixes = ["x64", "x86", "arm64", "aarch64", "x86_64"];
        let platform_patterns: Vec<String> = if cfg!(windows) {
            platform_suffixes
                .iter()
                .map(|suffix| format!("{}-{}.exe", exe_name, suffix))
                .collect()
        } else {
            platform_suffixes
                .iter()
                .map(|suffix| format!("{}-{}", exe_name, suffix))
                .collect()
        };

        // Collect all matching files using walkdir for deep search
        let mut all_candidates: Vec<PathBuf> = Vec::new();
        let mut platform_candidates: Vec<PathBuf> = Vec::new();

        // Use walkdir for recursive search with max depth of 8
        // This handles deep directory structures like MSVC
        for entry in walkdir::WalkDir::new(dir)
            .max_depth(8)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.is_file() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if possible_names.iter().any(|n| n == name) {
                        all_candidates.push(path.to_path_buf());
                    } else if platform_patterns.iter().any(|p| p == name) {
                        platform_candidates.push(path.to_path_buf());
                    }
                }
            }
        }

        // Prefer exact matches over platform-suffixed matches
        let result = Self::find_best_match(&all_candidates, &possible_names)
            .or_else(|| Self::find_best_match(&platform_candidates, &platform_patterns));

        if let Some(ref path) = result {
            tracing::trace!(
                "find_executable_in_dir: found executable at {}",
                path.display()
            );
        } else {
            tracing::trace!(
                "find_executable_in_dir: no executable found for '{}' in {} (candidates: {}, platform_candidates: {})",
                exe_name,
                dir.display(),
                all_candidates.len(),
                platform_candidates.len()
            );
        }

        result
    }

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

        // Create a tool installation in platform-specific store directory
        let platform_dir = resolver.manager().platform_store_dir("node", "18.17.0");
        std::fs::create_dir_all(&platform_dir).unwrap();
        let exe_name = if cfg!(windows) { "node.exe" } else { "node" };
        let exe_path = platform_dir.join(exe_name);
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

    #[test]
    fn test_deep_executable_search() {
        let temp_dir = TempDir::new().unwrap();
        let manager = PathManager::with_base_dir(temp_dir.path()).unwrap();
        let resolver = PathResolver::new(manager);

        // Create a deep directory structure like MSVC
        // msvc/14.42/<platform>/VC/Tools/MSVC/14.42.34433/bin/Hostx64/x64/cl.exe
        let _version_dir = resolver.manager().version_store_dir("msvc", "14.42");
        let platform_dir = resolver.manager().platform_store_dir("msvc", "14.42");
        let deep_dir = platform_dir.join("VC/Tools/MSVC/14.42.34433/bin/Hostx64/x64");
        std::fs::create_dir_all(&deep_dir).unwrap();
        let exe_name = if cfg!(windows) { "cl.exe" } else { "cl" };
        let exe_path = deep_dir.join(exe_name);
        std::fs::write(&exe_path, "fake executable").unwrap();

        // Should find the executable using find_executable_in_dir
        let found = resolver.find_executable_in_dir(&platform_dir, "cl");
        assert_eq!(found, Some(exe_path.clone()));

        // Should find using find_tool_executables_with_exe
        let executables = resolver
            .find_tool_executables_with_exe("msvc", "cl")
            .unwrap();
        assert_eq!(executables, vec![exe_path]);
    }

    #[test]
    fn test_platform_suffixed_executable_search() {
        let temp_dir = TempDir::new().unwrap();
        let manager = PathManager::with_base_dir(temp_dir.path()).unwrap();
        let resolver = PathResolver::new(manager);

        // Create a tool with platform-suffixed executable (like rcedit)
        // rcedit/2.0.0/<platform>/rcedit-x64.exe
        let version_dir = resolver.manager().version_store_dir("rcedit", "2.0.0");
        let platform_dir = resolver.manager().platform_store_dir("rcedit", "2.0.0");
        std::fs::create_dir_all(&platform_dir).unwrap();
        let exe_name = if cfg!(windows) {
            "rcedit-x64.exe"
        } else {
            "rcedit-x64"
        };
        let exe_path = platform_dir.join(exe_name);
        std::fs::write(&exe_path, "fake executable").unwrap();

        // Should find the platform-suffixed executable when searching for "rcedit"
        let found = resolver.find_executable_in_dir(&version_dir, "rcedit");
        assert_eq!(found, Some(exe_path.clone()));

        // Should find using find_in_store
        let location = resolver.find_in_store("rcedit").unwrap();
        assert!(location.is_some());
        assert_eq!(location.unwrap().path, exe_path);
    }

    #[test]
    fn test_exact_match_preferred_over_platform_suffix() {
        let temp_dir = TempDir::new().unwrap();
        let manager = PathManager::with_base_dir(temp_dir.path()).unwrap();
        let resolver = PathResolver::new(manager);

        // Create both exact and platform-suffixed executables
        let version_dir = resolver.manager().version_store_dir("mytool", "1.0.0");
        std::fs::create_dir_all(&version_dir).unwrap();

        let exact_name = if cfg!(windows) {
            "mytool.exe"
        } else {
            "mytool"
        };
        let suffixed_name = if cfg!(windows) {
            "mytool-x64.exe"
        } else {
            "mytool-x64"
        };

        let exact_path = version_dir.join(exact_name);
        let suffixed_path = version_dir.join(suffixed_name);
        std::fs::write(&exact_path, "exact executable").unwrap();
        std::fs::write(&suffixed_path, "suffixed executable").unwrap();

        // Should prefer exact match over platform-suffixed
        let found = resolver.find_executable_in_dir(&version_dir, "mytool");
        assert_eq!(found, Some(exact_path));
    }
}
