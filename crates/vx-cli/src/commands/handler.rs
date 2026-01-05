//! Command Handler trait and context
//!
//! This module provides a unified interface for all CLI commands,
//! enabling better extensibility and maintainability.

use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;
use vx_runtime::{CacheMode, ProviderRegistry, RuntimeContext};

/// Global CLI options
///
/// Extracted from Cli struct for easier management.
/// Adding new global options only requires:
/// 1. Add field here
/// 2. Add field in Cli struct
/// 3. Update From<&Cli> implementation
#[derive(Debug, Clone, Default)]
pub struct GlobalOptions {
    /// Whether to use system PATH for tool lookup
    pub use_system_path: bool,
    /// Cache mode for network-dependent operations (versions/resolutions)
    pub cache_mode: CacheMode,
    /// Verbose output mode
    pub verbose: bool,
    /// Debug output mode
    pub debug: bool,
}

impl GlobalOptions {
    /// Create new GlobalOptions with all defaults
    pub fn new() -> Self {
        Self::default()
    }

    /// Builder method: set use_system_path
    pub fn with_use_system_path(mut self, value: bool) -> Self {
        self.use_system_path = value;
        self
    }

    /// Builder method: set cache_mode
    pub fn with_cache_mode(mut self, value: CacheMode) -> Self {
        self.cache_mode = value;
        self
    }

    /// Builder method: set verbose
    pub fn with_verbose(mut self, value: bool) -> Self {
        self.verbose = value;
        self
    }

    /// Builder method: set debug
    pub fn with_debug(mut self, value: bool) -> Self {
        self.debug = value;
        self
    }
}

/// Unified context for command execution
///
/// Contains all dependencies that commands might need.
/// Commands should only use what they require.
pub struct CommandContext {
    /// Provider registry for runtime lookups
    pub registry: Arc<ProviderRegistry>,
    /// Runtime context for installations and version management
    pub runtime_context: Arc<RuntimeContext>,
    /// Global CLI options
    pub options: GlobalOptions,
}

impl CommandContext {
    /// Create a new command context with GlobalOptions
    pub fn new(
        registry: ProviderRegistry,
        runtime_context: RuntimeContext,
        options: GlobalOptions,
    ) -> Self {
        Self {
            registry: Arc::new(registry),
            runtime_context: Arc::new(runtime_context),
            options,
        }
    }

    /// Create a new command context with pre-existing Arc references
    pub fn new_with_arc(
        registry: Arc<ProviderRegistry>,
        runtime_context: Arc<RuntimeContext>,
        options: GlobalOptions,
    ) -> Self {
        Self {
            registry,
            runtime_context,
            options,
        }
    }

    /// Create a new command context with individual options (backwards compatible)
    pub fn with_options(
        registry: ProviderRegistry,
        runtime_context: RuntimeContext,
        use_system_path: bool,
        verbose: bool,
        debug: bool,
    ) -> Self {
        Self::new(
            registry,
            runtime_context,
            GlobalOptions {
                use_system_path,
                cache_mode: CacheMode::Normal,
                verbose,
                debug,
            },
        )
    }

    /// Get a reference to the registry
    pub fn registry(&self) -> &ProviderRegistry {
        &self.registry
    }

    /// Get a reference to the runtime context
    pub fn runtime_context(&self) -> &RuntimeContext {
        &self.runtime_context
    }

    /// Get global options
    pub fn options(&self) -> &GlobalOptions {
        &self.options
    }

    /// Check if using system PATH
    pub fn use_system_path(&self) -> bool {
        self.options.use_system_path
    }

    /// Get current cache mode
    pub fn cache_mode(&self) -> CacheMode {
        self.options.cache_mode
    }

    /// Check if verbose mode is enabled
    pub fn verbose(&self) -> bool {
        self.options.verbose
    }

    /// Check if debug mode is enabled
    pub fn debug(&self) -> bool {
        self.options.debug
    }
}

/// Trait for command handlers
///
/// All CLI commands implement this trait, providing a unified
/// interface for command execution.
#[async_trait]
pub trait CommandHandler: Send + Sync {
    /// Execute the command with the given context
    async fn execute(&self, ctx: &CommandContext) -> Result<()>;

    /// Get the command name (for logging/debugging)
    fn name(&self) -> &'static str {
        "unknown"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_global_options_default() {
        let opts = GlobalOptions::default();
        assert!(!opts.use_system_path);
        assert!(!opts.verbose);
        assert!(!opts.debug);
    }

    #[test]
    fn test_global_options_builder() {
        let opts = GlobalOptions::new()
            .with_use_system_path(true)
            .with_verbose(true)
            .with_debug(false);

        assert!(opts.use_system_path);
        assert!(opts.verbose);
        assert!(!opts.debug);
    }
}
