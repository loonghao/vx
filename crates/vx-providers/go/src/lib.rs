//! Go provider for vx
//!
//! This provider includes:
//! - Go programming language runtime (`go`)

mod config;
mod provider;
mod runtime;

pub use config::GoUrlBuilder;
pub use provider::GoProvider;
pub use runtime::GoRuntime;

use std::sync::Arc;
use vx_runtime::Provider;

/// Create a new Go provider instance
pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(GoProvider::new())
}
