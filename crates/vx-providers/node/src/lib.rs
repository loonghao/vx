//! Node.js provider for vx
//!
//! This provider includes:
//! - Node.js runtime (`node`)
//! - NPM package manager (`npm`)
//! - NPX package runner (`npx`)

mod config;
mod provider;
mod runtime;

pub use config::NodeUrlBuilder;
pub use provider::NodeProvider;
pub use runtime::{NodeRuntime, NpmRuntime, NpxRuntime};

use std::sync::Arc;
use vx_runtime::Provider;

/// Create a new Node.js provider instance
pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(NodeProvider::new())
}
