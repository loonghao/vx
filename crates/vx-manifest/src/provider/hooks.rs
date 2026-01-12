use serde::{Deserialize, Serialize};

use super::defaults::{default_hook_timeout, default_true};

/// Hooks configuration
///
/// Provides complete lifecycle hooks for runtime operations.
/// All hooks are optional and default to empty vectors.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct HooksDef {
    // === Installation lifecycle ===
    /// Hooks to run before installation
    #[serde(default)]
    pub pre_install: Vec<String>,
    /// Hooks to run after installation
    #[serde(default)]
    pub post_install: Vec<String>,
    /// Hooks to run before uninstallation
    #[serde(default)]
    pub pre_uninstall: Vec<String>,
    /// Hooks to run after uninstallation
    #[serde(default)]
    pub post_uninstall: Vec<String>,

    // === Activation lifecycle ===
    /// Hooks to run before activating a runtime version
    #[serde(default)]
    pub pre_activate: Vec<String>,
    /// Hooks to run after activating a runtime version
    #[serde(default)]
    pub post_activate: Vec<String>,
    /// Hooks to run before deactivating a runtime version
    #[serde(default)]
    pub pre_deactivate: Vec<String>,
    /// Hooks to run after deactivating a runtime version
    #[serde(default)]
    pub post_deactivate: Vec<String>,

    // === Execution lifecycle ===
    /// Hooks to run before executing the runtime
    #[serde(default)]
    pub pre_run: Vec<String>,
    /// Hooks to run after executing the runtime
    #[serde(default)]
    pub post_run: Vec<String>,

    // === Error handling hooks ===
    /// Hooks to run when installation fails
    #[serde(default)]
    pub on_install_error: Vec<String>,
    /// Hooks to run when requested version is not found
    #[serde(default)]
    pub on_version_not_found: Vec<String>,
    /// Hooks to run when health check fails
    #[serde(default)]
    pub on_health_check_fail: Vec<String>,

    // === Hook behavior configuration ===
    /// Hook execution configuration
    #[serde(default)]
    pub config: Option<HooksConfig>,
}

/// Hook execution configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HooksConfig {
    /// Whether to fail the operation if a hook fails
    #[serde(default = "default_true")]
    pub fail_on_error: bool,
    /// Timeout for each hook in milliseconds
    #[serde(default = "default_hook_timeout")]
    pub timeout_ms: u64,
    /// Whether to run hooks in parallel
    #[serde(default)]
    pub parallel: bool,
}

impl Default for HooksConfig {
    fn default() -> Self {
        Self {
            fail_on_error: true,
            timeout_ms: 30000,
            parallel: false,
        }
    }
}
