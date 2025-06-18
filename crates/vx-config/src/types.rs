//! Core types for vx configuration management

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Main vx configuration structure
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct VxConfig {
    /// Global default settings
    pub defaults: DefaultConfig,
    /// Tool-specific configurations
    pub tools: HashMap<String, ToolConfig>,
    /// Registry configurations
    pub registries: HashMap<String, RegistryConfig>,
}

/// Default configuration settings
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DefaultConfig {
    /// Whether to automatically install missing tools
    pub auto_install: bool,
    /// Default cache duration for downloads
    pub cache_duration: String,
    /// Whether to fall back to builtin tool configurations
    pub fallback_to_builtin: bool,
    /// Default installation directory
    pub install_dir: Option<String>,
    /// Whether to use system PATH for tools
    pub use_system_path: bool,
}

impl Default for DefaultConfig {
    fn default() -> Self {
        Self {
            auto_install: true,
            cache_duration: "7d".to_string(),
            fallback_to_builtin: true,
            install_dir: None,
            use_system_path: false,
        }
    }
}

/// Configuration for a specific tool
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ToolConfig {
    /// Specific version to use
    pub version: Option<String>,
    /// Installation method (auto, manual, system)
    pub install_method: Option<String>,
    /// Registry to use for this tool
    pub registry: Option<String>,
    /// Custom download sources
    pub custom_sources: Option<Vec<String>>,
}

/// Registry configuration
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RegistryConfig {
    /// Registry URL
    pub url: String,
    /// Authentication token if required
    pub token: Option<String>,
    /// Whether this registry is trusted
    pub trusted: bool,
}

/// Supported project types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ProjectType {
    Python,  // pyproject.toml
    Rust,    // Cargo.toml
    Node,    // package.json
    Go,      // go.mod
    Mixed,   // Multiple project types
    Unknown, // No recognized project files
}

/// Information about a detected project
#[derive(Debug, Clone)]
pub struct ProjectInfo {
    /// Type of project detected
    pub project_type: ProjectType,
    /// Path to the primary configuration file
    pub config_file: PathBuf,
    /// Tool versions detected from project files
    pub tool_versions: HashMap<String, String>,
}

/// Configuration status for diagnostics
#[derive(Debug, Clone)]
pub struct ConfigStatus {
    /// Information about configuration layers
    pub layers: Vec<LayerInfo>,
    /// List of available tools
    pub available_tools: Vec<String>,
    /// Whether fallback to builtin is enabled
    pub fallback_enabled: bool,
    /// Project information if detected
    pub project_info: Option<ProjectInfo>,
}

/// Information about a configuration layer
#[derive(Debug, Clone)]
pub struct LayerInfo {
    /// Name of the layer (builtin, user, project, environment)
    pub name: String,
    /// Whether this layer is available/active
    pub available: bool,
    /// Priority of this layer (higher = more important)
    pub priority: i32,
}

impl ConfigStatus {
    /// Get a summary of the configuration status
    pub fn summary(&self) -> String {
        let active_layers: Vec<&str> = self
            .layers
            .iter()
            .filter(|l| l.available)
            .map(|l| l.name.as_str())
            .collect();

        format!(
            "Configuration layers: {} | Tools: {} | Fallback: {}",
            active_layers.join(", "),
            self.available_tools.len(),
            if self.fallback_enabled {
                "enabled"
            } else {
                "disabled"
            }
        )
    }

    /// Check if the configuration is healthy
    pub fn is_healthy(&self) -> bool {
        // At least one layer should be available
        self.layers.iter().any(|l| l.available) && !self.available_tools.is_empty()
    }
}

/// Project configuration for .vx.toml files
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct ProjectConfig {
    /// Tool versions required for this project
    pub tools: HashMap<String, String>,
    /// Project-specific settings
    pub settings: ProjectSettings,
}

/// Settings specific to a project
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectSettings {
    /// Whether to auto-install missing tools
    pub auto_install: bool,
    /// Cache duration for this project
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
