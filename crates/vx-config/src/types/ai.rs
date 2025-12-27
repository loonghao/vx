//! AI integration configuration

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// AI configuration (Phase 2)
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct AiConfig {
    /// Enable AI integration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,

    /// AI provider
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
}
