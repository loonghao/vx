//! YASM provider for vx
//!
//! This crate provides the YASM (Yet Another Assembler) provider for vx.
//! YASM is a modular assembler with support for multiple output formats.
//!
//! # Example
//!
//! ```ignore
//! use vx_provider_yasm::create_provider;
//!
//! let provider = create_provider();
//! assert_eq!(provider.name(), "yasm");
//! ```

mod config;
mod provider;
mod runtime;

pub use config::YasmUrlBuilder;
pub use provider::YasmProvider;
pub use runtime::YasmRuntime;

use std::sync::Arc;
use vx_runtime::Provider;

/// Create a new YASM provider instance
///
/// This is the main entry point for the provider, used by the registry.
pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(YasmProvider::new())
}
