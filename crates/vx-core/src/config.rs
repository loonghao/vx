//! Configuration management traits and types

use crate::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Trait for managing tool configuration
pub trait ConfigManager: Send + Sync {
    /// Get configuration for a specific tool
    fn get_tool_config(&self, tool_name: &str) -> Result<Option<ToolConfig>>;
    
    /// Set configuration for a specific tool
    fn set_tool_config(&mut self, tool_name: &str, config: ToolConfig) -> Result<()>;
    
    /// Get global configuration
    fn get_global_config(&self) -> Result<GlobalConfig>;
    
    /// Set global configuration
    fn set_global_config(&mut self, config: GlobalConfig) -> Result<()>;
    
    /// Save configuration to disk
    fn save(&self) -> Result<()>;
    
    /// Load configuration from disk
    fn load(&mut self) -> Result<()>;
    
    /// Get configuration file path
    fn config_path(&self) -> PathBuf;
}

/// Configuration for a specific tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolConfig {
    /// Tool name
    pub name: String,
    
    /// Default version to use
    pub default_version: Option<String>,
    
    /// Download URLs for different versions
    pub download_urls: std::collections::HashMap<String, String>,
    
    /// Installation directory
    pub install_dir: Option<PathBuf>,
    
    /// Environment variables to set when using this tool
    pub environment: std::collections::HashMap<String, String>,
    
    /// Tool-specific settings
    pub settings: std::collections::HashMap<String, serde_json::Value>,
    
    /// Whether auto-installation is enabled
    pub auto_install: bool,
    
    /// Whether to check for updates automatically
    pub auto_update: bool,
}

/// Global vx configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalConfig {
    /// Default installation directory
    pub install_dir: PathBuf,
    
    /// Cache directory
    pub cache_dir: PathBuf,
    
    /// Configuration directory
    pub config_dir: PathBuf,
    
    /// Whether to use system PATH as fallback
    pub use_system_path: bool,
    
    /// Default isolation level
    pub isolation_level: crate::package_manager::IsolationLevel,
    
    /// Proxy settings
    pub proxy: Option<ProxyConfig>,
    
    /// Update check settings
    pub update_check: UpdateCheckConfig,
    
    /// Telemetry settings
    pub telemetry: TelemetryConfig,
    
    /// Plugin settings
    pub plugins: PluginConfig,
}

/// Proxy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyConfig {
    pub http_proxy: Option<String>,
    pub https_proxy: Option<String>,
    pub no_proxy: Vec<String>,
}

/// Update check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCheckConfig {
    pub enabled: bool,
    pub frequency: UpdateFrequency,
    pub include_prerelease: bool,
}

/// Update check frequency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UpdateFrequency {
    Never,
    Daily,
    Weekly,
    Monthly,
}

/// Telemetry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryConfig {
    pub enabled: bool,
    pub anonymous: bool,
}

/// Plugin configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    pub enabled_plugins: Vec<String>,
    pub plugin_dirs: Vec<PathBuf>,
    pub auto_discover: bool,
}

impl Default for ToolConfig {
    fn default() -> Self {
        Self {
            name: String::new(),
            default_version: None,
            download_urls: std::collections::HashMap::new(),
            install_dir: None,
            environment: std::collections::HashMap::new(),
            settings: std::collections::HashMap::new(),
            auto_install: true,
            auto_update: false,
        }
    }
}

impl Default for GlobalConfig {
    fn default() -> Self {
        let home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let vx_dir = home_dir.join(".vx");
        
        Self {
            install_dir: vx_dir.join("tools"),
            cache_dir: vx_dir.join("cache"),
            config_dir: vx_dir.join("config"),
            use_system_path: false,
            isolation_level: crate::package_manager::IsolationLevel::Project,
            proxy: None,
            update_check: UpdateCheckConfig {
                enabled: true,
                frequency: UpdateFrequency::Weekly,
                include_prerelease: false,
            },
            telemetry: TelemetryConfig {
                enabled: false,
                anonymous: true,
            },
            plugins: PluginConfig {
                enabled_plugins: Vec::new(),
                plugin_dirs: vec![vx_dir.join("plugins")],
                auto_discover: true,
            },
        }
    }
}
