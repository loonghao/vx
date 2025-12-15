//! Executor configuration
//!
//! Configuration options for the dynamic executor including
//! auto-installation behavior, timeout settings, and more.

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Configuration for the dynamic executor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutorConfig {
    /// Whether to automatically install missing tools
    pub auto_install: bool,

    /// Whether to automatically install missing dependencies
    pub auto_install_dependencies: bool,

    /// Whether to prefer vx-managed tools over system tools
    pub prefer_vx_managed: bool,

    /// Fallback to system PATH if vx-managed tool not found
    pub fallback_to_system: bool,

    /// Timeout for tool execution (None = no timeout)
    pub execution_timeout: Option<Duration>,

    /// Timeout for tool installation
    pub install_timeout: Duration,

    /// Whether to show progress during installation
    pub show_progress: bool,

    /// Whether to prompt user before auto-installation
    pub prompt_before_install: bool,

    /// Maximum parallel installations
    pub max_parallel_installs: usize,

    /// Whether to verify tool after installation
    pub verify_after_install: bool,
}

impl Default for ExecutorConfig {
    fn default() -> Self {
        Self {
            auto_install: true,
            auto_install_dependencies: true,
            prefer_vx_managed: true,
            fallback_to_system: true,
            execution_timeout: None,
            install_timeout: Duration::from_secs(300), // 5 minutes
            show_progress: true,
            prompt_before_install: false,
            max_parallel_installs: 4,
            verify_after_install: true,
        }
    }
}

impl ExecutorConfig {
    /// Create a new configuration with defaults
    pub fn new() -> Self {
        Self::default()
    }

    /// Disable auto-installation
    pub fn without_auto_install(mut self) -> Self {
        self.auto_install = false;
        self.auto_install_dependencies = false;
        self
    }

    /// Enable prompting before installation
    pub fn with_prompt(mut self) -> Self {
        self.prompt_before_install = true;
        self
    }

    /// Use system tools only (no vx-managed)
    pub fn system_only(mut self) -> Self {
        self.prefer_vx_managed = false;
        self.fallback_to_system = true;
        self.auto_install = false;
        self
    }

    /// Set execution timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.execution_timeout = Some(timeout);
        self
    }

    /// Disable progress display
    pub fn quiet(mut self) -> Self {
        self.show_progress = false;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = ExecutorConfig::default();
        assert!(config.auto_install);
        assert!(config.auto_install_dependencies);
        assert!(config.prefer_vx_managed);
        assert!(config.fallback_to_system);
    }

    #[test]
    fn test_config_builders() {
        let config = ExecutorConfig::new()
            .without_auto_install()
            .with_prompt()
            .quiet();

        assert!(!config.auto_install);
        assert!(config.prompt_before_install);
        assert!(!config.show_progress);
    }

    #[test]
    fn test_system_only_config() {
        let config = ExecutorConfig::new().system_only();

        assert!(!config.prefer_vx_managed);
        assert!(config.fallback_to_system);
        assert!(!config.auto_install);
    }
}
