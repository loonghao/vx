//! Command Handler trait and context
//!
//! This module provides a unified interface for all CLI commands,
//! enabling better extensibility and maintainability.

use crate::cli::OutputFormat;
use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;
use vx_runtime::{CacheMode, ProviderRegistry, RuntimeContext, TestCommand, TestConfig};
use vx_starlark::provider::types::PackageAlias;

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
    /// Whether to inherit system environment variables in isolated environments
    pub inherit_env: bool,
    /// Cache mode for network-dependent operations (versions/resolutions)
    pub cache_mode: CacheMode,
    /// Verbose output mode
    pub verbose: bool,
    /// Debug output mode
    pub debug: bool,
    /// Additional runtime dependencies to inject (--with flag)
    ///
    /// Each entry is a runtime spec like "bun" or "bun@1.1.0"
    pub with_deps: Vec<String>,
    /// Output format (RFC 0031: unified structured output)
    pub output_format: OutputFormat,
    /// Disable automatic installation of missing tools.
    ///
    /// When true, vx errors instead of auto-installing missing runtimes.
    /// Controlled by `--no-auto-install` flag or `VX_NO_AUTO_INSTALL=1`.
    pub no_auto_install: bool,
    /// Field mask: only return these fields in structured output.
    ///
    /// Empty = return all fields. Controlled by `--fields name,version,...`.
    pub fields: Vec<String>,
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

    /// Builder method: set inherit_env
    pub fn with_inherit_env(mut self, value: bool) -> Self {
        self.inherit_env = value;
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

    /// Builder method: set with_deps
    pub fn with_with_deps(mut self, value: Vec<String>) -> Self {
        self.with_deps = value;
        self
    }

    /// Builder method: set output_format
    pub fn with_output_format(mut self, value: OutputFormat) -> Self {
        self.output_format = value;
        self
    }

    /// Builder method: set no_auto_install
    pub fn with_no_auto_install(mut self, value: bool) -> Self {
        self.no_auto_install = value;
        self
    }

    /// Builder method: set fields mask
    pub fn with_fields(mut self, value: Vec<String>) -> Self {
        self.fields = value;
        self
    }

    /// Check if JSON output is requested
    pub fn is_json(&self) -> bool {
        self.output_format == OutputFormat::Json
    }

    /// Returns true if automatic installation of missing tools is disabled.
    ///
    /// Checks both the `--no-auto-install` flag and `VX_NO_AUTO_INSTALL` env var.
    pub fn auto_install_disabled(&self) -> bool {
        self.no_auto_install
            || matches!(
                std::env::var("VX_NO_AUTO_INSTALL").as_deref(),
                Ok("1") | Ok("true") | Ok("yes")
            )
    }

    /// Returns the active field mask (empty = all fields).
    pub fn field_mask(&self) -> &[String] {
        &self.fields
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
                inherit_env: false,
                cache_mode: CacheMode::Normal,
                verbose,
                debug,
                with_deps: Vec::new(),
                output_format: OutputFormat::default(),
                no_auto_install: false,
                fields: Vec::new(),
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

    /// Check if inheriting system environment variables
    pub fn inherit_env(&self) -> bool {
        self.options.inherit_env
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

    /// Get additional runtime dependencies (--with flag)
    pub fn with_deps(&self) -> &[String] {
        &self.options.with_deps
    }

    /// Get the output format (RFC 0031)
    pub fn output_format(&self) -> OutputFormat {
        self.options.output_format
    }

    /// Check if JSON output is requested
    pub fn is_json(&self) -> bool {
        self.options.is_json()
    }

    /// Returns true if automatic installation of missing tools is disabled.
    pub fn auto_install_disabled(&self) -> bool {
        self.options.auto_install_disabled()
    }

    /// Returns the active field mask (empty = all fields).
    pub fn field_mask(&self) -> &[String] {
        self.options.field_mask()
    }

    /// Get test configuration for a runtime from the global ProviderHandle registry.
    ///
    /// Returns `Some(TestConfig)` if the runtime has test commands defined in provider.star,
    /// or `None` if no test configuration is available.
    pub fn get_test_config(&self, runtime_name: &str) -> Option<TestConfig> {
        // Try to get from global ProviderHandle registry (sync read)
        let registry = vx_starlark::handle::GLOBAL_REGISTRY.try_read().ok()?;
        let handle = registry.get(runtime_name)?;

        // Find the matching runtime meta
        let runtime_meta = handle
            .runtime_metas()
            .iter()
            .find(|r| r.name == runtime_name || r.aliases.contains(&runtime_name.to_string()))?;

        if runtime_meta.test_commands.is_empty() {
            return None;
        }

        // Convert TestCommandMeta -> TestCommand
        let functional_commands = runtime_meta
            .test_commands
            .iter()
            .map(|tc| {
                use vx_runtime::TestCheckType as RtType;
                use vx_starlark::provider::types::TestCheckType as MetaType;
                let check_type = match tc.check_type {
                    MetaType::CheckPath => RtType::CheckPath,
                    MetaType::CheckNotPath => RtType::CheckNotPath,
                    MetaType::CheckEnv => RtType::CheckEnv,
                    MetaType::CheckNotEnv => RtType::CheckNotEnv,
                    MetaType::CheckFile => RtType::CheckFile,
                    MetaType::Command => RtType::Command,
                };
                TestCommand {
                    command: tc.command.clone(),
                    check_type,
                    expect_success: tc.expect_success,
                    expected_output: tc.expected_output.clone(),
                    expected_exit_code: None,
                    name: tc.name.clone(),
                    timeout_ms: tc.timeout_ms,
                }
            })
            .collect();

        Some(TestConfig {
            functional_commands,
            ..Default::default()
        })
    }

    /// Get runtime manifest definition by name (compatibility shim)
    ///
    /// Returns a minimal compat struct with only the `test` field populated.
    /// Prefer `get_test_config()` for new code.
    pub fn get_runtime_manifest(&self, runtime_name: &str) -> Option<RuntimeManifestCompat> {
        Some(RuntimeManifestCompat {
            test: self.get_test_config(runtime_name),
        })
    }

    /// Get the package alias for a runtime from the global ProviderHandle registry (RFC 0033)
    ///
    /// When a provider has `package_alias` in its metadata, executing `vx <name>` should
    /// be routed to `vx <ecosystem>:<package>` via the package execution path.
    pub fn get_package_alias(&self, runtime_name: &str) -> Option<PackageAlias> {
        crate::registry::find_package_alias(runtime_name)
    }
}

/// Compatibility shim for code that previously used RuntimeDef
///
/// Only exposes the `test` field that is actually used by test/handler.rs.
/// Will be removed once all callers are migrated to `get_test_config()`.
pub struct RuntimeManifestCompat {
    pub test: Option<TestConfig>,
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
