//! PNPM package manager provider for vx
//!
//! This crate provides PNPM runtime support using the new vx-runtime traits.

mod config;
mod provider;
mod runtime;

pub use config::PnpmConfig;
pub use provider::PnpmProvider;
pub use runtime::PnpmRuntime;

use std::sync::Arc;
use vx_runtime::Provider;

/// Create a new PNPM provider instance
pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(PnpmProvider::new())
}
