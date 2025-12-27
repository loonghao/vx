//! Settings configuration

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Settings configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct SettingsConfig {
    /// Auto-install missing tools
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_install: Option<bool>,

    /// Parallel installation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parallel_install: Option<bool>,

    /// Cache duration (e.g., "7d")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_duration: Option<String>,

    /// Shell to use (auto, bash, zsh, fish, pwsh)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shell: Option<String>,

    /// Log level
    #[serde(skip_serializing_if = "Option::is_none")]
    pub log_level: Option<String>,

    /// Experimental features
    #[serde(skip_serializing_if = "Option::is_none")]
    pub experimental: Option<ExperimentalConfig>,
}

/// Experimental features
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct ExperimentalConfig {
    /// Monorepo support
    #[serde(skip_serializing_if = "Option::is_none")]
    pub monorepo: Option<bool>,

    /// Workspaces support
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspaces: Option<bool>,
}
