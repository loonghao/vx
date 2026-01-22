//! Session context for shell environments
//!
//! This module provides a unified session context that can be used by both
//! `vx dev` (from vx.toml) and `vx env shell` (from environment directories).

use std::collections::HashMap;
use std::path::PathBuf;

/// Source of the session configuration
#[derive(Debug, Clone)]
pub enum SessionSource {
    /// Configuration from vx.toml
    VxToml(PathBuf),
    /// Configuration from an environment directory
    EnvDir { path: PathBuf, name: String },
    /// Inline configuration (from command line)
    Inline,
}

impl SessionSource {
    /// Get a display name for the source
    pub fn display_name(&self) -> String {
        match self {
            SessionSource::VxToml(path) => {
                format!("vx.toml ({})", path.display())
            }
            SessionSource::EnvDir { name, .. } => {
                format!("env:{}", name)
            }
            SessionSource::Inline => "inline".to_string(),
        }
    }
}

/// Isolation configuration for the environment
#[derive(Debug, Clone)]
pub struct IsolationConfig {
    /// Whether isolation mode is enabled
    pub enabled: bool,
    /// Patterns for environment variables to pass through
    pub passenv: Vec<String>,
}

impl Default for IsolationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            passenv: Vec::new(),
        }
    }
}

/// Unified session context for shell environments
///
/// This struct holds all the configuration needed to start a shell session,
/// whether it comes from vx.toml or an environment directory.
#[derive(Debug, Clone)]
pub struct SessionContext {
    /// Project or environment name
    pub name: String,
    /// Project root directory (if applicable)
    pub project_root: Option<PathBuf>,
    /// Tool name to version mapping
    pub tools: HashMap<String, String>,
    /// Custom environment variables to set
    pub env_vars: HashMap<String, String>,
    /// Isolation configuration
    pub isolation: IsolationConfig,
    /// Source of the configuration
    pub source: SessionSource,
}

impl SessionContext {
    /// Create a new session context
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            project_root: None,
            tools: HashMap::new(),
            env_vars: HashMap::new(),
            isolation: IsolationConfig::default(),
            source: SessionSource::Inline,
        }
    }

    /// Set the project root
    pub fn project_root(mut self, root: PathBuf) -> Self {
        self.project_root = Some(root);
        self
    }

    /// Set tools
    pub fn tools(mut self, tools: &HashMap<String, String>) -> Self {
        self.tools = tools.clone();
        self
    }

    /// Add a tool
    pub fn tool(mut self, name: impl Into<String>, version: impl Into<String>) -> Self {
        self.tools.insert(name.into(), version.into());
        self
    }

    /// Set custom environment variables
    pub fn env_vars(mut self, env_vars: &HashMap<String, String>) -> Self {
        self.env_vars.extend(env_vars.clone());
        self
    }

    /// Add an environment variable
    pub fn env_var(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env_vars.insert(key.into(), value.into());
        self
    }

    /// Set isolation configuration
    pub fn isolation(mut self, config: IsolationConfig) -> Self {
        self.isolation = config;
        self
    }

    /// Enable or disable isolation
    pub fn isolated(mut self, enabled: bool) -> Self {
        self.isolation.enabled = enabled;
        self
    }

    /// Set passenv patterns
    pub fn passenv(mut self, patterns: Vec<String>) -> Self {
        self.isolation.passenv = patterns;
        self
    }

    /// Set the source of the configuration
    pub fn source(mut self, source: SessionSource) -> Self {
        self.source = source;
        self
    }

    /// Get tools as a formatted string for display
    pub fn tools_display(&self) -> String {
        if self.tools.is_empty() {
            "(none)".to_string()
        } else {
            self.tools
                .iter()
                .map(|(k, v)| format!("{}@{}", k, v))
                .collect::<Vec<_>>()
                .join(", ")
        }
    }
}
