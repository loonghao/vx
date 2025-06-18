//! Main configuration manager implementation

use crate::{config::build_figment, detection::detect_project_info, types::*, Result};
use figment::Figment;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Main configuration manager using Figment for layered configuration
pub struct ConfigManager {
    figment: Figment,
    config: VxConfig,
    project_info: Option<ProjectInfo>,
}

impl ConfigManager {
    /// Create a new configuration manager with full layered configuration
    pub async fn new() -> Result<Self> {
        let project_info = detect_project_info()?;
        let figment = build_figment(&project_info)?;
        let config = figment.extract()?;

        Ok(Self {
            figment,
            config,
            project_info,
        })
    }

    /// Create a minimal configuration manager (builtin defaults only)
    pub fn minimal() -> Result<Self> {
        use figment::providers::Serialized;

        let figment = Figment::from(Serialized::defaults(VxConfig::default()));
        let config = figment.extract()?;

        Ok(Self {
            figment,
            config,
            project_info: None,
        })
    }

    /// Get tool configuration
    pub fn get_tool_config(&self, tool_name: &str) -> Option<&ToolConfig> {
        self.config.tools.get(tool_name)
    }

    /// Get tool version from configuration
    pub fn get_tool_version(&self, tool_name: &str) -> Option<String> {
        // First check if we have project info with tool versions
        if let Some(project_info) = &self.project_info {
            if let Some(version) = project_info.tool_versions.get(tool_name) {
                return Some(version.clone());
            }
        }

        // Then check tool-specific configuration
        if let Some(tool_config) = self.config.tools.get(tool_name) {
            return tool_config.version.clone();
        }

        None
    }

    /// Get list of available tools
    pub fn get_available_tools(&self) -> Vec<String> {
        let mut tools: Vec<String> = self.config.tools.keys().cloned().collect();

        // Add tools from project info
        if let Some(project_info) = &self.project_info {
            for tool in project_info.tool_versions.keys() {
                if !tools.contains(tool) {
                    tools.push(tool.clone());
                }
            }
        }

        // Add builtin tools if fallback is enabled
        if self.config.defaults.fallback_to_builtin {
            for builtin_tool in &["uv", "node", "go", "rust"] {
                if !tools.contains(&builtin_tool.to_string()) {
                    tools.push(builtin_tool.to_string());
                }
            }
        }

        tools.sort();
        tools
    }

    /// Check if a tool is supported
    pub fn supports_tool(&self, tool_name: &str) -> bool {
        // Check if configured
        if self.config.tools.contains_key(tool_name) {
            return true;
        }

        // Check builtin if fallback enabled
        if self.config.defaults.fallback_to_builtin {
            return ["uv", "node", "go", "rust"].contains(&tool_name);
        }

        false
    }

    /// Get the current configuration
    pub fn config(&self) -> &VxConfig {
        &self.config
    }

    /// Get project information
    pub fn project_info(&self) -> &Option<ProjectInfo> {
        &self.project_info
    }

    /// Get the underlying figment for advanced usage
    pub fn figment(&self) -> &Figment {
        &self.figment
    }

    /// Get configuration status for diagnostics
    pub fn get_status(&self) -> ConfigStatus {
        let layers = collect_layer_info();

        ConfigStatus {
            layers,
            available_tools: self.get_available_tools(),
            fallback_enabled: self.config.defaults.fallback_to_builtin,
            project_info: self.project_info.clone(),
        }
    }

    /// Initialize a new .vx.toml configuration file in the current directory
    pub async fn init_project_config(
        &self,
        tools: Option<HashMap<String, String>>,
        interactive: bool,
    ) -> Result<()> {
        let config_path = get_project_config_path()?;
        validate_config_not_exists(&config_path)?;

        let project_config = self.create_project_config(tools, interactive);
        let content = generate_config_content(&project_config)?;
        write_config_file(&config_path, &content)?;

        Ok(())
    }

    /// Create project configuration with tools and settings
    fn create_project_config(
        &self,
        tools: Option<HashMap<String, String>>,
        interactive: bool,
    ) -> ProjectConfig {
        let mut project_config = ProjectConfig::default();

        // Add provided tools or detect from project
        if let Some(tools) = tools {
            project_config.tools = tools;
        } else if interactive {
            // In interactive mode, we could prompt for tools
            // For now, just detect from existing project files
            if let Some(project_info) = &self.project_info {
                project_config.tools = project_info.tool_versions.clone();
            }
        }

        // Set sensible defaults
        project_config.settings.auto_install = true;
        project_config.settings.cache_duration = "7d".to_string();

        project_config
    }

    /// Validate the current configuration
    pub fn validate(&self) -> Result<Vec<String>> {
        let mut warnings = Vec::new();

        // Check for common configuration issues
        if self.config.tools.is_empty() && self.project_info.is_none() {
            warnings.push("No tools configured and no project detected".to_string());
        }

        // Check if auto_install is disabled but tools are missing
        if !self.config.defaults.auto_install {
            warnings
                .push("Auto-install is disabled - tools may need manual installation".to_string());
        }

        Ok(warnings)
    }

    /// Sync project configuration - install all required tools
    pub async fn sync_project(&self, _force: bool) -> Result<Vec<String>> {
        let mut installed_tools = Vec::new();

        // For now, just return what would be installed
        // TODO: Implement actual tool installation
        if let Some(project_info) = &self.project_info {
            for (tool_name, version) in &project_info.tool_versions {
                installed_tools.push(format!("{}@{}", tool_name, version));
            }
        }

        Ok(installed_tools)
    }

    /// Get download URL for a specific tool and version
    pub fn get_download_url(&self, tool_name: &str, version: &str) -> Result<String> {
        // Check if tool has custom sources configured
        if let Some(tool_config) = self.config.tools.get(tool_name) {
            if let Some(custom_sources) = &tool_config.custom_sources {
                if let Some(url) = custom_sources.first() {
                    return Ok(format!("{}/{}", url, version));
                }
            }
        }

        // Return error if no custom URL found - let caller handle fallback
        Err(crate::error::ConfigError::Other {
            message: format!("No download URL configured for tool: {}", tool_name),
        })
    }
}

/// Collect information about all configuration layers
fn collect_layer_info() -> Vec<LayerInfo> {
    let mut layers = Vec::new();

    layers.push(create_builtin_layer_info());

    if let Some(user_layer) = create_user_layer_info() {
        layers.push(user_layer);
    }

    layers.push(create_project_layer_info());
    layers.push(create_environment_layer_info());

    layers
}

/// Create layer info for builtin configuration
fn create_builtin_layer_info() -> LayerInfo {
    LayerInfo {
        name: "builtin".to_string(),
        available: true,
        priority: 10,
    }
}

/// Create layer info for user configuration
fn create_user_layer_info() -> Option<LayerInfo> {
    dirs::config_dir().map(|config_dir| {
        let global_config = config_dir.join("vx").join("config.toml");
        LayerInfo {
            name: "user".to_string(),
            available: global_config.exists(),
            priority: 50,
        }
    })
}

/// Create layer info for project configuration
fn create_project_layer_info() -> LayerInfo {
    let project_config = PathBuf::from(".vx.toml");
    LayerInfo {
        name: "project".to_string(),
        available: project_config.exists(),
        priority: 80,
    }
}

/// Create layer info for environment variables
fn create_environment_layer_info() -> LayerInfo {
    LayerInfo {
        name: "environment".to_string(),
        available: std::env::vars().any(|(k, _)| k.starts_with("VX_")),
        priority: 100,
    }
}

/// Get the path for the project configuration file
fn get_project_config_path() -> Result<PathBuf> {
    let config_path = std::env::current_dir()
        .map_err(|e| crate::error::ConfigError::Io {
            message: format!("Failed to get current directory: {}", e),
            source: e,
        })?
        .join(".vx.toml");
    Ok(config_path)
}

/// Validate that the configuration file doesn't already exist
fn validate_config_not_exists(config_path: &Path) -> Result<()> {
    if config_path.exists() {
        return Err(crate::error::ConfigError::Validation {
            message: "Configuration file .vx.toml already exists".to_string(),
        });
    }
    Ok(())
}

/// Generate the complete configuration file content
fn generate_config_content(project_config: &ProjectConfig) -> Result<String> {
    let toml_content = toml::to_string_pretty(project_config)?;

    let header = r#"# VX Project Configuration
# This file defines the tools and versions required for this project.
# Run 'vx sync' to install all required tools.

"#;

    Ok(format!("{}{}", header, toml_content))
}

/// Write configuration content to file
fn write_config_file(config_path: &PathBuf, content: &str) -> Result<()> {
    std::fs::write(config_path, content).map_err(|e| crate::error::ConfigError::Io {
        message: format!("Failed to write .vx.toml: {}", e),
        source: e,
    })
}
