//! Rust provider for vx
//!
//! This crate provides Rust toolchain support for vx.
//! Rust is installed via rustup, the official Rust toolchain installer.
//!
//! # Installation Methods
//!
//! - Windows: `winget install Rustlang.Rustup`
//! - macOS: `brew install rustup-init && rustup-init -y`
//! - Linux: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y`
//!
//! # Runtimes
//!
//! - `rustup` - The Rust toolchain installer (primary)
//! - `rustc` (alias: `rust`) - The Rust compiler (provided by rustup)
//! - `cargo` - Rust package manager and build tool (provided by rustup)
//!
//! # Example
//!
//! ```ignore
//! // Install rustup (installs rustc and cargo automatically)
//! vx install rustup
//!
//! // Use cargo
//! vx cargo build
//!
//! // Use rustc
//! vx rustc --version
//! ```

mod provider;
mod runtime;

pub use provider::RustProvider;
pub use runtime::{CargoRuntime, RustcRuntime, RustupRuntime};

use std::sync::Arc;
use vx_runtime::Provider;

/// Create a new Rust provider instance
pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(RustProvider::new())
}
