//! Team collaboration configuration

#[cfg(feature = "schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Team configuration (Phase 3)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(default)]
pub struct TeamConfig {
    /// Extends from URL (remote preset)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extends: Option<String>,

    /// Code owners configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code_owners: Option<CodeOwnersConfig>,

    /// Review rules
    #[serde(skip_serializing_if = "Option::is_none")]
    pub review: Option<ReviewConfig>,

    /// Conventions to enforce
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conventions: Option<ConventionsConfig>,
}

/// Code owners configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(default)]
pub struct CodeOwnersConfig {
    /// Enable CODEOWNERS generation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,

    /// Default owners for all files
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub default_owners: Vec<String>,

    /// Path-specific owners
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub paths: HashMap<String, Vec<String>>,

    /// Output file path (default: .github/CODEOWNERS)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<String>,
}

/// Review configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(default)]
pub struct ReviewConfig {
    /// Minimum required approvals
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required_approvals: Option<u32>,

    /// Require review from code owners
    #[serde(skip_serializing_if = "Option::is_none")]
    pub require_code_owner: Option<bool>,

    /// Dismiss stale reviews on new commits
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dismiss_stale: Option<bool>,

    /// Protected branches
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub protected_branches: Vec<String>,

    /// Auto-assign reviewers
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_assign: Option<AutoAssignConfig>,
}

/// Auto-assign reviewers configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(default)]
pub struct AutoAssignConfig {
    /// Enable auto-assign
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,

    /// Number of reviewers to assign
    #[serde(skip_serializing_if = "Option::is_none")]
    pub count: Option<u32>,

    /// Reviewer pool
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reviewers: Vec<String>,
}

/// Conventions configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(default)]
pub struct ConventionsConfig {
    /// Commit message format (conventional, angular, custom)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commit_format: Option<String>,

    /// Custom commit pattern (regex)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commit_pattern: Option<String>,

    /// Branch naming convention
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch_pattern: Option<String>,

    /// PR title format
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pr_title_pattern: Option<String>,

    /// Enforce linear history
    #[serde(skip_serializing_if = "Option::is_none")]
    pub linear_history: Option<bool>,

    /// Allowed merge strategies (merge, squash, rebase)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub merge_strategies: Vec<String>,
}
