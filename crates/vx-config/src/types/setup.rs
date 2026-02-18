//! Setup pipeline configuration
//!
//! This module defines the setup pipeline configuration for `vx setup`.
//! The setup process is a pipeline of hooks that can be customized.
//!
//! # Built-in Hooks
//!
//! - `install_tools` - Install all configured tools
//! - `export_paths` - Export tool paths for CI environments
//!
//! # CI Environment Detection
//!
//! The `export_paths` hook automatically detects CI environments:
//! - GitHub Actions: Uses `GITHUB_PATH`
//! - GitLab CI: Uses `GITLAB_CI` environment
//! - Generic CI: Detects via `CI=true`

#[cfg(feature = "schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Setup pipeline configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(default)]
pub struct SetupConfig {
    /// Pipeline hooks to execute (in order)
    /// Default: ["install_tools", "export_paths"]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pipeline: Option<Vec<String>>,

    /// Hook configurations
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hooks: Option<SetupHooksConfig>,

    /// CI-specific configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ci: Option<CiConfig>,
}

/// Setup hooks configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(default)]
pub struct SetupHooksConfig {
    /// Install tools hook configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub install_tools: Option<InstallToolsHook>,

    /// Export paths hook configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub export_paths: Option<ExportPathsHook>,

    /// Custom hooks (key = hook name, value = command)
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub custom: HashMap<String, SetupHookCommand>,
}

/// Install tools hook configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(default)]
pub struct InstallToolsHook {
    /// Whether this hook is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Install tools in parallel
    #[serde(default = "default_true")]
    pub parallel: bool,

    /// Force reinstall even if already installed
    #[serde(default)]
    pub force: bool,
}

/// Export paths hook configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(default)]
pub struct ExportPathsHook {
    /// Whether this hook is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Only run in CI environments
    #[serde(default = "default_true")]
    pub ci_only: bool,

    /// Additional paths to export
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub extra_paths: Vec<String>,
}

/// CI environment configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(default)]
pub struct CiConfig {
    /// Enable CI mode (auto-detected if not specified)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,

    /// CI provider (auto-detected if not specified)
    /// Supported: "github", "gitlab", "azure", "circleci", "jenkins", "generic"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,

    /// Custom environment variable for PATH export
    /// (overrides auto-detection)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path_env_file: Option<String>,

    /// Custom environment variable for environment export
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env_file: Option<String>,
}

/// Setup hook command configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(untagged)]
pub enum SetupHookCommand {
    /// Simple command string
    Simple(String),
    /// Multiple commands
    Multiple(Vec<String>),
    /// Detailed configuration
    Detailed(SetupHookDetail),
}

/// Detailed setup hook configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(default)]
pub struct SetupHookDetail {
    /// Command(s) to execute
    pub command: SetupHookCommandType,

    /// Whether this hook is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Only run in CI environments
    #[serde(default)]
    pub ci_only: bool,

    /// Only run in non-CI environments
    #[serde(default)]
    pub local_only: bool,

    /// Continue on failure
    #[serde(default)]
    pub continue_on_failure: bool,

    /// Working directory for the command
    #[serde(skip_serializing_if = "Option::is_none")]
    pub working_dir: Option<String>,

    /// Environment variables for the command
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub env: HashMap<String, String>,
}

/// Command type for setup hooks
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(untagged)]
pub enum SetupHookCommandType {
    /// Single command
    Single(String),
    /// Multiple commands
    Multiple(Vec<String>),
}

impl Default for SetupHookCommandType {
    fn default() -> Self {
        SetupHookCommandType::Single(String::new())
    }
}

fn default_true() -> bool {
    true
}

/// CI provider detection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CiProvider {
    /// GitHub Actions
    GitHub,
    /// GitLab CI
    GitLab,
    /// Azure Pipelines
    Azure,
    /// CircleCI
    CircleCI,
    /// Jenkins
    Jenkins,
    /// Generic CI (CI=true)
    Generic,
    /// Not in CI
    None,
}

impl CiProvider {
    /// Detect CI provider from environment variables
    pub fn detect() -> Self {
        if std::env::var("GITHUB_ACTIONS").is_ok() {
            return CiProvider::GitHub;
        }
        if std::env::var("GITLAB_CI").is_ok() {
            return CiProvider::GitLab;
        }
        if std::env::var("TF_BUILD").is_ok() || std::env::var("AZURE_PIPELINES").is_ok() {
            return CiProvider::Azure;
        }
        if std::env::var("CIRCLECI").is_ok() {
            return CiProvider::CircleCI;
        }
        if std::env::var("JENKINS_URL").is_ok() {
            return CiProvider::Jenkins;
        }
        if std::env::var("CI").map(|v| v == "true").unwrap_or(false) {
            return CiProvider::Generic;
        }
        CiProvider::None
    }

    /// Check if running in any CI environment
    pub fn is_ci(&self) -> bool {
        !matches!(self, CiProvider::None)
    }

    /// Get the PATH export file for this CI provider
    pub fn path_export_file(&self) -> Option<String> {
        match self {
            CiProvider::GitHub => std::env::var("GITHUB_PATH").ok(),
            CiProvider::GitLab => None, // GitLab uses different mechanism
            CiProvider::Azure => std::env::var("GITHUB_PATH").ok(), // Azure also supports GITHUB_PATH
            CiProvider::CircleCI => Some("$BASH_ENV".to_string()),
            CiProvider::Jenkins => None,
            CiProvider::Generic => None,
            CiProvider::None => None,
        }
    }

    /// Get the environment export file for this CI provider
    pub fn env_export_file(&self) -> Option<String> {
        match self {
            CiProvider::GitHub => std::env::var("GITHUB_ENV").ok(),
            CiProvider::GitLab => None,
            CiProvider::Azure => std::env::var("GITHUB_ENV").ok(),
            CiProvider::CircleCI => Some("$BASH_ENV".to_string()),
            CiProvider::Jenkins => None,
            CiProvider::Generic => None,
            CiProvider::None => None,
        }
    }

    /// Get display name
    pub fn name(&self) -> &'static str {
        match self {
            CiProvider::GitHub => "GitHub Actions",
            CiProvider::GitLab => "GitLab CI",
            CiProvider::Azure => "Azure Pipelines",
            CiProvider::CircleCI => "CircleCI",
            CiProvider::Jenkins => "Jenkins",
            CiProvider::Generic => "Generic CI",
            CiProvider::None => "Local",
        }
    }
}

impl std::fmt::Display for CiProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl SetupConfig {
    /// Get the default pipeline
    pub fn default_pipeline() -> Vec<String> {
        vec![
            "pre_setup".to_string(),
            "install_tools".to_string(),
            "export_paths".to_string(),
            "post_setup".to_string(),
        ]
    }

    /// Get the pipeline to execute
    pub fn get_pipeline(&self) -> Vec<String> {
        self.pipeline.clone().unwrap_or_else(Self::default_pipeline)
    }

    /// Check if a hook is enabled
    pub fn is_hook_enabled(&self, hook_name: &str) -> bool {
        match hook_name {
            "install_tools" => self
                .hooks
                .as_ref()
                .and_then(|h| h.install_tools.as_ref())
                .map(|h| h.enabled)
                .unwrap_or(true),
            "export_paths" => self
                .hooks
                .as_ref()
                .and_then(|h| h.export_paths.as_ref())
                .map(|h| h.enabled)
                .unwrap_or(true),
            _ => true,
        }
    }
}
