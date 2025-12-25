//! pre-commit provider for vx
//!
//! This crate provides the pre-commit framework provider for vx.
//! pre-commit is a framework for managing and maintaining multi-language pre-commit hooks.
//!
//! # Example
//!
//! ```ignore
//! use vx_provider_pre_commit::create_provider;
//!
//! let provider = create_provider();
//! assert_eq!(provider.name(), "pre-commit");
//! ```

mod provider;
mod runtime;

pub use provider::PreCommitProvider;
pub use runtime::PreCommitRuntime;

use std::sync::Arc;
use vx_runtime::Provider;

/// Create a new pre-commit provider instance
///
/// This is the main entry point for the provider, used by the registry.
pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(PreCommitProvider::new())
}
