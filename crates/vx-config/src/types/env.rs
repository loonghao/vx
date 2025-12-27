//! Environment variable configuration

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Environment variable configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct EnvConfig {
    /// Static environment variables
    #[serde(flatten)]
    pub vars: HashMap<String, String>,

    /// Required environment variables
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<HashMap<String, String>>,

    /// Optional environment variables
    #[serde(skip_serializing_if = "Option::is_none")]
    pub optional: Option<HashMap<String, String>>,

    /// Secret variables (loaded from secure storage)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secrets: Option<SecretsConfig>,
}

/// Secrets configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct SecretsConfig {
    /// Provider (auto, 1password, vault, aws-secrets)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,

    /// Secret items to load
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub items: Vec<String>,
}
