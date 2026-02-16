//! Path resolver for finding and resolving tool paths
//!
//! This module provides a unified interface for finding tool executables
//! across all vx-managed directories (store, npm-tools, pip-tools).

use crate::PathManager;
use anyhow::Result;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use vx_cache::ExecPathCache;

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
}

impl std::fmt::Display for ToolSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ToolSource::Store => write!(f, "store"),
            ToolSource::NpmTools => write!(f, "npm-tools"),
            ToolSource::PipTools => write!(f, "pip-tools"),
        }
    }
}

/// Resolves tool paths and finds installed tools
#[derive(Debug)]
pub struct PathResolver {
    manager: PathManager,
    /// Executable path cache (bincode-serialized)
    exec_cache: Mutex<ExecPathCache>,
    /// Cache directory for persisting cache
    cache_dir: Option<PathBuf>,
}

impl PathResolver {
    /// Create a new PathResolver
    pub fn new(manager: PathManager) -> Self {
        Self {
            manager,
            exec_cache: Mutex::new(ExecPathCache::new()),
            cache_dir: None,
        }
    }

    /// Create a PathResolver with default paths
    pub fn default_paths() -> Result<Self> {
        let manager = PathManager::new()?;
        let cache_dir = manager.cache_dir().to_path_buf();
        let exec_cache = ExecPathCache::load(&cache_dir);
        Ok(Self {
            manager,
            exec_cache: Mutex::new(exec_cache),
            cache_dir: Some(cache_dir),
        })
    }

    /// Create a PathResolver with explicit cache directory
    pub fn with_cache_dir(manager: PathManager, cache_dir: PathBuf) -> Self {
        let exec_cache = ExecPathCache::load(&cache_dir);
        Self {
            manager,
            exec_cache: Mutex::new(exec_cache),
            cache_dir: Some(cache_dir),
        }
    }

    /// Persist the exec path cache to disk.
    ///
    /// This should be called after operations that modified the cache (e.g., at the
    /// end of a command execution, or after install/uninstall).
    pub fn save_cache(&self) {
        if let Some(ref cache_dir) = self.cache_dir {
            let cache = self.exec_cache.lock().unwrap();
            if let Err(e) = cache.save(cache_dir) {
                tracing::debug!("Failed to save exec path cache: {}", e);
            }
        }
    }

    /// Invalidate cache entries for a specific runtime.
    ///
    /// Call this after installing or uninstalling a runtime version.
    pub fn invalidate_runtime_cache(&self, runtime_name: &str) {
        let runtime_store_dir = self.manager.runtime_store_dir(runtime_name);
        let mut cache = self.exec_cache.lock().unwrap();
        cache.invalidate_runtime(&runtime_store_dir);
        // Persist immediately after invalidation
        if let Some(ref cache_dir) = self.cache_dir
            && let Err(e) = cache.save(cache_dir)
        {
            tracing::debug!("Failed to save exec path cache after invalidation: {}", e);
        }
    }

    /// Clear the entire exec path cache.
    pub fn clear_exec_cache(&self) {
        let mut cache = self.exec_cache.lock().unwrap();
        cache.clear();
        if let Some(ref cache_dir) = self.cache_dir {
            let _ = ExecPathCache::remove_file(cache_dir);
        }
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
    /// - Fallback: <provider>/<version>/ (for cross-platform tools like vcpkg)
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

            // Fallback: Try version directory directly (for cross-platform tools like vcpkg)
            // Some tools don't use platform-specific subdirectories because they manage
            // multiple platforms/triplets within a single installation
            let version_dir = self.manager.version_store_dir(tool_name, version);
            if version_dir != platform_dir {
                if let Some(path) = self.find_executable_in_dir(&version_dir, exe_name) {
                    return Ok(Some(ToolLocation {
                        path,
                        version: version.clone(),
                        source: ToolSource::Store,
                    }));
                }
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
    /// - Fallback: <provider>/<version>/ (for cross-platform tools like vcpkg)
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
                continue;
            }

            // Fallback: Try version directory directly (for cross-platform tools like vcpkg)
            let version_dir = self.manager.version_store_dir(tool_name, version);
            if version_dir != platform_dir {
                if let Some(path) = self.find_executable_in_dir(&version_dir, exe_name) {
                    locations.push(ToolLocation {
                        path,
                        version: version.clone(),
                        source: ToolSource::Store,
                    });
                }
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

    /// Search for an executable in a directory (recursively, up to 8 levels)
    /// This handles various archive structures:
    /// - Direct: ~/.vx/store/uv/0.9.17/uv
    /// - One level: ~/.vx/store/uv/0.9.17/uv-platform/uv
    /// - Two levels: ~/.vx/store/go/1.25.5/go/bin/go
    /// - Deep: ~/.vx/store/msvc/14.42/VC/Tools/MSVC/14.42.34433/bin/Hostx64/x64/cl.exe
    /// - Platform-suffixed: ~/.vx/store/rcedit/2.0.0/rcedit-x64.exe
    ///
    /// ## Performance optimization
    ///
    /// Uses a two-phase search strategy:
    /// 1. **Quick check** (depth 0-3): Check common locations first (root, bin/, direct subdirs)
    /// 2. **Deep search** (depth 4-8): Full walkdir only if quick check fails, with
    ///    directory filtering to skip known non-target dirs (node_modules, lib, share, etc.)
    pub fn find_executable_in_dir(&self, dir: &Path, exe_name: &str) -> Option<PathBuf> {
        if !dir.exists() {
            tracing::trace!(
                "find_executable_in_dir: directory does not exist: {}",
                dir.display()
            );
            return None;
        }

        // Check cache first
        {
            let mut cache = self.exec_cache.lock().unwrap();
            if let Some(cached) = cache.get(dir, exe_name) {
                tracing::trace!(
                    "find_executable_in_dir: cache hit for '{}' in {} -> {}",
                    exe_name,
                    dir.display(),
                    cached.display()
                );
                return Some(cached);
            }
        }

        tracing::trace!(
            "find_executable_in_dir: searching for '{}' in {}",
            exe_name,
            dir.display()
        );

        // Build list of possible executable names in priority order
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

        // Phase 1: Quick check common locations (depth 0-3) without full walkdir.
        // Most runtimes have their executable in root, bin/, or one-level subdirectory.
        // This avoids traversing deep trees like node_modules/ for the common case.
        if let Some(result) = self.quick_find_executable(dir, &possible_names, &platform_patterns) {
            let mut cache = self.exec_cache.lock().unwrap();
            cache.put(dir, exe_name, result.clone());
            return Some(result);
        }

        // Phase 2: Deep search with directory filtering (depth 4-8).
        // Only reached for unusual layouts (e.g., MSVC deep nesting).
        // Skip known non-target directories to reduce filesystem traversal.
        let mut all_candidates: Vec<PathBuf> = Vec::new();
        let mut platform_candidates: Vec<PathBuf> = Vec::new();

        for entry in walkdir::WalkDir::new(dir)
            .max_depth(8)
            .into_iter()
            .filter_entry(|e| !Self::is_skip_directory(e))
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.is_file()
                && let Some(name) = path.file_name().and_then(|n| n.to_str())
            {
                if possible_names.iter().any(|n| n == name) {
                    all_candidates.push(path.to_path_buf());
                } else if platform_patterns.iter().any(|p| p == name) {
                    platform_candidates.push(path.to_path_buf());
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
            let mut cache = self.exec_cache.lock().unwrap();
            cache.put(dir, exe_name, path.clone());
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

    /// Quick search for executable in common locations (avoids full walkdir).
    ///
    /// Checks these locations in order:
    /// 1. Root directory (e.g., uv.exe directly in platform dir)
    /// 2. bin/ subdirectory (e.g., go/bin/go)
    /// 3. One-level subdirectories (e.g., node-v25.6.0-win-x64/node.exe)
    /// 4. One-level subdirectory + bin/ (e.g., node-v25.6.0-win-x64/bin/node)
    fn quick_find_executable(
        &self,
        dir: &Path,
        possible_names: &[String],
        platform_patterns: &[String],
    ) -> Option<PathBuf> {
        // Check 1: Direct files in root
        if let Some(found) = Self::check_files_in_dir(dir, possible_names) {
            return Some(found);
        }
        if let Some(found) = Self::check_files_in_dir(dir, platform_patterns) {
            return Some(found);
        }

        // Check 2: bin/ subdirectory
        let bin_dir = dir.join("bin");
        if bin_dir.is_dir()
            && let Some(found) = Self::check_files_in_dir(&bin_dir, possible_names)
        {
            return Some(found);
        }

        // Check 3 & 4: One level of subdirectories (and their bin/)
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                let sub_path = entry.path();
                if !sub_path.is_dir() {
                    continue;
                }
                // Skip known non-target directories
                if let Some(name) = sub_path.file_name().and_then(|n| n.to_str())
                    && matches!(
                        name,
                        "node_modules" | "lib" | "share" | "include" | "man" | "doc" | "docs"
                    )
                {
                    continue;
                }
                // Check files in subdirectory
                if let Some(found) = Self::check_files_in_dir(&sub_path, possible_names) {
                    return Some(found);
                }
                if let Some(found) = Self::check_files_in_dir(&sub_path, platform_patterns) {
                    return Some(found);
                }
                // Check subdirectory/bin/
                let sub_bin = sub_path.join("bin");
                if sub_bin.is_dir()
                    && let Some(found) = Self::check_files_in_dir(&sub_bin, possible_names)
                {
                    return Some(found);
                }
            }
        }

        None
    }

    /// Check if any of the named files exist in a directory (no recursion)
    fn check_files_in_dir(dir: &Path, names: &[String]) -> Option<PathBuf> {
        for name in names {
            let candidate = dir.join(name);
            if candidate.is_file() {
                return Some(candidate);
            }
        }
        None
    }

    /// Check if a walkdir entry is a directory we should skip during deep search.
    ///
    /// Skipping these directories significantly reduces filesystem traversal,
    /// especially for Node.js installs which have deep node_modules trees.
    fn is_skip_directory(entry: &walkdir::DirEntry) -> bool {
        if !entry.file_type().is_dir() {
            return false;
        }
        let Some(name) = entry.file_name().to_str() else {
            return false;
        };
        matches!(
            name,
            "node_modules"
                | ".git"
                | ".cache"
                | "__pycache__"
                | "site-packages"
                | "dist-info"
                | "egg-info"
        )
    }

    /// Helper to find the best matching executable from a list of candidates
    /// Returns the one with highest priority:
    /// 1. Match by name priority (lowest index in possible_names)
    /// 2. Prefer shorter paths (closer to root directory) to avoid picking up
    ///    nested copies (e.g., corepack shims in node_modules)
    fn find_best_match(candidates: &[PathBuf], possible_names: &[String]) -> Option<PathBuf> {
        for name in possible_names {
            // Find all candidates matching this name
            let mut matching: Vec<&PathBuf> = candidates
                .iter()
                .filter(|c| c.file_name().and_then(|n| n.to_str()) == Some(name.as_str()))
                .collect();

            if matching.is_empty() {
                continue;
            }

            // Sort by path depth (number of components) - prefer shallower paths
            // This ensures we pick node-v20.20.0-win-x64/npx.cmd over
            // node-v20.20.0-win-x64/node_modules/corepack/shims/nodewin/npx.cmd
            matching.sort_by_key(|p| p.components().count());

            return matching.first().map(|p| (*p).clone());
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
