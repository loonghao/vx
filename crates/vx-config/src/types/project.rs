//! Project metadata configuration

#[cfg(feature = "schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Project metadata
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(default)]
pub struct ProjectConfig {
    /// Project name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Project description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Project version
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,

    /// License
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,

    /// Repository URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repository: Option<String>,
}
