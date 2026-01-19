//! Resolver configuration
//!
//! Configuration options for the resolver including
//! auto-installation behavior, timeout settings, and more.

use serde::{Deserialize, Serialize};
use std::time::Duration;
use vx_runtime::CacheMode;

/// Default resolution cache TTL (15 minutes)
pub const DEFAULT_RESOLUTION_CACHE_TTL: Duration = Duration::from_secs(15 * 60);

/// Configuration for the resolver
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolverConfig {
    /// Whether to automatically install missing runtimes
    pub auto_install: bool,

    /// Whether to automatically install missing dependencies
    pub auto_install_dependencies: bool,

    /// Whether to prefer vx-managed runtimes over system runtimes
    pub prefer_vx_managed: bool,

    /// Fallback to system PATH if vx-managed runtime not found
    pub fallback_to_system: bool,

    /// Cache mode for resolution cache
    pub resolution_cache_mode: CacheMode,

    /// TTL for resolution cache entries
    pub resolution_cache_ttl: Duration,

    /// Timeout for runtime execution (None = no timeout)
    pub execution_timeout: Option<Duration>,

    /// Timeout for runtime installation
    pub install_timeout: Duration,

    /// Whether to show progress during installation
    pub show_progress: bool,

    /// Whether to prompt user before auto-installation
    pub prompt_before_install: bool,

    /// Maximum parallel installations
    pub max_parallel_installs: usize,

    /// Whether to verify runtime after installation
    pub verify_after_install: bool,

    /// Whether to inherit vx-managed tools PATH in subprocesses
    ///
    /// When enabled, subprocesses spawned by vx-managed tools will have access
    /// to all other vx-managed tools in PATH. This allows tools like `just` to
    /// invoke other vx tools (e.g., `uvx`, `npm`) without needing the `vx` prefix.
    ///
    /// This is enabled by default.
    pub inherit_vx_path: bool,
}

impl Default for ResolverConfig {
    fn default() -> Self {
        Self {
            auto_install: true,
            auto_install_dependencies: true,
            prefer_vx_managed: true,
            fallback_to_system: true,
            resolution_cache_mode: CacheMode::Normal,
            resolution_cache_ttl: DEFAULT_RESOLUTION_CACHE_TTL,
            execution_timeout: None,
            install_timeout: Duration::from_secs(300), // 5 minutes
            show_progress: true,
            prompt_before_install: false,
            max_parallel_installs: 4,
            verify_after_install: true,
            inherit_vx_path: true, // Enable subprocess PATH inheritance by default
        }
    }
}

impl ResolverConfig {
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

    /// Use system runtimes only (no vx-managed)
    pub fn system_only(mut self) -> Self {
        self.prefer_vx_managed = false;
        self.fallback_to_system = true;
        self.auto_install = false;
        self
    }

    /// Set resolution cache mode
    pub fn with_resolution_cache_mode(mut self, mode: CacheMode) -> Self {
        self.resolution_cache_mode = mode;
        self
    }

    /// Set resolution cache TTL
    pub fn with_resolution_cache_ttl(mut self, ttl: Duration) -> Self {
        self.resolution_cache_ttl = ttl;
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

    /// Set whether to inherit vx-managed tools PATH in subprocesses
    ///
    /// When enabled, subprocesses will have access to all vx-managed tools.
    /// This allows justfiles, makefiles, and other tools to use vx-managed
    /// tools directly without needing the `vx` prefix.
    pub fn with_inherit_vx_path(mut self, inherit: bool) -> Self {
        self.inherit_vx_path = inherit;
        self
    }
}
