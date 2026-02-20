//! # vx-starlark
//!
//! Starlark scripting support for vx providers.
//!
//! This crate provides:
//! - **Starlark runtime integration** for executing provider scripts
//! - **Sandbox security model** for safe script execution
//! - **ProviderContext API** for Starlark scripts to interact with vx
//! - **Hybrid format support** for both TOML and Starlark providers
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
pub mod loader;
pub mod metadata;
pub mod provider;
pub mod sandbox;
pub mod stdlib;

// Re-exports
pub use context::ProviderContext;
pub use engine::StarlarkEngine;
pub use error::{Error, Result};
pub use loader::VxModuleLoader;
pub use metadata::{StarMetadata, StarRuntimeMeta};
pub use provider::StarlarkProvider;
pub use sandbox::SandboxConfig;

/// Starlark provider file extension
pub const STARLARK_EXTENSION: &str = "star";

/// Default provider filename for Starlark
pub const PROVIDER_FILENAME: &str = "provider.star";
