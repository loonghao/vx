//! Versioning strategy configuration

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Versioning configuration (Phase 5)
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct VersioningConfig {
    /// Versioning strategy (semver, calver)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strategy: Option<String>,

    /// Auto-bump version
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_bump: Option<bool>,
}
