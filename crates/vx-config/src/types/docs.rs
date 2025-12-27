//! Documentation generation configuration

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Documentation configuration (Phase 2)
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct DocsConfig {
    /// Enable documentation generation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,

    /// Output directory
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<String>,
}
