//! VX Runtime System
//!
//! This crate provides the core runtime system for vx, including:
//!
//! - `Runtime` trait: Core abstraction for executable runtimes (node, go, uv, etc.)
//! - `Provider` trait: Container for related runtimes
//! - `ProviderRegistry`: Registry for all providers
//! - Dependency injection via `RuntimeContext` and `ExecutionContext`
//! - Mock implementations for testing
//!
//! # Architecture
//!
//! ```text
//! Provider (e.g., NodeProvider)
//!    ├── Runtime: NodeRuntime
//!    ├── Runtime: NpmRuntime
//!    └── Runtime: NpxRuntime
//! ```
//!
//! # Example
//!
//! ```rust,ignore
//! use vx_runtime::{Runtime, Provider, RuntimeContext, VersionInfo};
//!
//! // Implement a custom runtime
//! struct MyRuntime;
//!
//! #[async_trait::async_trait]
//! impl Runtime for MyRuntime {
//!     fn name(&self) -> &str { "myruntime" }
//!
//!     async fn fetch_versions(&self, ctx: &RuntimeContext) -> anyhow::Result<Vec<VersionInfo>> {
//!         // Fetch versions from API
//!         Ok(vec![])
//!     }
//! }
//! ```

pub mod context;
pub mod ecosystem;
pub mod impls;
pub mod platform;
pub mod provider;
pub mod registry;
pub mod runtime;
pub mod testing;
pub mod traits;
pub mod types;

// Re-exports
pub use context::{ExecutionContext, RuntimeContext};
pub use ecosystem::Ecosystem;
pub use impls::{
    create_runtime_context, create_runtime_context_with_base, RealCommandExecutor, RealFileSystem,
    RealHttpClient, RealInstaller, RealPathProvider,
};
pub use platform::{Arch, Os, Platform};
pub use provider::Provider;
pub use registry::ProviderRegistry;
pub use runtime::Runtime;
pub use traits::{CommandExecutor, FileSystem, HttpClient, Installer, PathProvider};
pub use types::{ExecutionResult, InstallResult, RuntimeDependency, RuntimeSpec, VersionInfo};

// Re-export testing utilities
pub use testing::{
    mock_context, MockCommandExecutor, MockFileSystem, MockHttpClient, MockInstaller,
    MockPathProvider,
};
