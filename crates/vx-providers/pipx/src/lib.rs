//! pipx provider for vx
//!
//! pipx is a tool to help you install and run end-user applications written in Python.
//! It's similar to npx for Node.js or uvx for Python.
//!
//! This provider installs pipx via uv (uv tool install pipx).

mod provider;
mod runtime;

pub use provider::PipxProvider;
pub use runtime::PipxRuntime;

use std::sync::Arc;
use vx_runtime::Provider;

/// Create a new pipx provider instance
pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(PipxProvider::new())
}
