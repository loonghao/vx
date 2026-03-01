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
//! - **Version constraints**: `VersionConstraint`, `RangeOp`, etc.
//! - **DI traits**: `HttpClient`, `FileSystem`, `PathProvider`, `Installer`, etc.
//! - **Ecosystem**: `Ecosystem` enum
//!
//! ## Note
//!
//! The `Runtime` trait and related `Provider`/`ProviderRegistry` abstractions
//! live in `vx-runtime`, not here. This crate only contains the shared primitive
//! types that `vx-runtime` (and `vx-env`) need without pulling in heavy deps.

pub mod ecosystem;
pub mod normalize;
pub mod platform;
pub mod traits;
pub mod types;
pub mod version;

// Re-exports for convenience
pub use ecosystem::Ecosystem;
pub use normalize::{
    AliasNormalize, DirectoryNormalize, EffectiveNormalizeConfig, ExecutableNormalize,
    MirrorConfig, NormalizeAction, NormalizeConfig, PlatformNormalizeConfig,
};
pub use platform::{Arch, Libc, Os, Platform, compare_semver};
pub use traits::{
    CommandExecutor, CorePathProvider, FileSystem, HttpClient, Installer, PathProvider,
};
pub use types::{
    ExecutionPrep, ExecutionResult, InstallResult, RuntimeDependency, RuntimeSpec, VersionInfo,
};
pub use version::{RangeConstraint, RangeOp, Version, VersionConstraint, VersionRequest};
