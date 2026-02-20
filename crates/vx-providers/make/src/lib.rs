//! GNU Make provider for vx
//!
//! GNU Make is a tool which controls the generation of executables and other
//! non-source files of a program from the program's source files.

mod config;
mod provider;
mod runtime;

pub use provider::MakeProvider;
pub use runtime::MakeRuntime;

use std::sync::Arc;
use vx_runtime::Provider;

/// Create a new Make provider
pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(MakeProvider::new())
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
