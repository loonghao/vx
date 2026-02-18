//! Tool version and configuration

#[cfg(feature = "schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Tool version specification
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
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
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
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

    /// Optional components to include (e.g., ["spectre", "mfc", "atl", "asan", "cli"])
    /// Used by MSVC provider to select additional msvc-kit components.
    /// See msvc-kit MsvcComponent for valid values.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub components: Option<Vec<String>>,

    /// Package ID patterns to exclude from installation (case-insensitive substring match)
    /// Used by MSVC provider for fine-grained control over package selection.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclude_patterns: Option<Vec<String>>,
}
