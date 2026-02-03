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
