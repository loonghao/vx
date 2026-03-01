//! VX Runtime Core
//!
//! This crate provides shared primitive types used by `vx-runtime` and related crates.
//! It is designed to be lightweight and fast to compile.
//!
//! ## Provided types
//!
//! - **Platform detection**: `Os`, `Arch`, `Platform`
//! - **Normalization config**: `MirrorConfig`, `NormalizeConfig`, etc.
//! - **Core types**: `VersionInfo`, `InstallResult`, `ExecutionResult`, etc.
//! - **Version constraints**: `VersionConstraint`, `RangeOp`, etc. (from `vx-versions`)
//! - **DI traits**: `HttpClient`, `FileSystem`, `PathProvider`, `Installer`, etc.
//! - **Ecosystem**: `Ecosystem` enum (from `vx-versions`)

pub mod normalize;
pub mod platform;
pub mod traits;
pub mod types;

// Re-exports from vx-versions (canonical definitions live there)
pub use vx_versions::{
    Ecosystem, RangeConstraint, RangeOp, Version, VersionConstraint, VersionInfo, VersionRequest,
};

// Re-exports from local modules
pub use normalize::{
    AliasNormalize, DirectoryNormalize, EffectiveNormalizeConfig, ExecutableNormalize,
    MirrorConfig, NormalizeAction, NormalizeConfig, PlatformNormalizeConfig,
};
pub use platform::{Arch, Libc, Os, Platform, compare_semver};
pub use traits::{
    CommandExecutor, CorePathProvider, FileSystem, HttpClient, Installer, PathProvider,
};
pub use types::{ExecutionPrep, ExecutionResult, InstallResult, RuntimeDependency, RuntimeSpec};
