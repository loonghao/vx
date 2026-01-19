//! GNU Make provider for vx
//!
//! GNU Make is a tool which controls the generation of executables and other
//! non-source files of a program from the program's source files.

mod config;
mod provider;
mod runtime;

pub use provider::MakeProvider;
pub use runtime::MakeRuntime;

use std::sync::Arc;
use vx_runtime::Provider;

/// Create a new Make provider
pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(MakeProvider::new())
}
