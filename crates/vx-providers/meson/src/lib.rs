//! Meson build system provider for vx
//!
//! Meson is an open source build system meant to be both extremely fast,
//! and as user friendly as possible.

mod provider;
mod runtime;

pub use provider::MesonProvider;
pub use runtime::MesonRuntime;

use std::sync::Arc;
use vx_runtime::Provider;

/// Create a new Meson provider
pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(MesonProvider::new())
}
