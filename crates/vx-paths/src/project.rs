//! Project configuration file discovery
//!
//! This module provides utilities for finding vx configuration files
//! in project directories, and defines all project-related path constants.

use std::path::{Path, PathBuf};

// ============================================
// Configuration File Constants
// ============================================

/// Primary vx configuration file name (preferred)
pub const CONFIG_FILE_NAME: &str = "vx.toml";

/// Legacy vx configuration file name (for backward compatibility)
pub const CONFIG_FILE_NAME_LEGACY: &str = ".vx.toml";

/// Standard vx configuration file names in order of preference
pub const CONFIG_NAMES: &[&str] = &[CONFIG_FILE_NAME, CONFIG_FILE_NAME_LEGACY];

// ============================================
// Project Directory Constants
// ============================================

/// Project-local vx directory name
pub const PROJECT_VX_DIR: &str = ".vx";

/// Project environment directory (relative to project root)
pub const PROJECT_ENV_DIR: &str = ".vx/env";

/// Project cache directory (relative to project root)
pub const PROJECT_CACHE_DIR: &str = ".vx/cache";

/// Project bin directory (relative to project root)
pub const PROJECT_BIN_DIR: &str = ".vx/bin";

// ============================================
// Lock File Constants
// ============================================

/// Lock file name
pub const LOCK_FILE_NAME: &str = "vx.lock";

/// Legacy lock file name
pub const LOCK_FILE_NAME_LEGACY: &str = ".vx.lock";

/// Lock file names in order of preference
pub const LOCK_FILE_NAMES: &[&str] = &[LOCK_FILE_NAME, LOCK_FILE_NAME_LEGACY];

// ============================================
// Functions
// ============================================

/// Find vx config file in a directory
///
/// Searches for config files in order of preference: `vx.toml`, `.vx.toml`
///
/// # Example
/// ```
/// use std::path::Path;
/// use vx_paths::project::find_config_file;
///
/// let config = find_config_file(Path::new("."));
/// if let Some(path) = config {
///     println!("Found config at: {}", path.display());
/// }
/// ```
pub fn find_config_file(dir: &Path) -> Option<PathBuf> {
    for name in CONFIG_NAMES {
        let path = dir.join(name);
        if path.exists() {
            return Some(path);
        }
    }
    None
}

/// Find vx config file by searching up the directory tree
///
/// Starts from the given directory and searches parent directories
/// until a config file is found or the root is reached.
///
/// # Example
/// ```
/// use std::path::Path;
/// use vx_paths::project::find_config_file_upward;
///
/// let config = find_config_file_upward(Path::new("."));
/// if let Some(path) = config {
///     println!("Found config at: {}", path.display());
/// }
/// ```
pub fn find_config_file_upward(start_dir: &Path) -> Option<PathBuf> {
    let mut current = start_dir.to_path_buf();

    loop {
        if let Some(config) = find_config_file(&current) {
            return Some(config);
        }

        if !current.pop() {
            return None;
        }
    }
}

/// Get the project root directory (directory containing vx.toml)
///
/// Searches upward from the given directory.
pub fn find_project_root(start_dir: &Path) -> Option<PathBuf> {
    find_config_file_upward(start_dir).map(|config| {
        config
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| start_dir.to_path_buf())
    })
}

/// Get the project environment directory path
///
/// Returns the `.vx/env` directory path for a project root.
pub fn project_env_dir(project_root: &Path) -> PathBuf {
    project_root.join(PROJECT_ENV_DIR)
}

/// Check if the current directory is inside a vx project
pub fn is_in_vx_project(dir: &Path) -> bool {
    find_config_file_upward(dir).is_some()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_find_config_file_vx_toml() {
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("vx.toml");
        fs::write(&config_path, "[runtimes]").unwrap();

        let found = find_config_file(dir.path());
        assert_eq!(found, Some(config_path));
    }

    #[test]
    fn test_find_config_file_dot_vx_toml() {
        let dir = tempdir().unwrap();
        let config_path = dir.path().join(".vx.toml");
        fs::write(&config_path, "[runtimes]").unwrap();

        let found = find_config_file(dir.path());
        assert_eq!(found, Some(config_path));
    }

    #[test]
    fn test_find_config_file_prefers_vx_toml() {
        let dir = tempdir().unwrap();
        let vx_toml = dir.path().join("vx.toml");
        let dot_vx_toml = dir.path().join(".vx.toml");
        fs::write(&vx_toml, "[runtimes]").unwrap();
        fs::write(&dot_vx_toml, "[runtimes]").unwrap();

        let found = find_config_file(dir.path());
        assert_eq!(found, Some(vx_toml));
    }

    #[test]
    fn test_find_config_file_not_found() {
        let dir = tempdir().unwrap();
        let found = find_config_file(dir.path());
        assert_eq!(found, None);
    }

    #[test]
    fn test_find_config_file_upward() {
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("vx.toml");
        fs::write(&config_path, "[runtimes]").unwrap();

        let subdir = dir.path().join("src").join("nested");
        fs::create_dir_all(&subdir).unwrap();

        let found = find_config_file_upward(&subdir);
        assert_eq!(found, Some(config_path));
    }

    #[test]
    fn test_find_project_root() {
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("vx.toml");
        fs::write(&config_path, "[runtimes]").unwrap();

        let subdir = dir.path().join("src");
        fs::create_dir_all(&subdir).unwrap();

        let root = find_project_root(&subdir);
        assert_eq!(root, Some(dir.path().to_path_buf()));
    }

    #[test]
    fn test_project_env_dir() {
        let project_root = PathBuf::from("/project");
        let env_dir = project_env_dir(&project_root);
        assert_eq!(env_dir, PathBuf::from("/project/.vx/env"));
    }

    #[test]
    fn test_is_in_vx_project() {
        let dir = tempdir().unwrap();
        // Use find_config_file for direct check (not upward search)
        assert!(find_config_file(dir.path()).is_none());

        let config_path = dir.path().join("vx.toml");
        fs::write(&config_path, "[runtimes]").unwrap();
        assert!(find_config_file(dir.path()).is_some());
        assert!(is_in_vx_project(dir.path()));
    }
}
