//! VX Runtime Core
//!
//! This crate provides the core traits and types for vx runtime system.
//! It is designed to be lightweight and fast to compile, containing only:
//!
//! - `Runtime` trait: Core abstraction for executable runtimes
//! - `Provider` trait: Container for related runtimes
//! - `ProviderRegistry`: Registry for all providers
//! - Core types: VersionInfo, InstallResult, ExecutionResult, etc.
//! - Platform detection: Os, Arch, Platform
//!
//! # Why a separate core crate?
//!
//! This crate enables fast compilation for provider development:
//! - Providers only need to depend on `vx-runtime-core` (compiles in ~5-8s)
//! - Heavy dependencies (HTTP, archive handling) stay in `vx-runtime`
//! - This allows ~57 providers to compile in parallel with the core
//!
//! # Example
//!
//! ```rust,ignore
//! use vx_runtime_core::{Runtime, Provider, RuntimeContext, VersionInfo, Ecosystem};
//!
//! struct MyRuntime;
//!
//! #[async_trait::async_trait]
//! impl Runtime for MyRuntime {
//!     fn name(&self) -> &str { "myruntime" }
//!
//!     fn ecosystem(&self) -> Ecosystem { Ecosystem::Custom("myeco") }
//!
//!     async fn fetch_versions(&self, ctx: &RuntimeContext) -> anyhow::Result<Vec<VersionInfo>> {
//!         Ok(vec![])
//!     }
//! }
//! ```

pub mod ecosystem;
pub mod platform;
pub mod provider;
pub mod registry;
pub mod runtime;
pub mod traits;
pub mod types;

// Re-exports for convenience
pub use ecosystem::Ecosystem;
pub use platform::{Arch, Libc, Os, Platform, compare_semver};
pub use provider::{PackageManager, Provider};
pub use registry::{PlatformError, ProviderRegistry};
pub use runtime::{Runtime, VerificationResult};
pub use traits::{CommandExecutor, FileSystem, HttpClient, Installer, PathProvider};
pub use types::{
    ExecutionPrep, ExecutionResult, InstallResult, RuntimeDependency, RuntimeSpec, VersionInfo,
};

// Re-export from vx-manifest for convenience
pub use vx_manifest::{PlatformConstraint, PlatformExclusion};
