//! Shim integration for vx tool manager
//!
//! This module provides integration between vx-core and vx-shim,
//! enabling seamless tool version switching through shim technology.

use crate::{Result, VxEnvironment, VxError};
use anyhow::Context;
use std::fs;
use std::path::{Path, PathBuf};
use vx_shim::ShimManager;

/// Manages shim creation and tool version switching for vx
pub struct VxShimManager {
    /// The shim manager instance
    shim_manager: ShimManager,
    /// Path to the shim directory
    shim_dir: PathBuf,
    /// Path to the vx-shim executable
    shim_executable: PathBuf,
    /// Environment instance for configuration
    #[allow(dead_code)]
    environment: VxEnvironment,
}

impl VxShimManager {
    /// Create a new VxShimManager
    pub fn new(environment: VxEnvironment) -> Result<Self> {
        let shim_dir = environment.shim_dir()?;

        // Ensure shim directory exists
        fs::create_dir_all(&shim_dir)
            .with_context(|| format!("Failed to create shim directory: {}", shim_dir.display()))?;

        let shim_manager = ShimManager::new(&shim_dir);

        // Find or build vx-shim executable
        let shim_executable = Self::find_shim_executable(&environment)?;

        Ok(Self {
            shim_manager,
            shim_dir,
            shim_executable,
            environment,
        })
    }

    /// Create a shim for a tool version
    pub fn create_tool_shim(
        &self,
        tool_name: &str,
        tool_path: &Path,
        version: &str,
        args: Option<&str>,
    ) -> Result<PathBuf> {
        // Create shim configuration file
        let shim_config_path = self
            .shim_manager
            .create_shim(tool_name, tool_path, args)
            .with_context(|| format!("Failed to create shim config for {}", tool_name))?;

        // Create the actual shim executable
        let shim_executable_path = self.create_shim_executable(tool_name)?;

        // Store metadata about this shim
        self.store_shim_metadata(tool_name, version, &shim_config_path, &shim_executable_path)?;

        Ok(shim_executable_path)
    }

    /// Switch tool version by updating shim configuration
    pub fn switch_tool_version(
        &self,
        tool_name: &str,
        new_version: &str,
        tool_path: &Path,
    ) -> Result<()> {
        // Check if shim already exists
        let shim_executable_path = self.shim_dir.join(self.get_executable_name(tool_name));

        if !shim_executable_path.exists() {
            // Create new shim if it doesn't exist
            self.create_tool_shim(tool_name, tool_path, new_version, None)?;
        } else {
            // Update existing shim configuration
            self.update_shim_config(tool_name, tool_path, None)?;
            self.store_shim_metadata(
                tool_name,
                new_version,
                &self.get_shim_config_path(tool_name),
                &shim_executable_path,
            )?;
        }

        Ok(())
    }

    /// List all managed shims
    pub fn list_shims(&self) -> Result<Vec<String>> {
        Ok(self
            .shim_manager
            .list_shims()
            .context("Failed to list shims")?)
    }

    /// Remove a shim
    pub fn remove_shim(&self, tool_name: &str) -> Result<()> {
        // Remove shim configuration
        self.shim_manager
            .remove_shim(tool_name)
            .with_context(|| format!("Failed to remove shim config for {}", tool_name))?;

        // Remove shim executable
        let shim_executable_path = self.shim_dir.join(self.get_executable_name(tool_name));
        if shim_executable_path.exists() {
            fs::remove_file(&shim_executable_path).with_context(|| {
                format!(
                    "Failed to remove shim executable: {}",
                    shim_executable_path.display()
                )
            })?;
        }

        // Remove metadata
        self.remove_shim_metadata(tool_name)?;

        Ok(())
    }

    /// Get the current version for a tool shim
    pub fn get_shim_version(&self, tool_name: &str) -> Result<Option<String>> {
        let metadata = self.load_shim_metadata(tool_name)?;
        Ok(metadata.and_then(|m| m.version))
    }

    /// Get shim directory path
    pub fn shim_dir(&self) -> &Path {
        &self.shim_dir
    }

    /// Create the actual shim executable by copying vx-shim
    fn create_shim_executable(&self, tool_name: &str) -> Result<PathBuf> {
        let shim_executable_path = self.shim_dir.join(self.get_executable_name(tool_name));

        // Copy vx-shim executable to create the tool shim
        fs::copy(&self.shim_executable, &shim_executable_path).with_context(|| {
            format!(
                "Failed to copy shim executable from {} to {}",
                self.shim_executable.display(),
                shim_executable_path.display()
            )
        })?;

        // Make executable on Unix systems
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&shim_executable_path)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&shim_executable_path, perms)?;
        }

        Ok(shim_executable_path)
    }

    /// Update shim configuration
    fn update_shim_config(
        &self,
        tool_name: &str,
        tool_path: &Path,
        args: Option<&str>,
    ) -> Result<()> {
        // Remove existing shim config and create new one
        let _ = self.shim_manager.remove_shim(tool_name);
        self.shim_manager
            .create_shim(tool_name, tool_path, args)
            .with_context(|| format!("Failed to update shim config for {}", tool_name))?;
        Ok(())
    }

    /// Get platform-specific executable name
    fn get_executable_name(&self, tool_name: &str) -> String {
        if cfg!(windows) {
            format!("{}.exe", tool_name)
        } else {
            tool_name.to_string()
        }
    }

    /// Get shim config file path
    fn get_shim_config_path(&self, tool_name: &str) -> PathBuf {
        self.shim_dir.join(format!("{}.shim", tool_name))
    }

    /// Find the vx-shim executable
    fn find_shim_executable(environment: &VxEnvironment) -> Result<PathBuf> {
        // First, try to find vx-shim in the same directory as vx
        if let Ok(current_exe) = std::env::current_exe() {
            if let Some(parent) = current_exe.parent() {
                let shim_path = parent.join(if cfg!(windows) {
                    "vx-shim.exe"
                } else {
                    "vx-shim"
                });
                if shim_path.exists() {
                    return Ok(shim_path);
                }
            }
        }

        // Try to find in vx bin directory
        let bin_dir = environment.bin_dir()?;
        let shim_path = bin_dir.join(if cfg!(windows) {
            "vx-shim.exe"
        } else {
            "vx-shim"
        });
        if shim_path.exists() {
            return Ok(shim_path);
        }

        // Try to find in PATH
        if let Ok(shim_path) = which::which("vx-shim") {
            return Ok(shim_path);
        }

        Err(VxError::ShimNotFound(
            "vx-shim executable not found. Please ensure vx-shim is installed and available."
                .to_string(),
        ))
    }

    /// Store metadata about a shim
    fn store_shim_metadata(
        &self,
        tool_name: &str,
        version: &str,
        config_path: &Path,
        executable_path: &Path,
    ) -> Result<()> {
        let metadata = ShimMetadata {
            tool_name: tool_name.to_string(),
            version: Some(version.to_string()),
            config_path: config_path.to_path_buf(),
            executable_path: executable_path.to_path_buf(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let metadata_path = self.get_metadata_path(tool_name);
        let content = toml::to_string(&metadata).context("Failed to serialize shim metadata")?;

        fs::write(&metadata_path, content).with_context(|| {
            format!("Failed to write shim metadata: {}", metadata_path.display())
        })?;

        Ok(())
    }

    /// Load metadata for a shim
    fn load_shim_metadata(&self, tool_name: &str) -> Result<Option<ShimMetadata>> {
        let metadata_path = self.get_metadata_path(tool_name);

        if !metadata_path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(&metadata_path).with_context(|| {
            format!("Failed to read shim metadata: {}", metadata_path.display())
        })?;

        let metadata: ShimMetadata =
            toml::from_str(&content).context("Failed to parse shim metadata")?;

        Ok(Some(metadata))
    }

    /// Remove metadata for a shim
    fn remove_shim_metadata(&self, tool_name: &str) -> Result<()> {
        let metadata_path = self.get_metadata_path(tool_name);

        if metadata_path.exists() {
            fs::remove_file(&metadata_path).with_context(|| {
                format!(
                    "Failed to remove shim metadata: {}",
                    metadata_path.display()
                )
            })?;
        }

        Ok(())
    }

    /// Get metadata file path for a tool
    fn get_metadata_path(&self, tool_name: &str) -> PathBuf {
        self.shim_dir.join(format!("{}.meta", tool_name))
    }
}

/// Metadata stored for each shim
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct ShimMetadata {
    tool_name: String,
    version: Option<String>,
    config_path: PathBuf,
    executable_path: PathBuf,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_shim_manager_creation() {
        let temp_dir = TempDir::new().unwrap();
        let env = VxEnvironment::new_with_base_dir(temp_dir.path()).unwrap();

        // This test might fail if vx-shim is not available, which is expected
        let result = VxShimManager::new(env);
        // We don't assert success here because vx-shim might not be available in test environment
        println!("Shim manager creation result: {:?}", result.is_ok());
    }

    #[test]
    fn test_executable_name() {
        let temp_dir = TempDir::new().unwrap();
        let env = VxEnvironment::new_with_base_dir(temp_dir.path()).unwrap();

        // Create a mock shim manager for testing
        let shim_dir = temp_dir.path().join("shims");
        fs::create_dir_all(&shim_dir).unwrap();

        let manager = VxShimManager {
            shim_manager: ShimManager::new(&shim_dir),
            shim_dir: shim_dir.clone(),
            shim_executable: shim_dir.join("vx-shim"),
            environment: env,
        };

        if cfg!(windows) {
            assert_eq!(manager.get_executable_name("node"), "node.exe");
        } else {
            assert_eq!(manager.get_executable_name("node"), "node");
        }
    }
}
