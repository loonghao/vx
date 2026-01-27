//! Execute command implementation - Dynamic proxy for runtime execution
//!
//! This module provides transparent command forwarding with:
//! - Automatic dependency detection
//! - Auto-installation of missing runtimes
//! - Smart routing to vx-managed or system runtimes
//! - Support for runtime@version syntax

use crate::ui::UI;
use anyhow::Result;
use vx_resolver::{Executor, ResolverConfig};
use vx_runtime::{CacheMode, ProviderRegistry, RuntimeContext};

/// Handle the execute command
pub async fn handle(
    registry: &ProviderRegistry,
    context: &RuntimeContext,
    runtime_name: &str,
    args: &[String],
    use_system_path: bool,
    inherit_env: bool,
    cache_mode: CacheMode,
) -> Result<()> {
    handle_with_version(
        registry,
        context,
        runtime_name,
        None,
        args,
        use_system_path,
        inherit_env,
        cache_mode,
    )
    .await
}

/// Handle the execute command with optional version specification
///
/// Supports `runtime@version` syntax:
/// - `vx yarn@1.21.1 global add terminalizer`
/// - `vx node@20 --version`
pub async fn handle_with_version(
    registry: &ProviderRegistry,
    context: &RuntimeContext,
    runtime_name: &str,
    version: Option<&str>,
    args: &[String],
    use_system_path: bool,
    inherit_env: bool,
    cache_mode: CacheMode,
) -> Result<()> {
    let exit_code = execute_runtime_with_version(
        registry,
        context,
        runtime_name,
        version,
        args,
        use_system_path,
        inherit_env,
        cache_mode,
    )
    .await?;

    // Exit with the appropriate code
    // Note: exit_code 130 indicates Ctrl+C termination (128 + SIGINT)
    // We exit silently in this case to avoid confusing error messages
    if exit_code != 0 {
        std::process::exit(exit_code);
    }
    Ok(())
}

/// Execute runtime with given arguments using the executor
///
/// This function:
/// 1. Resolves the runtime and its dependencies
/// 2. Auto-installs missing components if enabled
/// 3. Forwards the command to the appropriate executable
pub async fn execute_runtime(
    registry: &ProviderRegistry,
    context: &RuntimeContext,
    runtime_name: &str,
    args: &[String],
    use_system_path: bool,
    cache_mode: CacheMode,
) -> Result<i32> {
    execute_runtime_with_version(
        registry,
        context,
        runtime_name,
        None,
        args,
        use_system_path,
        false,
        cache_mode,
    )
    .await
}

/// Execute runtime with given arguments and optional version constraint
///
/// This function:
/// 1. Resolves the runtime and its dependencies
/// 2. If a version is specified, ensures that version is installed
/// 3. Auto-installs missing components if enabled
/// 4. Forwards the command to the appropriate executable
pub async fn execute_runtime_with_version(
    registry: &ProviderRegistry,
    context: &RuntimeContext,
    runtime_name: &str,
    version: Option<&str>,
    args: &[String],
    use_system_path: bool,
    inherit_env: bool,
    cache_mode: CacheMode,
) -> Result<i32> {
    // Print debug information
    if let Some(ver) = version {
        UI::debug(&format!(
            "Executing: {}@{} {}",
            runtime_name,
            ver,
            args.join(" ")
        ));
    } else {
        UI::debug(&format!("Executing: {} {}", runtime_name, args.join(" ")));
    }

    // Build executor configuration
    let config = (if use_system_path {
        ResolverConfig::default().system_only()
    } else {
        ResolverConfig::default()
    })
    .with_resolution_cache_mode(cache_mode);

    // Create the executor with runtime map from manifests (RFC 0017: single source of truth)
    let manifests = crate::registry::load_manifests_with_overrides();
    let runtime_map = vx_resolver::RuntimeMap::from_manifests(&manifests);

    let executor = Executor::new(config, registry, context, runtime_map)?;

    // Execute the runtime with optional version and environment inheritance
    executor
        .execute_with_version_and_env(runtime_name, version, args, inherit_env)
        .await
}

/// Execute a runtime using system PATH only (simple fallback)
pub async fn execute_system_runtime(runtime_name: &str, args: &[String]) -> Result<i32> {
    vx_resolver::execute_system_runtime(runtime_name, args).await
}

/// Check if a runtime is available (either vx-managed or system)
pub fn is_runtime_available(runtime_name: &str) -> bool {
    let config = ResolverConfig::default();
    let manifests = crate::registry::load_manifests_with_overrides();
    let runtime_map = vx_resolver::RuntimeMap::from_manifests(&manifests);
    if let Ok(resolver) = vx_resolver::Resolver::new(config, runtime_map) {
        resolver.check_runtime_status(runtime_name).is_available()
    } else {
        // Fallback to which
        which::which(runtime_name).is_ok()
    }
}

/// Get information about a runtime's availability
pub fn get_runtime_info(runtime_name: &str) -> RuntimeAvailability {
    let config = ResolverConfig::default();
    let manifests = crate::registry::load_manifests_with_overrides();
    let runtime_map = vx_resolver::RuntimeMap::from_manifests(&manifests);
    if let Ok(resolver) = vx_resolver::Resolver::new(config, runtime_map) {
        let status = resolver.check_runtime_status(runtime_name);
        match status {
            vx_resolver::RuntimeStatus::VxManaged { version, path } => {
                RuntimeAvailability::VxManaged {
                    version,
                    path: path.display().to_string(),
                }
            }
            vx_resolver::RuntimeStatus::SystemAvailable { path } => RuntimeAvailability::System {
                path: path.display().to_string(),
            },
            vx_resolver::RuntimeStatus::NotInstalled => RuntimeAvailability::NotInstalled,
            vx_resolver::RuntimeStatus::Unknown => RuntimeAvailability::Unknown,
        }
    } else {
        RuntimeAvailability::Unknown
    }
}

/// Runtime availability information
#[derive(Debug, Clone)]
pub enum RuntimeAvailability {
    /// Runtime is managed by vx
    VxManaged { version: String, path: String },
    /// Runtime is available in system PATH
    System { path: String },
    /// Runtime is not installed
    NotInstalled,
    /// Runtime status is unknown
    Unknown,
}

impl RuntimeAvailability {
    /// Check if the runtime is available
    pub fn is_available(&self) -> bool {
        matches!(
            self,
            RuntimeAvailability::VxManaged { .. } | RuntimeAvailability::System { .. }
        )
    }
}
