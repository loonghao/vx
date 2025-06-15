//! Global tool management system
//!
//! This module provides functionality for managing globally installed tools
//! and tracking their usage by virtual environments.

use crate::{Result, VxEnvironment, VxError};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use tokio::fs;

/// Information about a globally installed tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalToolInfo {
    /// Tool name
    pub name: String,
    /// Installed version
    pub version: String,
    /// Installation path
    pub install_path: PathBuf,
    /// Installation timestamp
    pub installed_at: chrono::DateTime<chrono::Utc>,
    /// Virtual environments that reference this tool
    pub referenced_by: HashSet<String>,
}

/// Dependency tracking for virtual environments
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VenvDependency {
    /// Virtual environment name/path
    pub venv_name: String,
    /// Tools this venv depends on
    pub dependencies: HashSet<String>,
    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Global tool manager for handling tool installations and dependencies
#[derive(Debug)]
pub struct GlobalToolManager {
    /// VX environment for path management
    #[allow(dead_code)]
    env: VxEnvironment,
    /// Path to global tools registry file
    registry_path: PathBuf,
    /// Path to venv dependencies file
    dependencies_path: PathBuf,
}

impl GlobalToolManager {
    /// Create a new global tool manager
    pub fn new() -> Result<Self> {
        let env = VxEnvironment::new()?;
        let data_dir = env
            .get_base_install_dir()
            .parent()
            .ok_or_else(|| VxError::Other {
                message: "Failed to get VX data directory".to_string(),
            })?
            .to_path_buf();

        let registry_path = data_dir.join("global_tools.json");
        let dependencies_path = data_dir.join("venv_dependencies.json");

        Ok(Self {
            env,
            registry_path,
            dependencies_path,
        })
    }

    /// Register a globally installed tool
    pub async fn register_global_tool(
        &self,
        name: &str,
        version: &str,
        install_path: &Path,
    ) -> Result<()> {
        let mut tools = self.load_global_tools().await?;

        let tool_info = GlobalToolInfo {
            name: name.to_string(),
            version: version.to_string(),
            install_path: install_path.to_path_buf(),
            installed_at: chrono::Utc::now(),
            referenced_by: HashSet::new(),
        };

        tools.insert(name.to_string(), tool_info);
        self.save_global_tools(&tools).await?;

        Ok(())
    }

    /// Check if a tool is globally installed
    pub async fn is_tool_installed(&self, name: &str) -> Result<bool> {
        let tools = self.load_global_tools().await?;
        Ok(tools.contains_key(name))
    }

    /// Get information about a globally installed tool
    pub async fn get_tool_info(&self, name: &str) -> Result<Option<GlobalToolInfo>> {
        let tools = self.load_global_tools().await?;
        Ok(tools.get(name).cloned())
    }

    /// List all globally installed tools
    pub async fn list_global_tools(&self) -> Result<Vec<GlobalToolInfo>> {
        let tools = self.load_global_tools().await?;
        Ok(tools.into_values().collect())
    }

    /// Add a virtual environment dependency
    pub async fn add_venv_dependency(&self, venv_name: &str, tool_name: &str) -> Result<()> {
        // Update global tool references
        let mut tools = self.load_global_tools().await?;
        if let Some(tool) = tools.get_mut(tool_name) {
            tool.referenced_by.insert(venv_name.to_string());
        }
        self.save_global_tools(&tools).await?;

        // Update venv dependencies
        let mut dependencies = self.load_venv_dependencies().await?;
        let venv_dep = dependencies
            .entry(venv_name.to_string())
            .or_insert_with(|| VenvDependency {
                venv_name: venv_name.to_string(),
                dependencies: HashSet::new(),
                created_at: chrono::Utc::now(),
            });

        venv_dep.dependencies.insert(tool_name.to_string());
        self.save_venv_dependencies(&dependencies).await?;

        Ok(())
    }

    /// Remove a virtual environment dependency
    pub async fn remove_venv_dependency(&self, venv_name: &str, tool_name: &str) -> Result<()> {
        // Update global tool references
        let mut tools = self.load_global_tools().await?;
        if let Some(tool) = tools.get_mut(tool_name) {
            tool.referenced_by.remove(venv_name);
        }
        self.save_global_tools(&tools).await?;

        // Update venv dependencies
        let mut dependencies = self.load_venv_dependencies().await?;
        if let Some(venv_dep) = dependencies.get_mut(venv_name) {
            venv_dep.dependencies.remove(tool_name);

            // Remove venv entry if no dependencies left
            if venv_dep.dependencies.is_empty() {
                dependencies.remove(venv_name);
            }
        }
        self.save_venv_dependencies(&dependencies).await?;

        Ok(())
    }

    /// Check if a tool can be safely removed (not referenced by any venv)
    pub async fn can_remove_tool(&self, tool_name: &str) -> Result<bool> {
        let tools = self.load_global_tools().await?;
        if let Some(tool) = tools.get(tool_name) {
            Ok(tool.referenced_by.is_empty())
        } else {
            Ok(true) // Tool doesn't exist, can be "removed"
        }
    }

    /// Get virtual environments that depend on a tool
    pub async fn get_tool_dependents(&self, tool_name: &str) -> Result<Vec<String>> {
        let tools = self.load_global_tools().await?;
        if let Some(tool) = tools.get(tool_name) {
            Ok(tool.referenced_by.iter().cloned().collect())
        } else {
            Ok(vec![])
        }
    }

    /// Remove a global tool (only if not referenced)
    pub async fn remove_global_tool(&self, tool_name: &str) -> Result<()> {
        if !self.can_remove_tool(tool_name).await? {
            let dependents = self.get_tool_dependents(tool_name).await?;
            return Err(VxError::Other {
                message: format!(
                    "Cannot remove tool '{}' - it is referenced by virtual environments: {}",
                    tool_name,
                    dependents.join(", ")
                ),
            });
        }

        let mut tools = self.load_global_tools().await?;
        if let Some(tool_info) = tools.remove(tool_name) {
            // Remove the actual installation directory
            if tool_info.install_path.exists() {
                fs::remove_dir_all(&tool_info.install_path)
                    .await
                    .map_err(|e| VxError::Other {
                        message: format!(
                            "Failed to remove tool directory {}: {}",
                            tool_info.install_path.display(),
                            e
                        ),
                    })?;
            }
        }

        self.save_global_tools(&tools).await?;
        Ok(())
    }

    /// Load global tools registry from disk
    async fn load_global_tools(&self) -> Result<HashMap<String, GlobalToolInfo>> {
        if !self.registry_path.exists() {
            return Ok(HashMap::new());
        }

        let content =
            fs::read_to_string(&self.registry_path)
                .await
                .map_err(|e| VxError::Other {
                    message: format!("Failed to read global tools registry: {}", e),
                })?;

        serde_json::from_str(&content).map_err(|e| VxError::Other {
            message: format!("Failed to parse global tools registry: {}", e),
        })
    }

    /// Save global tools registry to disk
    async fn save_global_tools(&self, tools: &HashMap<String, GlobalToolInfo>) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = self.registry_path.parent() {
            fs::create_dir_all(parent)
                .await
                .map_err(|e| VxError::Other {
                    message: format!("Failed to create registry directory: {}", e),
                })?;
        }

        let content = serde_json::to_string_pretty(tools).map_err(|e| VxError::Other {
            message: format!("Failed to serialize global tools registry: {}", e),
        })?;

        fs::write(&self.registry_path, content)
            .await
            .map_err(|e| VxError::Other {
                message: format!("Failed to write global tools registry: {}", e),
            })
    }

    /// Load venv dependencies from disk
    async fn load_venv_dependencies(&self) -> Result<HashMap<String, VenvDependency>> {
        if !self.dependencies_path.exists() {
            return Ok(HashMap::new());
        }

        let content = fs::read_to_string(&self.dependencies_path)
            .await
            .map_err(|e| VxError::Other {
                message: format!("Failed to read venv dependencies: {}", e),
            })?;

        serde_json::from_str(&content).map_err(|e| VxError::Other {
            message: format!("Failed to parse venv dependencies: {}", e),
        })
    }

    /// Save venv dependencies to disk
    async fn save_venv_dependencies(&self, deps: &HashMap<String, VenvDependency>) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = self.dependencies_path.parent() {
            fs::create_dir_all(parent)
                .await
                .map_err(|e| VxError::Other {
                    message: format!("Failed to create dependencies directory: {}", e),
                })?;
        }

        let content = serde_json::to_string_pretty(deps).map_err(|e| VxError::Other {
            message: format!("Failed to serialize venv dependencies: {}", e),
        })?;

        fs::write(&self.dependencies_path, content)
            .await
            .map_err(|e| VxError::Other {
                message: format!("Failed to write venv dependencies: {}", e),
            })
    }
}

impl Default for GlobalToolManager {
    fn default() -> Self {
        Self::new().expect("Failed to create GlobalToolManager")
    }
}
