//! Shim creation and management utilities

use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

use crate::config::ShimConfig;

/// Shim file manager for creating and managing shim files
#[allow(dead_code)]
pub struct ShimManager {
    shim_dir: PathBuf,
}

#[allow(dead_code)]
impl ShimManager {
    /// Create a new shim manager for the given directory
    pub fn new<P: AsRef<Path>>(shim_dir: P) -> Self {
        Self {
            shim_dir: shim_dir.as_ref().to_path_buf(),
        }
    }

    /// Create a shim file for the given target executable
    pub fn create_shim<P: AsRef<Path>>(
        &self,
        shim_name: &str,
        target_path: P,
        args: Option<&str>,
    ) -> Result<PathBuf> {
        let shim_path = self.shim_dir.join(format!("{}.shim", shim_name));

        let config = ShimConfig {
            path: target_path.as_ref().to_string_lossy().to_string(),
            args: args.map(|s| vec![s.to_string()]),
            working_dir: None,
            env: None,
            hide_console: None,
            run_as_admin: None,
            signal_handling: None,
        };

        let content = self.serialize_config(&config)?;
        fs::write(&shim_path, content)
            .with_context(|| format!("Failed to write shim file: {}", shim_path.display()))?;

        Ok(shim_path)
    }

    /// Remove a shim and its executable
    pub fn remove_shim(&self, shim_name: &str) -> Result<()> {
        let shim_path = self.shim_dir.join(format!("{}.shim", shim_name));
        let exe_name = if cfg!(windows) {
            format!("{}.exe", shim_name)
        } else {
            shim_name.to_string()
        };
        let exe_path = self.shim_dir.join(&exe_name);

        // Remove shim file
        if shim_path.exists() {
            fs::remove_file(&shim_path)
                .with_context(|| format!("Failed to remove shim file: {}", shim_path.display()))?;
        }

        // Remove executable
        if exe_path.exists() {
            fs::remove_file(&exe_path).with_context(|| {
                format!("Failed to remove shim executable: {}", exe_path.display())
            })?;
        }

        Ok(())
    }

    /// List all shims in the directory
    pub fn list_shims(&self) -> Result<Vec<String>> {
        let mut shims = Vec::new();

        if !self.shim_dir.exists() {
            return Ok(shims);
        }

        for entry in fs::read_dir(&self.shim_dir)? {
            let entry = entry?;
            let path = entry.path();

            if let Some(extension) = path.extension() {
                if extension == "shim" {
                    if let Some(stem) = path.file_stem() {
                        shims.push(stem.to_string_lossy().to_string());
                    }
                }
            }
        }

        shims.sort();
        Ok(shims)
    }

    /// Serialize configuration to string format
    fn serialize_config(&self, config: &ShimConfig) -> Result<String> {
        // Use TOML format for better structure and readability
        toml::to_string(config).context("Failed to serialize shim configuration")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_shim_manager_creation() {
        let temp_dir = TempDir::new().unwrap();
        let manager = ShimManager::new(temp_dir.path());
        assert_eq!(manager.shim_dir, temp_dir.path());
    }

    #[test]
    fn test_create_shim() {
        let temp_dir = TempDir::new().unwrap();
        let manager = ShimManager::new(temp_dir.path());

        let shim_path = manager
            .create_shim("test", "/bin/echo", Some("hello"))
            .unwrap();

        assert!(shim_path.exists());
        assert_eq!(shim_path.file_name().unwrap(), "test.shim");

        let content = fs::read_to_string(&shim_path).unwrap();
        assert!(content.contains("path = \"/bin/echo\""));
        assert!(content.contains("args = [\"hello\"]"));
    }

    #[test]
    fn test_list_shims() {
        let temp_dir = TempDir::new().unwrap();
        let manager = ShimManager::new(temp_dir.path());

        // Create some test shims
        manager.create_shim("test1", "/bin/echo", None).unwrap();
        manager.create_shim("test2", "/bin/cat", None).unwrap();

        let shims = manager.list_shims().unwrap();
        assert_eq!(shims, vec!["test1", "test2"]);
    }

    #[test]
    fn test_remove_shim() {
        let temp_dir = TempDir::new().unwrap();
        let manager = ShimManager::new(temp_dir.path());

        // Create a test shim
        let shim_path = manager.create_shim("test", "/bin/echo", None).unwrap();
        assert!(shim_path.exists());

        // Remove the shim
        manager.remove_shim("test").unwrap();
        assert!(!shim_path.exists());
    }
}
