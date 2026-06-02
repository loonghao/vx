//! AI integration configuration

#[cfg(feature = "schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// AI configuration (Phase 2)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(default)]
pub struct AiConfig {
    /// Enable AI integration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,

    /// AI provider
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,

    /// Hash of the built-in vx skills last installed for this project
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skills_hash: Option<String>,

    /// Timestamp when built-in vx skills were last installed for this project
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skills_updated_at: Option<String>,
}
