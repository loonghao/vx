//! Execute command implementation - Dynamic proxy for runtime execution
//!
//! This module provides transparent command forwarding with:
//! - Automatic dependency detection
//! - Auto-installation of missing runtimes
//! - Smart routing to vx-managed or system runtimes

use crate::ui::UI;
use anyhow::Result;
use vx_resolver::{Executor, ResolverConfig};
use vx_runtime::ProviderRegistry;

/// Handle the execute command
pub async fn handle(
    _registry: &ProviderRegistry,
    runtime_name: &str,
    args: &[String],
    use_system_path: bool,
) -> Result<()> {
    let exit_code = execute_runtime(runtime_name, args, use_system_path).await?;
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
    runtime_name: &str,
    args: &[String],
    use_system_path: bool,
) -> Result<i32> {
    UI::debug(&format!("Executing: {} {}", runtime_name, args.join(" ")));

    // Build executor configuration
    let config = if use_system_path {
        ResolverConfig::default().system_only()
    } else {
        ResolverConfig::default()
    };

    // Create the executor
    let executor = Executor::new(config)?;

    // Execute the runtime
    executor.execute(runtime_name, args).await
}

/// Execute a runtime using system PATH only (simple fallback)
pub async fn execute_system_runtime(runtime_name: &str, args: &[String]) -> Result<i32> {
    vx_resolver::execute_system_runtime(runtime_name, args).await
}

/// Check if a runtime is available (either vx-managed or system)
pub fn is_runtime_available(runtime_name: &str) -> bool {
    let config = ResolverConfig::default();
    if let Ok(resolver) = vx_resolver::Resolver::new(config) {
        resolver.check_runtime_status(runtime_name).is_available()
    } else {
        // Fallback to which
        which::which(runtime_name).is_ok()
    }
}

/// Get information about a runtime's availability
pub fn get_runtime_info(runtime_name: &str) -> RuntimeAvailability {
    let config = ResolverConfig::default();
    if let Ok(resolver) = vx_resolver::Resolver::new(config) {
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

// Backward compatibility aliases
pub use execute_runtime as execute_tool;
pub use execute_system_runtime as execute_system_tool;
pub use get_runtime_info as get_tool_info;
pub use is_runtime_available as is_tool_available;
pub type ToolAvailability = RuntimeAvailability;
