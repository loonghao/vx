//! NuGet package manager provider for vx
//!
//! This crate provides NuGet support using the vx-runtime traits.
//! NuGet is the package manager for .NET, providing tools for creating,
//! publishing, and consuming .NET packages.
//!
//! On Windows, the standalone nuget.exe is available.
//! On other platforms, use `dotnet nuget` commands via .NET SDK.

mod config;
mod provider;
mod runtime;

pub use config::NugetConfig;
pub use provider::NugetProvider;
pub use runtime::NugetRuntime;

use std::sync::Arc;
use vx_runtime::Provider;

/// Create a new NuGet provider instance
pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(NugetProvider::new())
}
