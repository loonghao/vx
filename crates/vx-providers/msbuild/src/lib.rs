//! MSBuild provider for vx
//!
//! This crate provides the MSBuild provider for vx.
//! MSBuild is the Microsoft Build Engine bundled with .NET SDK and Visual Studio.
//!
//! ## RFC 0028: Bundled Runtime Pattern
//!
//! MSBuild is a "bundled runtime" - it's not independently downloadable but rather
//! bundled with either:
//! 1. **.NET SDK**: Cross-platform MSBuild (`dotnet msbuild`)
//! 2. **Visual Studio**: Windows-only standalone MSBuild.exe
//!
//! When a user runs `vx msbuild MyProject.csproj`, vx will:
//! 1. Check `is_version_installable()` → returns `false` (bundled)
//! 2. Call `prepare_execution()`:
//!    - Find .NET SDK or Visual Studio installation
//!    - Return execution configuration with `command_prefix = ["dotnet"]`
//! 3. Execute: `dotnet msbuild MyProject.csproj`
//!
//! ## Usage Examples
//!
//! ```bash
//! # Build a project (uses .NET SDK's msbuild)
//! vx msbuild MyProject.csproj
//!
//! # Build with specific configuration
//! vx msbuild MyProject.sln /p:Configuration=Release
//!
//! # Use dotnet directly for more control
//! vx dotnet msbuild MyProject.csproj -p:Configuration=Release
//! ```

mod provider;
mod runtime;

pub use provider::MsbuildProvider;
pub use runtime::MsbuildRuntime;

use std::sync::Arc;
use vx_runtime::Provider;

/// Create a new MSBuild provider instance
///
/// This is the main entry point for the provider, used by the registry.
pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(MsbuildProvider::new())
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
