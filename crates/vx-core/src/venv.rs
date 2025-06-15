//! Simplified virtual environment management for vx
//!
//! This module provides a simplified virtual environment system that:
//! - Uses transparent proxy approach (no explicit activation needed)
//! - Automatically detects project configuration (.vx.toml)
//! - Manages tool versions through global installation + PATH manipulation
//! - Provides seamless user experience similar to nvm/pnpm

use crate::{GlobalToolManager, Result, VxEnvironment, VxError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::path::PathBuf;

/// Simplified virtual environment configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VenvConfig {
    /// Virtual environment name
    pub name: String,
    /// Tools and their versions
    pub tools: HashMap<String, String>, // tool_name -> version
    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Last modified timestamp
    pub modified_at: chrono::DateTime<chrono::Utc>,
    /// Whether this venv is currently active
    pub is_active: bool,
}

/// Project configuration from .vx.toml
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProjectConfig {
    /// Tools required by this project
    pub tools: HashMap<String, String>,
    /// Project settings
    pub settings: ProjectSettings,
}

/// Project settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectSettings {
    /// Automatically install missing tools
    pub auto_install: bool,
    /// Cache duration for version checks
    pub cache_duration: String,
}

impl Default for ProjectSettings {
    fn default() -> Self {
        Self {
            auto_install: true,
            cache_duration: "7d".to_string(),
        }
    }
}

/// Simplified virtual environment manager
pub struct VenvManager {
    /// VX environment for path management
    env: VxEnvironment,
    /// Global tool manager for tool installation
    #[allow(dead_code)]
    global_manager: GlobalToolManager,
    /// Path to venvs directory
    venvs_dir: PathBuf,
}

impl VenvManager {
    /// Create a new VenvManager instance
    pub fn new() -> Result<Self> {
        let env = VxEnvironment::new()?;
        let global_manager = GlobalToolManager::new()?;

        // Get venvs directory from VX environment
        let venvs_dir = env
            .get_base_install_dir()
            .parent()
            .ok_or_else(|| VxError::Other {
                message: "Failed to get VX data directory".to_string(),
            })?
            .join("venvs");

        // Ensure venvs directory exists
        std::fs::create_dir_all(&venvs_dir).map_err(|e| VxError::Other {
            message: format!("Failed to create venvs directory: {}", e),
        })?;

        Ok(Self {
            env,
            global_manager,
            venvs_dir,
        })
    }

    /// Load project configuration from .vx.toml
    pub fn load_project_config(&self) -> Result<Option<ProjectConfig>> {
        let config_path = std::env::current_dir()
            .map_err(|e| VxError::Other {
                message: format!("Failed to get current directory: {}", e),
            })?
            .join(".vx.toml");

        if !config_path.exists() {
            return Ok(None);
        }

        let content = std::fs::read_to_string(&config_path).map_err(|e| VxError::Other {
            message: format!("Failed to read .vx.toml: {}", e),
        })?;

        let config: ProjectConfig = toml::from_str(&content).map_err(|e| VxError::Other {
            message: format!("Failed to parse .vx.toml: {}", e),
        })?;

        Ok(Some(config))
    }

    /// Get the tool version for the current project context
    pub async fn get_project_tool_version(&self, tool_name: &str) -> Result<Option<String>> {
        // First check project configuration
        if let Some(config) = self.load_project_config()? {
            if let Some(version) = config.tools.get(tool_name) {
                return Ok(Some(version.clone()));
            }
        }

        // TODO: Check for tool-specific version files (.nvmrc, .python-version, etc.)

        Ok(None)
    }

    /// Ensure a tool is available for the current project
    pub async fn ensure_tool_available(&self, tool_name: &str) -> Result<PathBuf> {
        // Get the required version for this project
        let version = self
            .get_project_tool_version(tool_name)
            .await?
            .unwrap_or_else(|| "latest".to_string());

        // Check if the specific version is installed
        let install_dir = self.env.get_version_install_dir(tool_name, &version);
        let is_installed = self.env.is_version_installed(tool_name, &version);

        if !is_installed {
            // Auto-install if enabled
            let should_auto_install = if let Some(config) = self.load_project_config()? {
                config.settings.auto_install
            } else {
                true // Default to auto-install
            };

            if should_auto_install {
                // TODO: Implement auto-installation through plugin system
                return Err(VxError::Other {
                    message: format!(
                        "Tool '{}' version '{}' is not installed. Auto-installation not yet implemented. Run 'vx install {}@{}' to install it.",
                        tool_name, version, tool_name, version
                    ),
                });
            } else {
                return Err(VxError::Other {
                    message: format!(
                        "Tool '{}' version '{}' is not installed. Run 'vx install {}@{}' to install it.",
                        tool_name, version, tool_name, version
                    ),
                });
            }
        }

        // Get the executable path
        self.env.find_executable_in_dir(&install_dir, tool_name)
    }

    /// Create a new virtual environment
    pub fn create(&self, name: &str, tools: &[(String, String)]) -> Result<()> {
        let venv_dir = self.venvs_dir.join(name);

        if venv_dir.exists() {
            return Err(VxError::Other {
                message: format!("Virtual environment '{}' already exists", name),
            });
        }

        // Create venv directory structure
        std::fs::create_dir_all(&venv_dir).map_err(|e| VxError::Other {
            message: format!("Failed to create venv directory: {}", e),
        })?;
        std::fs::create_dir_all(venv_dir.join("bin")).map_err(|e| VxError::Other {
            message: format!("Failed to create bin directory: {}", e),
        })?;
        std::fs::create_dir_all(venv_dir.join("config")).map_err(|e| VxError::Other {
            message: format!("Failed to create config directory: {}", e),
        })?;

        // Create venv configuration
        let mut tool_versions = HashMap::new();
        for (tool, version) in tools {
            tool_versions.insert(tool.clone(), version.clone());
        }

        let venv_config = VenvConfig {
            name: name.to_string(),
            tools: tool_versions,
            created_at: chrono::Utc::now(),
            modified_at: chrono::Utc::now(),
            is_active: false,
        };

        // Save configuration
        self.save_venv_config(&venv_config)?;

        // TODO: Install tools for this venv
        for (tool, version) in tools {
            self.install_tool_for_venv(name, tool, version)?;
        }

        Ok(())
    }

    /// Activate a virtual environment (returns shell commands to execute)
    pub fn activate(&self, name: &str) -> Result<String> {
        let _venv_config = self.load_venv_config(name)?;

        // Generate activation script
        let mut commands = Vec::new();

        // Set VX_VENV environment variable
        commands.push(format!("export VX_VENV={name}"));

        // In the simplified design, we don't modify PATH directly
        // Instead, vx commands will automatically use the correct tool versions
        // based on the project configuration

        // Set prompt indicator
        commands.push(format!("export PS1=\"(vx:{name}) $PS1\""));

        Ok(commands.join("\n"))
    }

    /// Deactivate current virtual environment
    pub fn deactivate() -> String {
        [
            "unset VX_VENV",
            "# Restore original PATH (implementation needed)",
            "# Restore original PS1 (implementation needed)",
        ]
        .join("\n")
    }

    /// List all virtual environments
    pub fn list(&self) -> Result<Vec<String>> {
        let mut venvs = Vec::new();

        if self.venvs_dir.exists() {
            for entry in std::fs::read_dir(&self.venvs_dir).map_err(|e| VxError::Other {
                message: format!("Failed to read venvs directory: {}", e),
            })? {
                let entry = entry.map_err(|e| VxError::Other {
                    message: format!("Failed to read directory entry: {}", e),
                })?;
                if entry
                    .file_type()
                    .map_err(|e| VxError::Other {
                        message: format!("Failed to get file type: {}", e),
                    })?
                    .is_dir()
                {
                    if let Some(name) = entry.file_name().to_str() {
                        venvs.push(name.to_string());
                    }
                }
            }
        }

        venvs.sort();
        Ok(venvs)
    }

    /// Remove a virtual environment
    pub fn remove(&self, name: &str) -> Result<()> {
        let venv_dir = self.venvs_dir.join(name);

        if !venv_dir.exists() {
            return Err(VxError::Other {
                message: format!("Virtual environment '{}' does not exist", name),
            });
        }

        std::fs::remove_dir_all(&venv_dir).map_err(|e| VxError::Other {
            message: format!("Failed to remove venv directory: {}", e),
        })?;
        Ok(())
    }

    /// Get current active virtual environment
    pub fn current() -> Option<String> {
        env::var("VX_VENV").ok()
    }

    /// Check if we're in a virtual environment
    pub fn is_active() -> bool {
        env::var("VX_VENV").is_ok()
    }

    /// Install a tool for a specific virtual environment
    fn install_tool_for_venv(&self, _venv_name: &str, _tool: &str, _version: &str) -> Result<()> {
        // TODO: Implement actual tool installation
        Ok(())
    }

    /// Save virtual environment configuration
    fn save_venv_config(&self, config: &VenvConfig) -> Result<()> {
        let config_dir = self.venvs_dir.join(&config.name).join("config");
        let config_path = config_dir.join("venv.toml");

        // Ensure config directory exists
        std::fs::create_dir_all(&config_dir).map_err(|e| VxError::Other {
            message: format!("Failed to create config directory: {}", e),
        })?;

        let toml_content = toml::to_string_pretty(config).map_err(|e| VxError::Other {
            message: format!("Failed to serialize venv config: {}", e),
        })?;
        std::fs::write(config_path, toml_content).map_err(|e| VxError::Other {
            message: format!("Failed to write venv config: {}", e),
        })?;
        Ok(())
    }

    /// Load virtual environment configuration
    fn load_venv_config(&self, name: &str) -> Result<VenvConfig> {
        let config_path = self.venvs_dir.join(name).join("config").join("venv.toml");

        if !config_path.exists() {
            return Err(VxError::Other {
                message: format!("Virtual environment '{}' configuration not found", name),
            });
        }

        let content = std::fs::read_to_string(config_path).map_err(|e| VxError::Other {
            message: format!("Failed to read venv config: {}", e),
        })?;
        let config: VenvConfig = toml::from_str(&content).map_err(|e| VxError::Other {
            message: format!("Failed to parse venv config: {}", e),
        })?;
        Ok(config)
    }
}

impl Default for VenvManager {
    fn default() -> Self {
        Self::new().expect("Failed to create VenvManager")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_venv_manager_creation() {
        let manager = VenvManager::new();
        assert!(manager.is_ok());
    }

    #[test]
    fn test_current_venv() {
        // Should return None when not in a venv
        assert!(!VenvManager::is_active());
    }
}
