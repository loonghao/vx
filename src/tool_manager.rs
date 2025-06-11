// Tool manager - integrates tools with Figment configuration system
// Provides high-level interface for tool execution and management

use crate::config_figment::FigmentConfigManager;
use crate::tool::ToolInfo;
use crate::tool_registry::ToolRegistry;
use crate::installer::InstallConfig;
use anyhow::{anyhow, Result};
use std::path::PathBuf;

/// Tool manager that integrates tools with configuration
pub struct ToolManager {
    registry: ToolRegistry,
    config: FigmentConfigManager,
    install_base_dir: PathBuf,
}

impl ToolManager {
    /// Create a new tool manager
    pub fn new() -> Result<Self> {
        let install_base_dir = dirs::cache_dir()
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
            .join("vx")
            .join("tools");

        Ok(Self {
            registry: ToolRegistry::new(),
            config: FigmentConfigManager::new()?,
            install_base_dir,
        })
    }

    /// Create a new tool manager with custom config and install directory
    pub fn new_with_config(config: FigmentConfigManager, install_base_dir: PathBuf) -> Self {
        Self {
            registry: ToolRegistry::new(),
            config,
            install_base_dir,
        }
    }

    /// Create a minimal tool manager (for fallback)
    pub fn minimal() -> Result<Self> {
        let install_base_dir = dirs::cache_dir()
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
            .join("vx")
            .join("tools");

        Ok(Self {
            registry: ToolRegistry::new(),
            config: FigmentConfigManager::minimal()?,
            install_base_dir,
        })
    }

    /// Execute a tool with given arguments
    pub fn execute_tool(&self, tool_name: &str, args: &[String]) -> Result<i32> {
        let tool = self
            .registry
            .get(tool_name)
            .ok_or_else(|| anyhow!("Tool '{}' not found", tool_name))?;

        // Check if tool is installed
        if !tool.is_installed()? {
            if tool.supports_auto_install() {
                println!("🔧 Tool '{}' is not installed. Installing...", tool_name);
                self.install_tool(tool_name)?;
            } else {
                return Err(anyhow!(
                    "Tool '{}' is not installed and does not support auto-installation",
                    tool_name
                ));
            }
        }

        // Execute the tool
        tool.execute(args)
    }

    /// Install a tool using the configuration system
    pub fn install_tool(&self, tool_name: &str) -> Result<()> {
        let tool = self
            .registry
            .get(tool_name)
            .ok_or_else(|| anyhow!("Tool '{}' not found", tool_name))?;

        if !tool.supports_auto_install() {
            return Err(anyhow!(
                "Tool '{}' does not support auto-installation",
                tool_name
            ));
        }

        // Get version from configuration
        let version = self
            .config
            .get_tool_config(tool_name)
            .and_then(|config| config.version.clone())
            .unwrap_or_else(|| "latest".to_string());

        // Get download URL from configuration
        let download_url = self.config.get_download_url(tool_name, &version)?;

        println!(
            "📥 Downloading {} {} from {}",
            tool_name, version, download_url
        );

        // Use the installer to install the tool
        crate::installer::install_tool(tool_name, &version, &download_url)?;

        println!("✅ Successfully installed {} {}", tool_name, version);
        Ok(())
    }

    /// Check if a tool is available (registered)
    pub fn has_tool(&self, tool_name: &str) -> bool {
        self.registry.has_tool(tool_name)
    }

    /// Get information about a tool
    pub fn get_tool_info(&self, tool_name: &str) -> Result<ToolInfo> {
        self.registry.get_tool_info(tool_name)
    }

    /// Get information about all tools
    pub fn get_all_tools(&self) -> Vec<ToolInfo> {
        self.registry.get_all_tool_info()
    }

    /// Get list of available tool names
    pub fn get_tool_names(&self) -> Vec<String> {
        self.registry.tool_names()
    }

    /// Check tool installation status
    pub fn check_tool_status(&self, tool_name: &str) -> Result<ToolStatus> {
        let tool = self
            .registry
            .get(tool_name)
            .ok_or_else(|| anyhow!("Tool '{}' not found", tool_name))?;

        let installed = tool.is_installed()?;
        let version = if installed { tool.get_version()? } else { None };

        // Get configured version from Figment
        let configured_version = self
            .config
            .get_tool_config(tool_name)
            .and_then(|config| config.version.clone());

        Ok(ToolStatus {
            name: tool_name.to_string(),
            installed,
            current_version: version,
            configured_version,
            supports_auto_install: tool.supports_auto_install(),
        })
    }

    /// Get configuration manager (for advanced usage)
    pub fn config(&self) -> &FigmentConfigManager {
        &self.config
    }

    /// Reload configuration
    pub fn reload_config(&mut self) -> Result<()> {
        self.config.reload()
    }
}

/// Tool status information
#[derive(Debug, Clone)]
pub struct ToolStatus {
    pub name: String,
    pub installed: bool,
    pub current_version: Option<String>,
    pub configured_version: Option<String>,
    pub supports_auto_install: bool,
}

impl ToolStatus {
    /// Check if the tool needs to be installed or updated
    pub fn needs_action(&self) -> bool {
        if !self.installed {
            return self.supports_auto_install;
        }

        // Check if version mismatch (simplified)
        if let (Some(current), Some(configured)) = (&self.current_version, &self.configured_version)
        {
            return current != configured && configured != "latest";
        }

        false
    }

    /// Get a status summary string
    pub fn summary(&self) -> String {
        match (self.installed, &self.current_version) {
            (true, Some(version)) => format!("✅ {} ({})", self.name, version),
            (true, None) => format!("✅ {} (unknown version)", self.name),
            (false, _) => {
                if self.supports_auto_install {
                    format!("❌ {} (not installed, can auto-install)", self.name)
                } else {
                    format!("❌ {} (not installed)", self.name)
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_manager_creation() {
        // Should work with minimal config as fallback
        let manager = ToolManager::new().or_else(|_| ToolManager::minimal());
        assert!(manager.is_ok());
    }

    #[test]
    fn test_tool_availability() {
        let manager = ToolManager::minimal().unwrap();

        assert!(manager.has_tool("uv"));
        assert!(manager.has_tool("node"));
        assert!(!manager.has_tool("nonexistent"));
    }

    #[test]
    fn test_tool_info() {
        let manager = ToolManager::minimal().unwrap();

        let info = manager.get_tool_info("uv");
        assert!(info.is_ok());

        let info = info.unwrap();
        assert_eq!(info.name, "uv");
    }

    #[test]
    fn test_tool_status() {
        let manager = ToolManager::minimal().unwrap();

        let status = manager.check_tool_status("uv");
        assert!(status.is_ok());

        let status = status.unwrap();
        assert_eq!(status.name, "uv");
        assert!(status.supports_auto_install);
    }
}
