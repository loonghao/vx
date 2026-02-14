//! Runtime Resolver
//!
//! This crate provides a universal runtime command forwarding system that:
//! - Detects runtime dependencies automatically
//! - Auto-installs missing runtimes when needed
//! - Forwards commands to the appropriate runtime executable
//!
//! # Architecture
//!
//! The resolver uses a layered approach:
//! 1. **Runtime Registry** - Maps runtime names and aliases to their specifications
//! 2. **Dependency Resolver** - Determines what needs to be installed
//! 3. **Auto Installer** - Downloads and installs missing dependencies
//! 4. **Command Forwarder** - Executes the actual runtime command
//!
//! # Example
//!
//! ```rust,ignore
//! use vx_resolver::{Executor, ResolverConfig};
//!
//! async fn example() -> anyhow::Result<()> {
//!     let executor = Executor::new(ResolverConfig::default())?;
//!
//!     // Execute: vx npm install express
//!     // This will:
//!     // 1. Detect npm requires node
//!     // 2. Check if node is installed
//!     // 3. Auto-install node if missing
//!     // 4. Forward "install express" to npm
//!     let exit_code = executor.execute("npm", &["install".into(), "express".into()]).await?;
//!     Ok(())
//! }
//! ```

mod config;
mod executor;
mod resolution_cache;
mod resolver;
mod runtime_index;
mod runtime_map;
mod runtime_request;
mod runtime_spec;
pub mod version;

pub use config::{DEFAULT_RESOLUTION_CACHE_TTL, ResolverConfig};
pub use executor::{
    BUNDLE_DIR, BUNDLE_MANIFEST, BundleContext, BundleManifest, BundledToolInfo, Executor,
    ProjectToolsConfig, clear_bin_dir_cache, execute_bundle, execute_system_runtime,
    exit_code_from_status, has_bundle, init_bin_dir_cache, invalidate_bin_dir_cache,
    is_ctrl_c_exit, is_online, save_bin_dir_cache, try_get_bundle_context,
};

// Pipeline types (RFC 0029)
pub use executor::pipeline::{
    EnsureError, EnsureStage, ExecuteError, ExecuteStage, ExecutionConfig, ExecutionPipeline,
    ExecutionPlan, InstallStatus, PipelineError, PlannedRuntime, PrepareError, PrepareStage,
    PreparedExecution, ProxyConfig, ResolveError, ResolveRequest, ResolveStage, Stage,
    VersionResolution, VersionSource, WithDepRequest,
};
pub use resolution_cache::{
    RESOLUTION_CACHE_DIR_NAME, RESOLUTION_CACHE_SCHEMA_VERSION, ResolutionCache, ResolutionCacheKey,
};
pub use resolver::{
    IncompatibleDependency, ResolutionResult, ResolvedGraph, Resolver, RuntimeStatus,
    UnsupportedPlatformRuntime,
};
pub use runtime_index::{
    DEFAULT_INDEX_TTL, IndexData, IndexMetadata, RUNTIME_INDEX_DIR, RUNTIME_INDEX_SCHEMA_VERSION,
    RuntimeIndex, RuntimeIndexEntry,
};
pub use runtime_map::RuntimeMap;
pub use runtime_request::RuntimeRequest;
pub use runtime_spec::{Ecosystem, RuntimeDependency, RuntimeSpec};

// Re-export version types for convenience
pub use version::{
    ApplyConfigResult, BoundsCheckResult, Conflict, ConflictDetectionError, ConflictDetector,
    DependencyRequirement, LockFile, LockFileError, LockFileInconsistency, LockedTool,
    MergedRequirement, RangeConstraint, RangeOp, ResolvedVersion, SolverConfig, SolverError,
    SolverResult, SolverStatus, Version, VersionConstraint, VersionRangeConfig,
    VersionRangeResolver, VersionRequest, VersionSolver, VersionStrategy,
};

/// Result type for resolver operations
pub type Result<T> = anyhow::Result<T>;
