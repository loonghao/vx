//! Chocolatey package manager provider for vx
//!
//! This crate provides Chocolatey package manager support using the vx-runtime traits.
//! Chocolatey is the package manager for Windows, providing easy installation of
//! software, tools, and libraries.

mod config;
mod provider;
mod runtime;

pub use config::ChocoConfig;
pub use provider::ChocoProvider;
pub use runtime::ChocoRuntime;

use std::sync::Arc;
use vx_runtime::Provider;

/// Create a new Chocolatey provider instance
pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(ChocoProvider::new())
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
