//! Tool version and configuration

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Tool version specification
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum ToolVersion {
    /// Simple version string
    Simple(String),
    /// Detailed tool configuration
    Detailed(ToolConfig),
}

impl Default for ToolVersion {
    fn default() -> Self {
        ToolVersion::Simple("latest".to_string())
    }
}

/// Detailed tool configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct ToolConfig {
    /// Version string
    pub version: String,

    /// Post-install command
    #[serde(skip_serializing_if = "Option::is_none")]
    pub postinstall: Option<String>,

    /// OS restrictions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub os: Option<Vec<String>>,

    /// Install environment variables
    #[serde(skip_serializing_if = "Option::is_none")]
    pub install_env: Option<HashMap<String, String>>,
}
