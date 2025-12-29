//! Common types for vx-setup

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Hook command configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum HookCommand {
    /// Single command string
    Single(String),
    /// Multiple commands
    Multiple(Vec<String>),
}

impl HookCommand {
    /// Get commands as a vector
    pub fn as_vec(&self) -> Vec<String> {
        match self {
            HookCommand::Single(cmd) => vec![cmd.clone()],
            HookCommand::Multiple(cmds) => cmds.clone(),
        }
    }
}

/// Setup pipeline configuration
///
/// This is a simplified configuration struct that can be constructed
/// from vx-config's SetupConfig and HooksConfig.
#[derive(Debug, Clone, Default)]
pub struct SetupPipelineConfig {
    /// Pipeline hooks to execute (in order)
    /// Default: ["pre_setup", "install_tools", "export_paths", "post_setup"]
    pub pipeline: Vec<String>,

    /// Tool versions from config
    pub tools: HashMap<String, String>,

    /// Pre-setup hook command
    pub pre_setup: Option<HookCommand>,

    /// Post-setup hook command
    pub post_setup: Option<HookCommand>,

    /// Custom hooks (key = hook name, value = command)
    pub custom_hooks: HashMap<String, CustomHookConfig>,

    /// CI-specific configuration
    pub ci: Option<CiConfig>,

    /// Export paths configuration
    pub export_paths: Option<ExportPathsConfig>,

    /// Install tools configuration
    pub install_tools: Option<InstallToolsConfig>,
}

impl SetupPipelineConfig {
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
        if self.pipeline.is_empty() {
            Self::default_pipeline()
        } else {
            self.pipeline.clone()
        }
    }
}

/// Custom hook configuration
#[derive(Debug, Clone)]
pub struct CustomHookConfig {
    /// Command(s) to execute
    pub command: HookCommand,

    /// Whether this hook is enabled
    pub enabled: bool,

    /// Only run in CI environments
    pub ci_only: bool,

    /// Only run in non-CI environments
    pub local_only: bool,

    /// Continue on failure
    pub continue_on_failure: bool,

    /// Working directory for the command
    pub working_dir: Option<String>,

    /// Environment variables for the command
    pub env: HashMap<String, String>,
}

impl Default for CustomHookConfig {
    fn default() -> Self {
        Self {
            command: HookCommand::Single(String::new()),
            enabled: true,
            ci_only: false,
            local_only: false,
            continue_on_failure: false,
            working_dir: None,
            env: HashMap::new(),
        }
    }
}

/// CI environment configuration
#[derive(Debug, Clone, Default)]
pub struct CiConfig {
    /// Enable CI mode (auto-detected if not specified)
    pub enabled: Option<bool>,

    /// CI provider (auto-detected if not specified)
    pub provider: Option<String>,

    /// Custom environment variable for PATH export
    pub path_env_file: Option<String>,

    /// Custom environment variable for environment export
    pub env_file: Option<String>,
}

/// Export paths hook configuration
#[derive(Debug, Clone)]
pub struct ExportPathsConfig {
    /// Whether this hook is enabled
    pub enabled: bool,

    /// Only run in CI environments
    pub ci_only: bool,

    /// Additional paths to export
    pub extra_paths: Vec<String>,
}

impl Default for ExportPathsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            ci_only: true,
            extra_paths: Vec::new(),
        }
    }
}

/// Install tools hook configuration
#[derive(Debug, Clone)]
pub struct InstallToolsConfig {
    /// Whether this hook is enabled
    pub enabled: bool,

    /// Install tools in parallel
    pub parallel: bool,

    /// Force reinstall even if already installed
    pub force: bool,
}

impl Default for InstallToolsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            parallel: true,
            force: false,
        }
    }
}
