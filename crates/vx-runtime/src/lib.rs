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

pub mod constraints;
pub mod context;
pub mod ecosystem;
pub mod impls;
pub mod manifest_registry;
pub mod package_runtime;
pub mod platform;
pub mod plugin;
pub mod provider;
pub mod registry;
pub mod runtime;
pub mod testing;
pub mod traits;
pub mod types;
pub mod version_cache;
pub mod version_resolver;

// Re-exports
pub use context::{ExecutionContext, GitHubReleaseOptions, RuntimeContext};
pub use ecosystem::Ecosystem;
pub use impls::{
    create_runtime_context, create_runtime_context_with_base, RealCommandExecutor, RealFileSystem,
    RealHttpClient, RealInstaller, RealPathProvider,
};
pub use package_runtime::{InstallMethod, PackageRuntime};
pub use platform::{compare_semver, Arch, Os, Platform};
pub use provider::Provider;
pub use registry::{PlatformError, ProviderRegistry};
pub use runtime::{Runtime, VerificationResult};
pub use traits::{CommandExecutor, FileSystem, HttpClient, Installer, PathProvider};
pub use types::{ExecutionResult, InstallResult, RuntimeDependency, RuntimeSpec, VersionInfo};

// Re-export testing utilities
pub use testing::{
    mock_context, MockCommandExecutor, MockFileSystem, MockHttpClient, MockInstaller,
    MockPathProvider,
};
pub use version_cache::{
    CacheMode, CacheStats, VersionCache, DEFAULT_CACHE_TTL, LONG_CACHE_TTL, SHORT_CACHE_TTL,
};
pub use version_resolver::VersionResolver;

// Constraints system
pub use constraints::{
    get_default_constraints, init_constraints_from_manifests,
    load_constraints_from_manifest_content, ConstraintRule, ConstraintsRegistry,
    DependencyConstraint, ManifestVersionPattern, VersionPattern, DEFAULT_CONSTRAINTS,
};

// Manifest-driven registry
pub use manifest_registry::{ManifestRegistry, RuntimeMetadata};

// Re-export platform types from vx-manifest for convenience
pub use vx_manifest::{Arch as ManifestArch, Os as ManifestOs, Platform as ManifestPlatform};
pub use vx_manifest::{PlatformConstraint, PlatformExclusion};

// Plugin system
pub use plugin::{default_plugin_paths, PluginLoader, ProviderLoader, ProviderPlugin};
