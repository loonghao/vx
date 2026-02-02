//! .NET SDK provider for vx
//!
//! This crate provides the .NET SDK provider for vx.
//! .NET is a free, cross-platform, open-source developer platform for building many different types of applications.
//! With .NET, you can use multiple languages, editors, and libraries to build for web, mobile, desktop, games, and IoT.
//!
//! # Example
//!
//! ```ignore
//! use vx_provider_dotnet::create_provider;
//!
//! let provider = create_provider();
//! assert_eq!(provider.name(), "dotnet");
//! ```

mod config;
mod provider;
mod runtime;

pub use config::DotnetUrlBuilder;
pub use provider::DotnetProvider;
pub use runtime::DotnetRuntime;

use std::sync::Arc;
use vx_runtime::Provider;

/// Create a new .NET provider instance
///
/// This is the main entry point for the provider, used by the registry.
pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(DotnetProvider::new())
}
