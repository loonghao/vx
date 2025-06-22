//! Cross-platform path management for vx tool installations
//!
//! This crate provides a unified interface for managing tool installation paths
//! across different platforms, ensuring consistent directory structures and
//! proper handling of executable file extensions.

use anyhow::Result;
use std::path::{Path, PathBuf};

pub mod config;
pub mod manager;
pub mod resolver;
pub mod shim;

pub use config::PathConfig;
pub use manager::PathManager;
pub use resolver::PathResolver;
pub use shim::ShimManager;

/// Standard vx directory structure
#[derive(Debug, Clone)]
pub struct VxPaths {
    /// Base vx directory (~/.vx)
    pub base_dir: PathBuf,
    /// Tools installation directory (~/.vx/tools)
    pub tools_dir: PathBuf,
    /// Cache directory (~/.vx/cache)
    pub cache_dir: PathBuf,
    /// Configuration directory (~/.vx/config)
    pub config_dir: PathBuf,
    /// Temporary directory (~/.vx/tmp)
    pub tmp_dir: PathBuf,
}

impl VxPaths {
    /// Create VxPaths with default locations
    pub fn new() -> Result<Self> {
        let home_dir = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?;

        let base_dir = home_dir.join(".vx");

        Ok(Self {
            tools_dir: base_dir.join("tools"),
            cache_dir: base_dir.join("cache"),
            config_dir: base_dir.join("config"),
            tmp_dir: base_dir.join("tmp"),
            base_dir,
        })
    }

    /// Create VxPaths with custom base directory
    pub fn with_base_dir<P: AsRef<Path>>(base_dir: P) -> Self {
        let base_dir = base_dir.as_ref().to_path_buf();

        Self {
            tools_dir: base_dir.join("tools"),
            cache_dir: base_dir.join("cache"),
            config_dir: base_dir.join("config"),
            tmp_dir: base_dir.join("tmp"),
            base_dir,
        }
    }

    /// Ensure all directories exist
    pub fn ensure_dirs(&self) -> Result<()> {
        std::fs::create_dir_all(&self.base_dir)?;
        std::fs::create_dir_all(&self.tools_dir)?;
        std::fs::create_dir_all(&self.cache_dir)?;
        std::fs::create_dir_all(&self.config_dir)?;
        std::fs::create_dir_all(&self.tmp_dir)?;
        Ok(())
    }
}

impl Default for VxPaths {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| {
            // Fallback to current directory if home directory is not available
            Self::with_base_dir(".vx")
        })
    }
}

/// Get the executable file extension for the current platform
pub fn executable_extension() -> &'static str {
    if cfg!(target_os = "windows") {
        ".exe"
    } else {
        ""
    }
}

/// Add executable extension to a tool name if needed
pub fn with_executable_extension(tool_name: &str) -> String {
    format!("{}{}", tool_name, executable_extension())
}

/// Custom which function that respects PATHEXT on Windows and supports vx-managed tools
pub fn which_tool(tool_name: &str, use_system_path: bool) -> Result<Option<PathBuf>> {
    // First, try to find in vx-managed tools if not using system path
    if !use_system_path {
        if let Ok(vx_path) = find_vx_managed_tool(tool_name) {
            return Ok(Some(vx_path));
        }
    }

    // Then try system PATH if allowed
    if use_system_path {
        if let Ok(system_path) = find_system_tool(tool_name) {
            return Ok(Some(system_path));
        }
    }

    Ok(None)
}

/// Find a tool in vx-managed installations
pub fn find_vx_managed_tool(tool_name: &str) -> Result<PathBuf> {
    let path_manager = PathManager::new()?;

    // Get all installed versions of the tool
    let versions = path_manager.list_tool_versions(tool_name)?;

    if let Some(latest_version) = versions.first() {
        let tool_path = path_manager.tool_executable_path(tool_name, latest_version);

        // Check if the executable exists with various extensions
        if let Some(existing_path) = find_executable_with_extensions(&tool_path, tool_name) {
            return Ok(existing_path);
        }
    }

    Err(anyhow::anyhow!(
        "Tool {} not found in vx-managed installations",
        tool_name
    ))
}

/// Find a tool in system PATH
pub fn find_system_tool(tool_name: &str) -> Result<PathBuf> {
    // Get PATH environment variable
    let path_env = std::env::var("PATH")
        .map_err(|_| anyhow::anyhow!("PATH environment variable not found"))?;

    let path_separator = if cfg!(target_os = "windows") {
        ";"
    } else {
        ":"
    };

    for path_dir in path_env.split(path_separator) {
        let dir_path = PathBuf::from(path_dir);
        if !dir_path.exists() {
            continue;
        }

        let tool_path = dir_path.join(tool_name);

        // Check if the executable exists with various extensions
        if let Some(existing_path) = find_executable_with_extensions(&tool_path, tool_name) {
            return Ok(existing_path);
        }
    }

    Err(anyhow::anyhow!(
        "Tool {} not found in system PATH",
        tool_name
    ))
}

/// Find executable with various extensions based on PATHEXT (Windows) or standard extensions
pub fn find_executable_with_extensions(base_path: &Path, tool_name: &str) -> Option<PathBuf> {
    let extensions = get_executable_extensions();

    for ext in extensions {
        let full_path = if ext.is_empty() {
            base_path.to_path_buf()
        } else {
            base_path.with_extension(&ext[1..]) // Remove the leading dot
        };

        if full_path.exists() && full_path.is_file() {
            return Some(full_path);
        }

        // Also try with the tool name + extension
        let tool_with_ext = format!("{}{}", tool_name, ext);
        let alt_path = base_path.parent()?.join(tool_with_ext);
        if alt_path.exists() && alt_path.is_file() {
            return Some(alt_path);
        }
    }

    None
}

/// Get executable extensions based on platform
pub fn get_executable_extensions() -> Vec<String> {
    if cfg!(target_os = "windows") {
        // Use PATHEXT environment variable if available, otherwise use defaults
        if let Ok(pathext) = std::env::var("PATHEXT") {
            pathext
                .split(';')
                .filter(|ext| !ext.is_empty())
                .map(|ext| ext.to_lowercase())
                .collect()
        } else {
            // Default Windows executable extensions
            vec![
                ".exe".to_string(),
                ".cmd".to_string(),
                ".bat".to_string(),
                ".com".to_string(),
                ".vbs".to_string(),
                ".vbe".to_string(),
                ".js".to_string(),
                ".jse".to_string(),
                ".wsf".to_string(),
                ".wsh".to_string(),
                ".msc".to_string(),
            ]
        }
    } else {
        // Unix-like systems: try without extension first, then common script extensions
        vec![
            "".to_string(),    // No extension
            ".sh".to_string(), // Shell script
            ".py".to_string(), // Python script
            ".pl".to_string(), // Perl script
            ".rb".to_string(), // Ruby script
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vx_paths_creation() {
        let paths = VxPaths::with_base_dir("/tmp/test-vx");

        assert_eq!(paths.base_dir, PathBuf::from("/tmp/test-vx"));
        assert_eq!(paths.tools_dir, PathBuf::from("/tmp/test-vx/tools"));
        assert_eq!(paths.cache_dir, PathBuf::from("/tmp/test-vx/cache"));
        assert_eq!(paths.config_dir, PathBuf::from("/tmp/test-vx/config"));
        assert_eq!(paths.tmp_dir, PathBuf::from("/tmp/test-vx/tmp"));
    }

    #[test]
    fn test_executable_extension() {
        if cfg!(target_os = "windows") {
            assert_eq!(executable_extension(), ".exe");
            assert_eq!(with_executable_extension("node"), "node.exe");
        } else {
            assert_eq!(executable_extension(), "");
            assert_eq!(with_executable_extension("node"), "node");
        }
    }

    #[test]
    fn test_get_executable_extensions() {
        let extensions = get_executable_extensions();
        assert!(!extensions.is_empty());

        if cfg!(target_os = "windows") {
            assert!(extensions.contains(&".exe".to_string()));
            assert!(extensions.contains(&".cmd".to_string()));
            assert!(extensions.contains(&".bat".to_string()));
        } else {
            assert!(extensions.contains(&"".to_string())); // No extension
            assert!(extensions.contains(&".sh".to_string()));
        }
    }

    #[test]
    fn test_find_executable_with_extensions() {
        use std::fs;
        use tempfile::tempdir;

        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path();

        // Create a test executable
        let test_exe = if cfg!(target_os = "windows") {
            temp_path.join("test.cmd")
        } else {
            temp_path.join("test")
        };

        fs::write(&test_exe, "echo test").unwrap();

        // Test finding the executable
        let base_path = temp_path.join("test");
        let found = find_executable_with_extensions(&base_path, "test");

        assert!(found.is_some());
        assert_eq!(found.unwrap(), test_exe);
    }
}
