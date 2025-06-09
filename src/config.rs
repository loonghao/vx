use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Config {
    pub tools: HashMap<String, ToolConfig>,
    pub defaults: DefaultConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ToolConfig {
    pub version: Option<String>,
    pub install_method: Option<String>,
    pub proxy_command: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DefaultConfig {
    pub auto_install: bool,
    pub check_updates: bool,
    pub update_interval: String,
}

impl Default for DefaultConfig {
    fn default() -> Self {
        Self {
            auto_install: true,
            check_updates: true,
            update_interval: "24h".to_string(),
        }
    }
}

impl Config {
    /// Load configuration from file
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;
        
        if !config_path.exists() {
            return Ok(Self::default());
        }
        
        let content = fs::read_to_string(&config_path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }
    
    /// Save configuration to file
    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path()?;
        
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        let content = toml::to_string_pretty(self)?;
        fs::write(&config_path, content)?;
        Ok(())
    }
    
    /// Load project-specific configuration
    pub fn load_project() -> Result<Option<Self>> {
        let project_config = PathBuf::from(".vx.toml");
        
        if !project_config.exists() {
            return Ok(None);
        }
        
        let content = fs::read_to_string(&project_config)?;
        let config: Config = toml::from_str(&content)?;
        Ok(Some(config))
    }
    
    /// Get configuration file path
    fn config_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?;
        Ok(config_dir.join("vx").join("config.toml"))
    }
    
    /// Get tool configuration
    pub fn get_tool(&self, tool: &str) -> Option<&ToolConfig> {
        self.tools.get(tool)
    }
    
    /// Set tool configuration
    pub fn set_tool(&mut self, tool: String, config: ToolConfig) {
        self.tools.insert(tool, config);
    }
}
