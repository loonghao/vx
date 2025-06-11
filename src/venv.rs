// Virtual environment management for vx
// Similar to Python's venv, allows users to enter an isolated environment

use crate::config_figment::FigmentConfigManager;
use crate::tool_manager::ToolManager;
use anyhow::{anyhow, Result};
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
    #[allow(dead_code)]
    config: FigmentConfigManager,
    #[allow(dead_code)]
    tool_manager: ToolManager,
    venvs_dir: PathBuf,
}

impl VenvManager {
    pub fn new() -> Result<Self> {
        let config = FigmentConfigManager::new()?;
        let tool_manager = ToolManager::new()?;
        
        // Get venvs directory from config or use default
        let venvs_dir = if let Some(config_dir) = dirs::config_dir() {
            config_dir.join("vx").join("venvs")
        } else {
            dirs::home_dir()
                .unwrap_or_else(|| std::env::current_dir().unwrap())
                .join(".vx")
                .join("venvs")
        };
        
        // Ensure venvs directory exists
        std::fs::create_dir_all(&venvs_dir)?;

        Ok(Self {
            config,
            tool_manager,
            venvs_dir,
        })
    }

    /// Create a new virtual environment
    pub fn create(&self, name: &str, tools: &[(String, String)]) -> Result<()> {
        let venv_dir = self.venvs_dir.join(name);
        
        if venv_dir.exists() {
            return Err(anyhow!("Virtual environment '{}' already exists", name));
        }

        // Create venv directory structure
        std::fs::create_dir_all(&venv_dir)?;
        std::fs::create_dir_all(venv_dir.join("bin"))?;
        std::fs::create_dir_all(venv_dir.join("config"))?;

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

        // Install tools for this venv
        for (tool, version) in tools {
            self.install_tool_for_venv(name, tool, version)?;
        }

        println!("Virtual environment '{}' created successfully", name);
        Ok(())
    }

    /// Activate a virtual environment (returns shell commands to execute)
    pub fn activate(&self, name: &str) -> Result<String> {
        let venv_config = self.load_venv_config(name)?;
        
        // Generate activation script
        let mut commands = Vec::new();
        
        // Set VX_VENV environment variable
        commands.push(format!("export VX_VENV={}", name));
        
        // Prepend venv bin directory to PATH
        for path_entry in &venv_config.path_entries {
            commands.push(format!("export PATH={}:$PATH", path_entry.display()));
        }
        
        // Set custom environment variables
        for (key, value) in &venv_config.env_vars {
            commands.push(format!("export {}={}", key, value));
        }
        
        // Set prompt indicator
        commands.push(format!("export PS1=\"(vx:{}) $PS1\"", name));
        
        Ok(commands.join("\n"))
    }

    /// Deactivate current virtual environment
    pub fn deactivate() -> String {
        vec![
            "unset VX_VENV",
            "# Restore original PATH (implementation needed)",
            "# Restore original PS1 (implementation needed)",
        ].join("\n")
    }

    /// List all virtual environments
    pub fn list(&self) -> Result<Vec<String>> {
        let mut venvs = Vec::new();
        
        if self.venvs_dir.exists() {
            for entry in std::fs::read_dir(&self.venvs_dir)? {
                let entry = entry?;
                if entry.file_type()?.is_dir() {
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
            return Err(anyhow!("Virtual environment '{}' does not exist", name));
        }

        std::fs::remove_dir_all(&venv_dir)?;
        println!("Virtual environment '{}' removed", name);
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
    fn install_tool_for_venv(&self, venv_name: &str, tool: &str, version: &str) -> Result<()> {
        // This would integrate with the tool manager to install tools
        // in the venv-specific directory
        println!("Installing {} {} for venv '{}'", tool, version, venv_name);
        // TODO: Implement actual tool installation
        Ok(())
    }

    /// Save virtual environment configuration
    fn save_venv_config(&self, config: &VenvConfig) -> Result<()> {
        let config_path = self.venvs_dir.join(&config.name).join("config").join("venv.toml");
        let toml_content = toml::to_string_pretty(config)?;
        std::fs::write(config_path, toml_content)?;
        Ok(())
    }

    /// Load virtual environment configuration
    fn load_venv_config(&self, name: &str) -> Result<VenvConfig> {
        let config_path = self.venvs_dir.join(name).join("config").join("venv.toml");
        
        if !config_path.exists() {
            return Err(anyhow!("Virtual environment '{}' configuration not found", name));
        }

        let content = std::fs::read_to_string(config_path)?;
        let config: VenvConfig = toml::from_str(&content)?;
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
