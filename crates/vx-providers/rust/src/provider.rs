//! Rust provider implementation
//!
//! vx manages Rust toolchains directly, replacing the need for rustup.

use crate::runtime::{CargoRuntime, RustcRuntime};
use std::sync::Arc;
use vx_runtime::{Provider, Runtime};

/// Rust provider
///
/// Provides rustc (Rust compiler) and cargo (package manager).
/// vx handles version management directly, eliminating the need for rustup.
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
        "Provides Rust toolchain support (rustc, cargo)"
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![
            Arc::new(RustcRuntime::new()),
            Arc::new(CargoRuntime::new()),
        ]
    }

    fn supports(&self, name: &str) -> bool {
        matches!(name, "rust" | "rustc" | "cargo")
    }

    fn get_runtime(&self, name: &str) -> Option<Arc<dyn Runtime>> {
        match name {
            "rust" | "rustc" => Some(Arc::new(RustcRuntime::new())),
            "cargo" => Some(Arc::new(CargoRuntime::new())),
            _ => None,
        }
    }
}
