//! Provider trait definition
//!
//! A Provider is a container for related runtimes.

use crate::runtime::Runtime;
use crate::Platform;
use std::sync::Arc;

/// Trait for package managers (npm, yarn, pip, etc.)
///
/// Package managers are tools that install packages/dependencies,
/// as opposed to runtimes which are the execution environments.
pub trait PackageManager: Send + Sync {
    /// Package manager name (e.g., "npm", "yarn", "pip")
    fn name(&self) -> &str;

    /// Description
    fn description(&self) -> &str {
        "A package manager"
    }

    /// The runtime this package manager belongs to
    fn runtime(&self) -> &str;
}

/// Provider is a container for related runtimes
///
/// For example, NodeProvider provides:
/// - NodeRuntime (node)
/// - NpmRuntime (npm)
/// - NpxRuntime (npx)
///
/// # Example
///
/// ```rust,no_run
/// use vx_runtime::{Provider, Runtime};
/// use std::sync::Arc;
///
/// struct NodeProvider;
///
/// impl Provider for NodeProvider {
///     fn name(&self) -> &str {
///         "node"
///     }
///
///     fn description(&self) -> &str {
///         "Node.js JavaScript runtime and tools"
///     }
///
///     fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
///         vec![
///             // Arc::new(NodeRuntime::new()),
///             // Arc::new(NpmRuntime::new()),
///             // Arc::new(NpxRuntime::new()),
///         ]
///     }
/// }
/// ```
pub trait Provider: Send + Sync {
    /// Provider name
    fn name(&self) -> &str;

    /// Provider description
    fn description(&self) -> &str;

    /// Get all runtimes provided by this provider
    fn runtimes(&self) -> Vec<Arc<dyn Runtime>>;

    /// Get package managers provided by this provider (optional)
    fn package_managers(&self) -> Vec<Arc<dyn PackageManager>> {
        vec![]
    }

    /// Check if this provider supports a runtime by name or alias
    fn supports(&self, name: &str) -> bool {
        self.runtimes()
            .iter()
            .any(|r| r.name() == name || r.aliases().contains(&name))
    }

    /// Get a specific runtime by name or alias
    fn get_runtime(&self, name: &str) -> Option<Arc<dyn Runtime>> {
        self.runtimes()
            .into_iter()
            .find(|r| r.name() == name || r.aliases().contains(&name))
    }

    /// Check if this provider supports the given platform
    ///
    /// Default implementation checks if any runtime in this provider
    /// supports the platform. Override for provider-level platform constraints.
    fn is_platform_supported(&self, platform: &Platform) -> bool {
        // By default, a provider is supported if any of its runtimes are supported
        self.runtimes()
            .iter()
            .any(|r| r.is_platform_supported(platform))
    }

    /// Get runtimes that support the given platform
    fn supported_runtimes_for(&self, platform: &Platform) -> Vec<Arc<dyn Runtime>> {
        self.runtimes()
            .into_iter()
            .filter(|r| r.is_platform_supported(platform))
            .collect()
    }
}
