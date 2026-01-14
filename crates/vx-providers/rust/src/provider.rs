//! Rust provider implementation
//!
//! Rust is installed via rustup, the official Rust toolchain installer.

use crate::runtime::{CargoRuntime, RustcRuntime, RustupRuntime};
use std::sync::Arc;
use vx_runtime::{Provider, Runtime};

/// Rust provider
///
/// Provides rustup, rustc, and cargo.
/// rustup is the primary runtime that manages the Rust toolchain.
#[derive(Debug, Default)]
pub struct RustProvider;

impl RustProvider {
    /// Create a new Rust provider
    pub fn new() -> Self {
        Self
    }
}

impl Provider for RustProvider {
    fn name(&self) -> &str {
        "rust"
    }

    fn description(&self) -> &str {
        "Provides Rust toolchain support (rustup, rustc, cargo)"
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![
            Arc::new(RustupRuntime::new()),
            Arc::new(RustcRuntime::new()),
            Arc::new(CargoRuntime::new()),
        ]
    }

    fn supports(&self, name: &str) -> bool {
        matches!(name, "rust" | "rustup" | "rustc" | "cargo")
    }

    fn get_runtime(&self, name: &str) -> Option<Arc<dyn Runtime>> {
        match name {
            "rustup" => Some(Arc::new(RustupRuntime::new())),
            "rust" | "rustc" => Some(Arc::new(RustcRuntime::new())),
            "cargo" => Some(Arc::new(CargoRuntime::new())),
            _ => None,
        }
    }
}
