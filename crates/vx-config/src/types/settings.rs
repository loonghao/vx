//! Settings configuration

#[cfg(feature = "schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Settings configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(default)]
pub struct SettingsConfig {
    /// Auto-install missing tools
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_install: Option<bool>,

    /// Parallel installation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parallel_install: Option<bool>,

    /// Cache duration (e.g., "7d")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_duration: Option<String>,

    /// Shell to use (auto, bash, zsh, fish, pwsh)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shell: Option<String>,

    /// Log level
    #[serde(skip_serializing_if = "Option::is_none")]
    pub log_level: Option<String>,

    /// Isolation mode for dev environment
    ///
    /// When `true` (default), `vx dev` creates an isolated environment where only
    /// vx-managed tools are available in PATH. System tools are NOT inherited.
    ///
    /// When `false`, the system PATH is inherited, allowing access to both
    /// vx-managed and system-installed tools (vx tools take priority).
    ///
    /// Can be overridden with `vx dev --inherit-system`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub isolation: Option<bool>,

    /// Environment variables to pass through to the dev environment (like tox's passenv)
    ///
    /// When in isolation mode, only these environment variables from the host
    /// system will be available in the dev environment. Supports glob patterns.
    ///
    /// Example:
    /// ```toml
    /// [settings]
    /// passenv = ["HOME", "USER", "SSH_*", "GITHUB_*", "CI"]
    /// ```
    ///
    /// By default, essential system variables are always passed:
    /// - Windows: SYSTEMROOT, TEMP, TMP, USERPROFILE, APPDATA, LOCALAPPDATA, HOMEDRIVE, HOMEPATH
    /// - Unix: HOME, USER, SHELL, TERM, LANG, LC_*
    #[serde(skip_serializing_if = "Option::is_none")]
    pub passenv: Option<Vec<String>>,

    /// Environment variables to explicitly set in the dev environment
    ///
    /// These override any passed-through variables with the same name.
    ///
    /// Example:
    /// ```toml
    /// [settings]
    /// setenv = { NODE_ENV = "development", DEBUG = "1" }
    /// ```
    #[serde(skip_serializing_if = "Option::is_none")]
    pub setenv: Option<std::collections::HashMap<String, String>>,

    /// Experimental features
    #[serde(skip_serializing_if = "Option::is_none")]
    pub experimental: Option<ExperimentalConfig>,
}

/// Experimental features
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(default)]
pub struct ExperimentalConfig {
    /// Monorepo support
    #[serde(skip_serializing_if = "Option::is_none")]
    pub monorepo: Option<bool>,

    /// Workspaces support
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspaces: Option<bool>,
}
