//! # vx-starlark
//!
//! Starlark scripting support for vx providers.
//!
//! This crate provides:
//! - **Starlark runtime integration** for executing provider scripts
//! - **Sandbox security model** for safe script execution
//! - **ProviderContext API** for Starlark scripts to interact with vx
//! - **Hybrid format support** for both TOML and Starlark providers
//!
//! ## Overview
//!
//! ```ignore
//! use vx_starlark::{StarlarkProvider, SandboxConfig};
//!
//! // Load a Starlark provider
//! let provider = StarlarkProvider::load("path/to/provider.star")?;
//!
//! // Call provider functions
//! let versions = provider.call("fetch_versions", &ctx)?;
//! ```

pub mod context;
pub mod error;
pub mod provider;
pub mod sandbox;
pub mod stdlib;

// Re-exports
pub use context::ProviderContext;
pub use error::{Error, Result};
pub use provider::StarlarkProvider;
pub use sandbox::SandboxConfig;

/// Starlark provider file extension
pub const STARLARK_EXTENSION: &str = "star";

/// Default provider filename for Starlark
pub const PROVIDER_FILENAME: &str = "provider.star";
