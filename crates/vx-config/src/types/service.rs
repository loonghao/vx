//! Service configuration

#[cfg(feature = "schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Service configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(default)]
pub struct ServiceConfig {
    /// Docker image
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,

    /// Command to run (for non-container services)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,

    /// Port mappings
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ports: Vec<String>,

    /// Environment variables
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub env: HashMap<String, String>,

    /// Environment file
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env_file: Option<String>,

    /// Volume mounts
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub volumes: Vec<String>,

    /// Dependencies
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub depends_on: Vec<String>,

    /// Health check command
    #[serde(skip_serializing_if = "Option::is_none")]
    pub healthcheck: Option<String>,

    /// Working directory
    #[serde(skip_serializing_if = "Option::is_none")]
    pub working_dir: Option<String>,
}
