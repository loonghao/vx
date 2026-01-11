//! NASM provider for vx
//!
//! This crate provides the NASM (Netwide Assembler) provider for vx.
//! NASM is a portable 80x86 and x86-64 assembler.
//!
//! # Example
//!
//! ```ignore
//! use vx_provider_nasm::create_provider;
//!
//! let provider = create_provider();
//! assert_eq!(provider.name(), "nasm");
//! ```

pub mod config;
mod provider;
mod runtime;

pub use config::NasmUrlBuilder;
pub use provider::NasmProvider;
pub use runtime::NasmRuntime;

use std::sync::Arc;
use vx_runtime::Provider;

/// Create a new NASM provider instance
///
/// This is the main entry point for the provider, used by the registry.
pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(NasmProvider::new())
}
