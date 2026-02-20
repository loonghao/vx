//! Homebrew package manager provider for vx
//!
//! This crate provides Homebrew package manager support using the vx-runtime traits.
//! Homebrew is the missing package manager for macOS (and Linux).

mod config;
mod provider;
mod runtime;

pub use config::BrewConfig;
pub use provider::BrewProvider;
pub use runtime::BrewRuntime;

use std::sync::Arc;
use vx_runtime::Provider;

/// Create a new Homebrew provider instance
pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(BrewProvider::new())
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
