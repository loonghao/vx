//! Security scanning configuration

#[cfg(feature = "schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Security configuration (Phase 4)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(default)]
pub struct SecurityConfig {
    /// Enable security scanning
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,

    /// Fail on severity level (critical, high, medium, low)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fail_on: Option<String>,

    /// Dependency vulnerability scanning
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audit: Option<SecurityAuditConfig>,

    /// Secret detection
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secrets: Option<SecretDetectionConfig>,

    /// SAST (Static Application Security Testing)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sast: Option<SastConfig>,

    /// Allowed licenses
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub allowed_licenses: Vec<String>,

    /// Denied licenses
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub denied_licenses: Vec<String>,
}

/// Security audit configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(default)]
pub struct SecurityAuditConfig {
    /// Enable dependency audit
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,

    /// Ignore specific CVEs
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ignore: Vec<String>,

    /// Audit on install
    #[serde(skip_serializing_if = "Option::is_none")]
    pub on_install: Option<bool>,

    /// Audit on CI
    #[serde(skip_serializing_if = "Option::is_none")]
    pub on_ci: Option<bool>,
}

/// Secret detection configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(default)]
pub struct SecretDetectionConfig {
    /// Enable secret detection
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,

    /// Patterns to detect
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub patterns: Vec<String>,

    /// Files to exclude
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub exclude: Vec<String>,

    /// Pre-commit hook
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pre_commit: Option<bool>,

    /// Baseline file (known secrets to ignore)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub baseline: Option<String>,
}

/// SAST configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(default)]
pub struct SastConfig {
    /// Enable SAST
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,

    /// SAST tool (semgrep, codeql, snyk)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool: Option<String>,

    /// Ruleset to use
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ruleset: Option<String>,

    /// Custom rules path
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rules_path: Option<String>,

    /// Paths to exclude
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub exclude: Vec<String>,
}
