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

/// The raw content of `provider.star`, embedded at compile time.
///
/// This is the single source of truth for provider metadata (name, description,
/// aliases, platform constraints, etc.).  The `build.rs` script ensures Cargo
/// re-compiles this crate whenever `provider.star` changes.
pub const PROVIDER_STAR: &str = include_str!("../provider.star");

/// Lazily-parsed metadata from `provider.star`.
///
/// Use this to access provider/runtime metadata without spinning up the full
/// Starlark engine.  The metadata is parsed once on first access.
pub fn star_metadata() -> &'static vx_starlark::StarMetadata {
    use std::sync::OnceLock;
    static META: OnceLock<vx_starlark::StarMetadata> = OnceLock::new();
    META.get_or_init(|| vx_starlark::StarMetadata::parse(PROVIDER_STAR))
}
