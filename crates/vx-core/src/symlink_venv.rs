//! Symlink-based virtual environment system
//! 
//! This module provides functionality for creating virtual environments
//! that use symlinks to reference globally installed tools, similar to pnpm.

use crate::{GlobalToolManager, Result, VxError, VxEnvironment};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::fs;

/// Virtual environment information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymlinkVenv {
    /// Virtual environment name
    pub name: String,
    /// Virtual environment path
    pub path: PathBuf,
    /// Tools linked in this venv
    pub linked_tools: HashMap<String, String>, // tool_name -> version
    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Last modified timestamp
    pub modified_at: chrono::DateTime<chrono::Utc>,
}

/// Symlink virtual environment manager
#[derive(Debug)]
pub struct SymlinkVenvManager {
    /// VX environment for path management
    env: VxEnvironment,
    /// Global tool manager for dependency tracking
    global_manager: GlobalToolManager,
    /// Path to venv registry file
    registry_path: PathBuf,
}

impl SymlinkVenvManager {
    /// Create a new symlink venv manager
    pub fn new() -> Result<Self> {
        let env = VxEnvironment::new()?;
        let global_manager = GlobalToolManager::new()?;
        
        let data_dir = env
            .get_base_install_dir()
            .parent()
            .ok_or_else(|| VxError::Other {
                message: "Failed to get VX data directory".to_string(),
            })?
            .to_path_buf();
        
        let registry_path = data_dir.join("symlink_venvs.json");
        
        Ok(Self {
            env,
            global_manager,
            registry_path,
        })
    }

    /// Create a new virtual environment
    pub async fn create_venv(&self, name: &str, path: &Path) -> Result<()> {
        // Check if venv already exists
        if self.venv_exists(name).await? {
            return Err(VxError::Other {
                message: format!("Virtual environment '{}' already exists", name),
            });
        }

        // Create venv directory structure
        fs::create_dir_all(path).await.map_err(|e| VxError::Other {
            message: format!("Failed to create venv directory: {}", e),
        })?;

        // Create bin directory for symlinks
        let bin_dir = path.join("bin");
        fs::create_dir_all(&bin_dir).await.map_err(|e| VxError::Other {
            message: format!("Failed to create venv bin directory: {}", e),
        })?;

        // Create venv info
        let venv = SymlinkVenv {
            name: name.to_string(),
            path: path.to_path_buf(),
            linked_tools: HashMap::new(),
            created_at: chrono::Utc::now(),
            modified_at: chrono::Utc::now(),
        };

        // Register venv
        let mut venvs = self.load_venvs().await?;
        venvs.insert(name.to_string(), venv);
        self.save_venvs(&venvs).await?;

        Ok(())
    }

    /// Link a global tool to a virtual environment
    pub async fn link_tool(&self, venv_name: &str, tool_name: &str, version: &str) -> Result<()> {
        // Check if tool is globally installed
        if !self.global_manager.is_tool_installed(tool_name).await? {
            return Err(VxError::Other {
                message: format!("Tool '{}' is not globally installed", tool_name),
            });
        }

        // Get venv info
        let mut venvs = self.load_venvs().await?;
        let venv = venvs.get_mut(venv_name).ok_or_else(|| VxError::Other {
            message: format!("Virtual environment '{}' not found", venv_name),
        })?;

        // Get global tool installation path
        let global_install_dir = self.env.get_version_install_dir(tool_name, version);
        let global_exe = self.env.find_executable_in_dir(&global_install_dir, tool_name)?;

        // Create symlink in venv bin directory
        let venv_bin_dir = venv.path.join("bin");
        let venv_exe = venv_bin_dir.join(tool_name);

        // Remove existing symlink if it exists
        if venv_exe.exists() {
            fs::remove_file(&venv_exe).await.map_err(|e| VxError::Other {
                message: format!("Failed to remove existing symlink: {}", e),
            })?;
        }

        // Create symlink
        self.create_symlink(&global_exe, &venv_exe).await?;

        // Update venv registry
        venv.linked_tools.insert(tool_name.to_string(), version.to_string());
        venv.modified_at = chrono::Utc::now();

        // Update global tool dependencies
        self.global_manager.add_venv_dependency(venv_name, tool_name).await?;

        // Save changes
        self.save_venvs(&venvs).await?;

        Ok(())
    }

    /// Unlink a tool from a virtual environment
    pub async fn unlink_tool(&self, venv_name: &str, tool_name: &str) -> Result<()> {
        // Get venv info
        let mut venvs = self.load_venvs().await?;
        let venv = venvs.get_mut(venv_name).ok_or_else(|| VxError::Other {
            message: format!("Virtual environment '{}' not found", venv_name),
        })?;

        // Remove symlink
        let venv_exe = venv.path.join("bin").join(tool_name);
        if venv_exe.exists() {
            fs::remove_file(&venv_exe).await.map_err(|e| VxError::Other {
                message: format!("Failed to remove symlink: {}", e),
            })?;
        }

        // Update venv registry
        venv.linked_tools.remove(tool_name);
        venv.modified_at = chrono::Utc::now();

        // Update global tool dependencies
        self.global_manager.remove_venv_dependency(venv_name, tool_name).await?;

        // Save changes
        self.save_venvs(&venvs).await?;

        Ok(())
    }

    /// Remove a virtual environment
    pub async fn remove_venv(&self, name: &str) -> Result<()> {
        let mut venvs = self.load_venvs().await?;
        
        if let Some(venv) = venvs.get(name) {
            // Remove all tool dependencies
            for tool_name in venv.linked_tools.keys() {
                self.global_manager.remove_venv_dependency(name, tool_name).await?;
            }

            // Remove venv directory
            if venv.path.exists() {
                fs::remove_dir_all(&venv.path).await.map_err(|e| VxError::Other {
                    message: format!("Failed to remove venv directory: {}", e),
                })?;
            }

            // Remove from registry
            venvs.remove(name);
            self.save_venvs(&venvs).await?;
        }

        Ok(())
    }

    /// List all virtual environments
    pub async fn list_venvs(&self) -> Result<Vec<SymlinkVenv>> {
        let venvs = self.load_venvs().await?;
        Ok(venvs.into_values().collect())
    }

    /// Get virtual environment info
    pub async fn get_venv(&self, name: &str) -> Result<Option<SymlinkVenv>> {
        let venvs = self.load_venvs().await?;
        Ok(venvs.get(name).cloned())
    }

    /// Check if virtual environment exists
    pub async fn venv_exists(&self, name: &str) -> Result<bool> {
        let venvs = self.load_venvs().await?;
        Ok(venvs.contains_key(name))
    }

    /// Create a symlink (cross-platform)
    async fn create_symlink(&self, target: &Path, link: &Path) -> Result<()> {
        #[cfg(unix)]
        {
            tokio::fs::symlink(target, link).await.map_err(|e| VxError::Other {
                message: format!("Failed to create symlink: {}", e),
            })
        }

        #[cfg(windows)]
        {
            // On Windows, try to create a symlink, but fall back to copying if it fails
            match tokio::fs::symlink_file(target, link).await {
                Ok(()) => Ok(()),
                Err(_) => {
                    // Fall back to copying the file
                    tokio::fs::copy(target, link).await.map_err(|e| VxError::Other {
                        message: format!("Failed to copy file (symlink fallback): {}", e),
                    })?;
                    Ok(())
                }
            }
        }
    }

    /// Load virtual environments registry from disk
    async fn load_venvs(&self) -> Result<HashMap<String, SymlinkVenv>> {
        if !self.registry_path.exists() {
            return Ok(HashMap::new());
        }

        let content = fs::read_to_string(&self.registry_path).await.map_err(|e| {
            VxError::Other {
                message: format!("Failed to read venv registry: {}", e),
            }
        })?;

        serde_json::from_str(&content).map_err(|e| VxError::Other {
            message: format!("Failed to parse venv registry: {}", e),
        })
    }

    /// Save virtual environments registry to disk
    async fn save_venvs(&self, venvs: &HashMap<String, SymlinkVenv>) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = self.registry_path.parent() {
            fs::create_dir_all(parent).await.map_err(|e| VxError::Other {
                message: format!("Failed to create registry directory: {}", e),
            })?;
        }

        let content = serde_json::to_string_pretty(venvs).map_err(|e| VxError::Other {
            message: format!("Failed to serialize venv registry: {}", e),
        })?;

        fs::write(&self.registry_path, content).await.map_err(|e| VxError::Other {
            message: format!("Failed to write venv registry: {}", e),
        })
    }
}

impl Default for SymlinkVenvManager {
    fn default() -> Self {
        Self::new().expect("Failed to create SymlinkVenvManager")
    }
}
