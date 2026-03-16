//! Execute command implementation - Dynamic proxy for runtime execution
//!
//! This module provides transparent command forwarding with:
//! - Automatic dependency detection
//! - Auto-installation of missing runtimes
//! - Smart routing to vx-managed or system runtimes
//! - Support for runtime@version syntax
//! - Support for --with flag to inject additional runtimes

use crate::ui::UI;
use anyhow::Result;
use vx_core::WithDependency;
use vx_resolver::{Executor, ResolverConfig};
use vx_runtime::{CacheMode, ProviderRegistry, RuntimeContext};

// ──────────────────────────────────────────────────────────────────────────────
// Options struct
// ──────────────────────────────────────────────────────────────────────────────

/// All optional parameters for a runtime execution request.
///
/// Use [`ExecuteOptions::default()`] as a starting point and override the
/// fields that are relevant for your call site.
///
/// # Example
/// ```rust
/// let opts = ExecuteOptions {
///     version: Some("20"),
///     cache_mode: CacheMode::Force,
///     ..Default::default()
/// };
/// ```
#[derive(Debug, Default)]
pub struct ExecuteOptions<'a> {
    /// Pin to a specific version (e.g. `"20"` or `"1.21.1"`).
    pub version: Option<&'a str>,
    /// Override the executable name inside the runtime (e.g. `"npx"` inside `node`).
    pub executable: Option<&'a str>,
    /// Route through system PATH only, skipping vx-managed installations.
    pub use_system_path: bool,
    /// Inherit the caller's environment variables into the subprocess.
    pub inherit_env: bool,
    /// Cache mode for version resolution.
    pub cache_mode: CacheMode,
    /// Additional runtimes injected via `--with`.
    pub with_deps: &'a [WithDependency],
}

// ──────────────────────────────────────────────────────────────────────────────
// Public API  (kept stable for existing callers)
// ──────────────────────────────────────────────────────────────────────────────

/// Handle the execute command (exits the process on non-zero exit code).
pub async fn handle(
    registry: &ProviderRegistry,
    context: &RuntimeContext,
    runtime_name: &str,
    args: &[String],
    use_system_path: bool,
    inherit_env: bool,
    cache_mode: CacheMode,
) -> Result<()> {
    let opts = ExecuteOptions {
        use_system_path,
        inherit_env,
        cache_mode,
        ..Default::default()
    };
    handle_with_options(registry, context, runtime_name, args, opts).await
}

/// Handle the execute command with optional version specification.
///
/// Supports `runtime@version` syntax:
/// - `vx yarn@1.21.1 global add terminalizer`
/// - `vx node@20 --version`
#[allow(clippy::too_many_arguments)]
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
    let opts = ExecuteOptions {
        version,
        use_system_path,
        inherit_env,
        cache_mode,
        ..Default::default()
    };
    handle_with_options(registry, context, runtime_name, args, opts).await
}

/// Handle the execute command with `--with` dependencies.
///
/// Supports injecting additional runtimes via `--with` flag:
/// - `vx --with bun npm:opencode-ai@latest::opencode`
/// - `vx --with bun@1.1.0 --with deno node my-script.js`
#[allow(clippy::too_many_arguments)]
pub async fn handle_with_deps(
    registry: &ProviderRegistry,
    context: &RuntimeContext,
    runtime_name: &str,
    version: Option<&str>,
    executable: Option<&str>,
    args: &[String],
    use_system_path: bool,
    inherit_env: bool,
    cache_mode: CacheMode,
    with_deps: &[WithDependency],
) -> Result<()> {
    let opts = ExecuteOptions {
        version,
        executable,
        use_system_path,
        inherit_env,
        cache_mode,
        with_deps,
    };
    handle_with_options(registry, context, runtime_name, args, opts).await
}

/// Execute runtime and return the exit code (does not exit the process).
pub async fn execute_runtime(
    registry: &ProviderRegistry,
    context: &RuntimeContext,
    runtime_name: &str,
    args: &[String],
    use_system_path: bool,
    cache_mode: CacheMode,
) -> Result<i32> {
    let opts = ExecuteOptions {
        use_system_path,
        cache_mode,
        ..Default::default()
    };
    execute_runtime_with_options(registry, context, runtime_name, args, opts).await
}

/// Execute runtime with an optional version constraint and return the exit code.
#[allow(clippy::too_many_arguments)]
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
    let opts = ExecuteOptions {
        version,
        use_system_path,
        inherit_env,
        cache_mode,
        ..Default::default()
    };
    execute_runtime_with_options(registry, context, runtime_name, args, opts).await
}

/// Execute runtime with version, executable override, and `--with` dependencies.
#[allow(clippy::too_many_arguments)]
pub async fn execute_runtime_with_deps(
    registry: &ProviderRegistry,
    context: &RuntimeContext,
    runtime_name: &str,
    version: Option<&str>,
    executable: Option<&str>,
    args: &[String],
    use_system_path: bool,
    inherit_env: bool,
    cache_mode: CacheMode,
    with_deps: &[WithDependency],
) -> Result<i32> {
    let opts = ExecuteOptions {
        version,
        executable,
        use_system_path,
        inherit_env,
        cache_mode,
        with_deps,
    };
    execute_runtime_with_options(registry, context, runtime_name, args, opts).await
}

// ──────────────────────────────────────────────────────────────────────────────
// Core implementations
// ──────────────────────────────────────────────────────────────────────────────

/// Entry point for handle-style callers: runs the command and exits on failure.
pub async fn handle_with_options(
    registry: &ProviderRegistry,
    context: &RuntimeContext,
    runtime_name: &str,
    args: &[String],
    opts: ExecuteOptions<'_>,
) -> Result<()> {
    let exit_code =
        execute_runtime_with_options(registry, context, runtime_name, args, opts).await?;

    // exit_code 130 = Ctrl+C (128 + SIGINT): exit silently to avoid noise.
    if exit_code != 0 {
        std::process::exit(exit_code);
    }
    Ok(())
}

/// Core execution logic: resolves, auto-installs, and forwards the command.
pub async fn execute_runtime_with_options(
    registry: &ProviderRegistry,
    context: &RuntimeContext,
    runtime_name: &str,
    args: &[String],
    opts: ExecuteOptions<'_>,
) -> Result<i32> {
    // Debug logging
    match opts.version {
        Some(ver) => UI::debug(&format!(
            "Executing: {}@{} {}",
            runtime_name,
            ver,
            args.join(" ")
        )),
        None => UI::debug(&format!("Executing: {} {}", runtime_name, args.join(" "))),
    }
    if let Some(exe) = opts.executable {
        UI::debug(&format!("Executable override: {}", exe));
    }
    if !opts.with_deps.is_empty() {
        UI::debug(&format!(
            "With dependencies: {}",
            opts.with_deps
                .iter()
                .map(|d| d.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }

    // Build executor configuration
    let config = (if opts.use_system_path {
        ResolverConfig::default().system_only()
    } else {
        ResolverConfig::default()
    })
    .with_resolution_cache_mode(opts.cache_mode);

    // Create the executor with runtime map from provider.star handles (RFC-0037)
    crate::registry::ensure_provider_metadata_initialized().await;
    let runtime_map = crate::registry::build_runtime_map();
    let executor = Executor::new(config, registry, context, runtime_map)?;

    executor
        .execute_with_with_deps(
            runtime_name,
            opts.version,
            opts.executable,
            args,
            opts.inherit_env,
            opts.with_deps,
        )
        .await
}

/// Execute a runtime using system PATH only (simple fallback)
pub async fn execute_system_runtime(runtime_name: &str, args: &[String]) -> Result<i32> {
    vx_resolver::execute_system_runtime(runtime_name, args).await
}

/// Check if a runtime is available (either vx-managed or system)
pub fn is_runtime_available(runtime_name: &str) -> bool {
    let config = ResolverConfig::default();
    let runtime_map = vx_resolver::RuntimeMap::empty();
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
    let runtime_map = vx_resolver::RuntimeMap::empty();
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
