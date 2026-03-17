//! # vx-starlark
//!
//! Starlark scripting support for vx providers.
//!
//! This crate provides:
//! - **Starlark runtime integration** for executing provider scripts
//! - **Sandbox security model** for safe script execution
//! - **ProviderContext API** for Starlark scripts to interact with vx
//! - **@vx//stdlib module system** for shared utilities (Buck2-inspired load())
//! - **Two-phase execution** (Analysis → Execution, Buck2-inspired)
//! - **Incremental analysis cache** (content-hash based, Buck2-inspired)
//!
//! ## Overview
//!
//! ```ignore
//! use vx_starlark::{StarlarkProvider, SandboxConfig};
//!
//! // Load a Starlark provider
//! let provider = StarlarkProvider::load("path/to/provider.star").await?;
//!
//! // Call provider functions
//! let versions = provider.fetch_versions().await?;
//! ```

pub mod context;
pub mod engine;
pub mod error;
pub mod handle;
pub mod loader;
pub mod provider;
pub mod provider_test_support;
pub mod sandbox;
pub mod stdlib;

/// Test mocks for provider tests (only available with #[cfg(test)] or in dev builds)
#[cfg(any(test, feature = "test-mocks"))]
pub mod test_mocks;

// Re-exports
pub use context::ProviderContext;
pub use engine::{ProviderLint, StarlarkEngine};
pub use error::{Error, Result};
pub use handle::{
    PostInstallOps, ProviderHandle, ProviderHandleRegistry, VersionFilter, global_registry,
    global_registry_mut,
};
pub use loader::VxModuleLoader;
pub use provider::version_cache::{
    DEFAULT_VERSION_CACHE_TTL_SECS, DEV_VERSION_CACHE_TTL_SECS, VersionCache, VersionCacheStats,
    global_version_cache,
};
pub use provider::{
    EnvOp, InstallLayout, PostExtractAction, ProviderMeta, RuntimeMeta, StarlarkProvider,
    apply_env_ops, build_runtimes, create_provider, make_download_url_fn, make_fetch_versions_fn,
    make_install_layout_fn,
};
pub use sandbox::SandboxConfig;
pub use vx_star_metadata::{StarMetadata, StarRuntimeMeta};

/// Starlark provider file extension
pub const STARLARK_EXTENSION: &str = "star";

/// Default provider filename for Starlark
pub const PROVIDER_FILENAME: &str = "provider.star";
