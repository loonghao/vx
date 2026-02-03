//! Runtime root resolution and environment variable generation
//!
//! This module provides a unified interface for:
//! 1. Resolving the root directory of any runtime installation
//! 2. Generating REZ-like environment variables for runtime execution
//!
//! # REZ-like Environment Variables
//!
//! For each runtime, the following environment variables are generated:
//!
//! | Variable | Description | Example |
//! |----------|-------------|---------|
//! | `VX_{NAME}_ROOT` | Root directory of the runtime installation | `~/.vx/store/node/20.0.0/windows-x64/node-v20.0.0-win-x64` |
//! | `VX_{NAME}_BASE` | Base version directory (without platform) | `~/.vx/store/node/20.0.0` |
//! | `VX_{NAME}_BIN` | Bin directory containing executables | `~/.vx/store/node/20.0.0/windows-x64/node-v20.0.0-win-x64` |
//! | `VX_{NAME}_VERSION` | Resolved version string | `20.0.0` |
//! | `VX_{NAME}_VERSIONS` | All installed versions (colon-separated) | `18.20.0:20.0.0:22.0.0` |
//!
//! # Example
//!
//! ```rust,ignore
//! use vx_paths::{RuntimeRoot, VxPaths};
//!
//! let paths = VxPaths::new()?;
//!
//! // Get runtime root for a specific version
//! if let Some(root) = RuntimeRoot::find("node", "20.0.0", &paths)? {
//!     println!("Node.js root: {}", root.root_dir().display());
//!     println!("Node.js bin: {}", root.bin_dir().display());
//!     
//!     // Generate environment variables
//!     for (key, value) in root.env_vars() {
//!         std::env::set_var(key, value);
//!     }
//! }
//! ```

use crate::{with_executable_extension, PathManager, VxPaths};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Resolved runtime root information
///
/// This struct provides access to the runtime's installation directories
/// and can generate REZ-like environment variables.
#[derive(Debug, Clone)]
pub struct RuntimeRoot {
    /// Runtime name (e.g., "node", "python", "go")
    pub name: String,
    /// Resolved version string
    pub version: String,
    /// Base version directory (~/.vx/store/{name}/{version})
    pub base_dir: PathBuf,
    /// Platform-specific directory (~/.vx/store/{name}/{version}/{platform})
    pub platform_dir: PathBuf,
    /// Root directory containing the actual runtime files
    /// This may be nested within platform_dir for some runtimes
    pub root_dir: PathBuf,
    /// Bin directory containing executables
    pub bin_dir: PathBuf,
    /// Path to the main executable
    pub executable_path: PathBuf,
    /// All installed versions of this runtime
    pub all_versions: Vec<String>,
}

impl RuntimeRoot {
    /// Find runtime root for a specific version
    ///
    /// # Arguments
    /// * `name` - Runtime name (e.g., "node", "python")
    /// * `version` - Version string (e.g., "20.0.0")
    /// * `paths` - VxPaths instance
    ///
    /// # Returns
    /// `Some(RuntimeRoot)` if the version is installed, `None` otherwise
    pub fn find(name: &str, version: &str, paths: &VxPaths) -> anyhow::Result<Option<Self>> {
        let manager = PathManager::from_paths(paths.clone());
        Self::find_with_manager(name, version, &manager)
    }

    /// Find runtime root using a PathManager
    pub fn find_with_manager(
        name: &str,
        version: &str,
        manager: &PathManager,
    ) -> anyhow::Result<Option<Self>> {
        let base_dir = manager.version_store_dir(name, version);
        let platform_dir = manager.platform_store_dir(name, version);

        if !platform_dir.exists() {
            return Ok(None);
        }

        // Find the actual root directory within platform_dir
        // Some runtimes have nested directories (e.g., node-v20.0.0-win-x64)
        let (root_dir, bin_dir, executable_path) = Self::resolve_dirs(&platform_dir, name)?;

        // Get all installed versions
        let all_versions = manager.list_store_versions(name).unwrap_or_default();

        Ok(Some(Self {
            name: name.to_string(),
            version: version.to_string(),
            base_dir,
            platform_dir,
            root_dir,
            bin_dir,
            executable_path,
            all_versions,
        }))
    }

    /// Find the latest installed version of a runtime
    pub fn find_latest(name: &str, paths: &VxPaths) -> anyhow::Result<Option<Self>> {
        let manager = PathManager::from_paths(paths.clone());
        let versions = manager.list_store_versions(name)?;

        if let Some(version) = versions.last() {
            Self::find_with_manager(name, version, &manager)
        } else {
            Ok(None)
        }
    }

    /// Resolve the actual directories within a platform directory
    ///
    /// This handles various installation layouts:
    /// - Direct: platform_dir/bin/node (Unix standard)
    /// - Direct: platform_dir/node.exe (Windows flat)
    /// - Nested: platform_dir/node-v20.0.0-win-x64/node.exe (Node.js style)
    /// - Nested with bin: platform_dir/go/bin/go (Go style)
    fn resolve_dirs(
        platform_dir: &Path,
        runtime_name: &str,
    ) -> anyhow::Result<(PathBuf, PathBuf, PathBuf)> {
        let exe_name = with_executable_extension(runtime_name);

        // Strategy 1: Check for executable directly in platform_dir
        let direct_exe = platform_dir.join(&exe_name);
        if direct_exe.is_file() {
            return Ok((
                platform_dir.to_path_buf(),
                platform_dir.to_path_buf(),
                direct_exe,
            ));
        }

        // Strategy 2: Check for bin/ subdirectory (Unix standard layout)
        let bin_dir = platform_dir.join("bin");
        let bin_exe = bin_dir.join(&exe_name);
        if bin_exe.is_file() {
            return Ok((platform_dir.to_path_buf(), bin_dir, bin_exe));
        }

        // Strategy 3: Search subdirectories (handles nested layouts like Node.js)
        if let Ok(entries) = std::fs::read_dir(platform_dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                let sub_path = entry.path();
                if !sub_path.is_dir() {
                    continue;
                }

                // Check for executable directly in subdirectory
                let sub_exe = sub_path.join(&exe_name);
                if sub_exe.is_file() {
                    return Ok((sub_path.clone(), sub_path, sub_exe));
                }

                // Check for bin/ in subdirectory
                let sub_bin_dir = sub_path.join("bin");
                let sub_bin_exe = sub_bin_dir.join(&exe_name);
                if sub_bin_exe.is_file() {
                    return Ok((sub_path, sub_bin_dir, sub_bin_exe));
                }

                // Check one more level deep (for nested structures like Node.js)
                if let Ok(sub_entries) = std::fs::read_dir(&sub_path) {
                    for sub_entry in sub_entries.filter_map(|e| e.ok()) {
                        let deep_path = sub_entry.path();
                        if !deep_path.is_dir() {
                            continue;
                        }

                        let deep_exe = deep_path.join(&exe_name);
                        if deep_exe.is_file() {
                            return Ok((deep_path.clone(), deep_path, deep_exe));
                        }

                        let deep_bin_dir = deep_path.join("bin");
                        let deep_bin_exe = deep_bin_dir.join(&exe_name);
                        if deep_bin_exe.is_file() {
                            return Ok((deep_path, deep_bin_dir, deep_bin_exe));
                        }
                    }
                }
            }
        }

        // Fallback: return platform_dir as root, even if executable not found
        // This allows callers to handle missing executables
        Ok((
            platform_dir.to_path_buf(),
            platform_dir.join("bin"),
            platform_dir.join("bin").join(&exe_name),
        ))
    }

    /// Get the root directory path
    pub fn root_dir(&self) -> &Path {
        &self.root_dir
    }

    /// Get the bin directory path
    pub fn bin_dir(&self) -> &Path {
        &self.bin_dir
    }

    /// Get the main executable path
    pub fn executable_path(&self) -> &Path {
        &self.executable_path
    }

    /// Check if the executable exists
    pub fn executable_exists(&self) -> bool {
        self.executable_path.exists()
    }

    /// Generate REZ-like environment variables
    ///
    /// Returns a HashMap with the following keys:
    /// - `VX_{NAME}_ROOT` - Root directory
    /// - `VX_{NAME}_BASE` - Base version directory
    /// - `VX_{NAME}_BIN` - Bin directory
    /// - `VX_{NAME}_VERSION` - Version string
    /// - `VX_{NAME}_VERSIONS` - All installed versions (colon-separated)
    pub fn env_vars(&self) -> HashMap<String, String> {
        let name_upper = self.name.to_uppercase().replace('-', "_");
        let sep = if cfg!(windows) { ";" } else { ":" };

        let mut vars = HashMap::new();

        vars.insert(
            format!("VX_{}_ROOT", name_upper),
            self.root_dir.display().to_string(),
        );
        vars.insert(
            format!("VX_{}_BASE", name_upper),
            self.base_dir.display().to_string(),
        );
        vars.insert(
            format!("VX_{}_BIN", name_upper),
            self.bin_dir.display().to_string(),
        );
        vars.insert(format!("VX_{}_VERSION", name_upper), self.version.clone());
        vars.insert(
            format!("VX_{}_VERSIONS", name_upper),
            self.all_versions.join(sep),
        );

        vars
    }

    /// Generate environment variables with a custom prefix
    ///
    /// Useful when the runtime name differs from the provider name
    /// (e.g., "nodejs" vs "node")
    pub fn env_vars_with_prefix(&self, prefix: &str) -> HashMap<String, String> {
        let prefix_upper = prefix.to_uppercase().replace('-', "_");
        let sep = if cfg!(windows) { ";" } else { ":" };

        let mut vars = HashMap::new();

        vars.insert(
            format!("VX_{}_ROOT", prefix_upper),
            self.root_dir.display().to_string(),
        );
        vars.insert(
            format!("VX_{}_BASE", prefix_upper),
            self.base_dir.display().to_string(),
        );
        vars.insert(
            format!("VX_{}_BIN", prefix_upper),
            self.bin_dir.display().to_string(),
        );
        vars.insert(format!("VX_{}_VERSION", prefix_upper), self.version.clone());
        vars.insert(
            format!("VX_{}_VERSIONS", prefix_upper),
            self.all_versions.join(sep),
        );

        vars
    }
}

/// Convenience function to get runtime root
///
/// # Example
///
/// ```rust,ignore
/// use vx_paths::get_runtime_root;
///
/// if let Some(root) = get_runtime_root("node", "20.0.0")? {
///     println!("Node.js root: {}", root.root_dir().display());
/// }
/// ```
pub fn get_runtime_root(name: &str, version: &str) -> anyhow::Result<Option<RuntimeRoot>> {
    let paths = VxPaths::new()?;
    RuntimeRoot::find(name, version, &paths)
}

/// Convenience function to get the latest runtime root
///
/// # Example
///
/// ```rust,ignore
/// use vx_paths::get_latest_runtime_root;
///
/// if let Some(root) = get_latest_runtime_root("node")? {
///     println!("Latest Node.js: {} at {}", root.version, root.root_dir().display());
/// }
/// ```
pub fn get_latest_runtime_root(name: &str) -> anyhow::Result<Option<RuntimeRoot>> {
    let paths = VxPaths::new()?;
    RuntimeRoot::find_latest(name, &paths)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_paths(temp_dir: &TempDir) -> VxPaths {
        VxPaths::with_base_dir(temp_dir.path())
    }

    fn setup_node_installation(paths: &VxPaths, version: &str) {
        let manager = PathManager::from_paths(paths.clone());
        let platform_dir = manager.platform_store_dir("node", version);

        // Create nested Node.js-style layout
        let nested_dir =
            platform_dir.join(format!("node-v{}-{}", version, manager.platform_dir_name()));
        std::fs::create_dir_all(&nested_dir).unwrap();

        let exe_name = with_executable_extension("node");
        let exe_path = nested_dir.join(&exe_name);
        std::fs::write(&exe_path, "fake node").unwrap();

        // Create npm and npx
        for tool in &["npm", "npx"] {
            let tool_exe = with_executable_extension(tool);
            let tool_path = nested_dir.join(&tool_exe);
            std::fs::write(&tool_path, "fake tool").unwrap();
        }
    }

    fn setup_go_installation(paths: &VxPaths, version: &str) {
        let manager = PathManager::from_paths(paths.clone());
        let platform_dir = manager.platform_store_dir("go", version);

        // Create Go-style layout with bin/ subdirectory
        let go_dir = platform_dir.join("go");
        let bin_dir = go_dir.join("bin");
        std::fs::create_dir_all(&bin_dir).unwrap();

        let exe_name = with_executable_extension("go");
        let exe_path = bin_dir.join(&exe_name);
        std::fs::write(&exe_path, "fake go").unwrap();
    }

    #[test]
    fn test_find_node_root() {
        let temp_dir = TempDir::new().unwrap();
        let paths = create_test_paths(&temp_dir);
        setup_node_installation(&paths, "20.0.0");

        let root = RuntimeRoot::find("node", "20.0.0", &paths)
            .unwrap()
            .expect("Should find node root");

        assert_eq!(root.name, "node");
        assert_eq!(root.version, "20.0.0");
        assert!(root.executable_exists());
    }

    #[test]
    fn test_find_go_root() {
        let temp_dir = TempDir::new().unwrap();
        let paths = create_test_paths(&temp_dir);
        setup_go_installation(&paths, "1.21.0");

        let root = RuntimeRoot::find("go", "1.21.0", &paths)
            .unwrap()
            .expect("Should find go root");

        assert_eq!(root.name, "go");
        assert_eq!(root.version, "1.21.0");
        assert!(root.executable_exists());
        // Check that bin_dir ends with "bin" component (cross-platform)
        assert!(
            root.bin_dir()
                .components()
                .next_back()
                .map(|c| c.as_os_str() == "bin")
                .unwrap_or(false),
            "bin_dir should end with 'bin' component: {:?}",
            root.bin_dir()
        );
    }

    #[test]
    fn test_find_latest() {
        let temp_dir = TempDir::new().unwrap();
        let paths = create_test_paths(&temp_dir);
        setup_node_installation(&paths, "18.0.0");
        setup_node_installation(&paths, "20.0.0");
        setup_node_installation(&paths, "22.0.0");

        let root = RuntimeRoot::find_latest("node", &paths)
            .unwrap()
            .expect("Should find latest node");

        assert_eq!(root.version, "22.0.0");
        assert_eq!(root.all_versions.len(), 3);
    }

    #[test]
    fn test_env_vars() {
        let temp_dir = TempDir::new().unwrap();
        let paths = create_test_paths(&temp_dir);
        setup_node_installation(&paths, "20.0.0");

        let root = RuntimeRoot::find("node", "20.0.0", &paths)
            .unwrap()
            .expect("Should find node root");

        let vars = root.env_vars();

        assert!(vars.contains_key("VX_NODE_ROOT"));
        assert!(vars.contains_key("VX_NODE_BASE"));
        assert!(vars.contains_key("VX_NODE_BIN"));
        assert!(vars.contains_key("VX_NODE_VERSION"));
        assert!(vars.contains_key("VX_NODE_VERSIONS"));
        assert_eq!(vars.get("VX_NODE_VERSION"), Some(&"20.0.0".to_string()));
    }

    #[test]
    fn test_env_vars_with_prefix() {
        let temp_dir = TempDir::new().unwrap();
        let paths = create_test_paths(&temp_dir);
        setup_node_installation(&paths, "20.0.0");

        let root = RuntimeRoot::find("node", "20.0.0", &paths)
            .unwrap()
            .expect("Should find node root");

        let vars = root.env_vars_with_prefix("nodejs");

        assert!(vars.contains_key("VX_NODEJS_ROOT"));
        assert!(vars.contains_key("VX_NODEJS_VERSION"));
    }

    #[test]
    fn test_not_installed() {
        let temp_dir = TempDir::new().unwrap();
        let paths = create_test_paths(&temp_dir);

        let root = RuntimeRoot::find("node", "99.99.99", &paths).unwrap();
        assert!(root.is_none());
    }
}
