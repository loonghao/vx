//! Spack provider for vx
//!
//! This crate provides the Spack provider for vx.
//! Spack is a flexible package manager designed for supercomputers, Linux, and macOS.
//! It makes installing scientific software easy by automatically handling dependencies,
//! multiple versions, configurations, platforms, and compilers.
//!
//! # Features
//!
//! - Install Spack on Windows, macOS, and Linux
//! - Version management
//! - Support for HPC and scientific computing workflows
//! - Compiler management and detection
//!
//! # Example
//!
//! ```ignore
//! use vx_provider_spack::create_provider;
//!
//! let provider = create_provider();
//! assert_eq!(provider.name(), "spack");
//! ```

mod config;
mod provider;
mod runtime;

pub use config::SpackUrlBuilder;
pub use provider::SpackProvider;
pub use runtime::SpackRuntime;

use std::sync::Arc;
use vx_runtime::Provider;

/// Create a new Spack provider instance
///
/// This is the main entry point for the provider, used by the registry.
pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(SpackProvider::new())
}
