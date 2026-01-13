//! Rust provider for vx
//!
//! This crate provides Rust toolchain support for vx.
//! vx manages Rust versions directly, replacing the need for rustup.
//!
//! # Runtimes
//!
//! - `rustc` (alias: `rust`) - The Rust compiler
//! - `cargo` - Rust package manager and build tool
//!
//! # Example
//!
//! ```ignore
//! // Install Rust 1.75.0
//! vx install rust@1.75.0
//!
//! // Use cargo
//! vx cargo build
//!
//! // Use rustc
//! vx rustc --version
//! ```

mod config;
mod provider;
mod runtime;

pub use config::RustUrlBuilder;
pub use provider::RustProvider;
pub use runtime::{CargoRuntime, RustcRuntime};

use std::sync::Arc;
use vx_runtime::Provider;

/// Create a new Rust provider instance
pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(RustProvider::new())
}
