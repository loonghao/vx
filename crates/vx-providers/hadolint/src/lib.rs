//! Hadolint (Dockerfile linter) provider for vx
//!
//! This crate provides the Hadolint provider for vx.
//! Hadolint is a Dockerfile linter that helps you build best practice
//! Docker images by parsing Dockerfiles and checking them against
//! proven best practices.
//!
//! Homepage: <https://github.com/hadolint/hadolint>

pub mod config;
mod provider;
mod runtime;

pub use config::HadolintUrlBuilder;
pub use provider::HadolintProvider;
pub use runtime::HadolintRuntime;

use std::sync::Arc;
use vx_runtime::Provider;

/// Create a new Hadolint provider instance
///
/// This is the main entry point for the provider, used by the registry.
pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(HadolintProvider::new())
}
