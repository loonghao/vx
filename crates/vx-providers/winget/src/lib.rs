//! Windows Package Manager (winget) provider for vx
//!
//! This crate provides winget support using the vx-runtime traits.
//! winget is the official package manager for Windows, built-in on Windows 11
//! and available on Windows 10 via App Installer.

mod config;
mod provider;
mod runtime;

pub use config::WingetConfig;
pub use provider::WingetProvider;
pub use runtime::WingetRuntime;

use std::sync::Arc;
use vx_runtime::Provider;

/// Create a new winget provider instance
pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(WingetProvider::new())
}
