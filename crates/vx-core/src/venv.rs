// Virtual environment management for vx
// Similar to Python's venv, allows users to enter an isolated environment

use crate::{Result, VxError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::path::PathBuf;

/// Virtual environment configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VenvConfig {
    pub name: String,
    pub tools: HashMap<String, String>, // tool_name -> version
    pub path_entries: Vec<PathBuf>,
    pub env_vars: HashMap<String, String>,
}

/// Virtual environment manager
pub struct VenvManager {
    venvs_dir: PathBuf,
}

impl VenvManager {
    pub fn new() -> Result<Self> {
        // Get venvs directory from VX_HOME or config or use default
        let venvs_dir = if let Ok(vx_home) = env::var("VX_HOME") {
            PathBuf::from(vx_home).join("venvs")
        } else if let Some(config_dir) = dirs::config_dir() {
            config_dir.join("vx").join("venvs")
        } else {
            dirs::home_dir()
                .unwrap_or_else(|| std::env::current_dir().unwrap())
                .join(".vx")
                .join("venvs")
        };

        // Ensure venvs directory exists
        std::fs::create_dir_all(&venvs_dir).map_err(|e| VxError::Other {
            message: format!("Failed to create venvs directory: {}", e),
        })?;

        Ok(Self { venvs_dir })
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
            path_entries: vec![venv_dir.join("bin")],
            env_vars: HashMap::new(),
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
        let venv_config = self.load_venv_config(name)?;

        // Generate activation script
        let mut commands = Vec::new();

        // Set VX_VENV environment variable
        commands.push(format!("export VX_VENV={name}"));

        // Prepend venv bin directory to PATH
        for path_entry in &venv_config.path_entries {
            commands.push(format!("export PATH={}:$PATH", path_entry.display()));
        }

        // Set custom environment variables
        for (key, value) in &venv_config.env_vars {
            commands.push(format!("export {key}={value}"));
        }

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
