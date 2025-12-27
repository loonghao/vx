//! Lifecycle hooks configuration

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Lifecycle hooks configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct HooksConfig {
    /// Pre-setup hook
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pre_setup: Option<HookCommand>,

    /// Post-setup hook
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post_setup: Option<HookCommand>,

    /// Pre-commit hook
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pre_commit: Option<HookCommand>,

    /// Directory enter hook
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enter: Option<HookCommand>,

    /// Custom hooks
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub custom: HashMap<String, HookCommand>,
}

/// Hook command (string or array)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum HookCommand {
    /// Single command
    Single(String),
    /// Multiple commands
    Multiple(Vec<String>),
}

impl Default for HookCommand {
    fn default() -> Self {
        HookCommand::Single(String::new())
    }
}
