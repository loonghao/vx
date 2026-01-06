//! Provider Manifest Library
//!
//! This crate provides types and parsing for `provider.toml` manifest files.
//! It enables declarative definition of Provider metadata, Runtime definitions,
//! and dependency constraints.
//!
//! # Example
//!
//! ```rust,ignore
//! use vx_manifest::ProviderManifest;
//!
//! let manifest = ProviderManifest::load("provider.toml")?;
//! for runtime in &manifest.runtimes {
//!     println!("Runtime: {}", runtime.name);
//!     for constraint in &runtime.constraints {
//!         if constraint.matches("1.22.22") {
//!             for dep in &constraint.requires {
//!                 println!("  Requires: {} {}", dep.runtime, dep.version);
//!             }
//!         }
//!     }
//! }
//! ```

mod ecosystem;
mod error;
mod loader;
mod provider;
mod satisfies;

pub use ecosystem::Ecosystem;
pub use error::ManifestError;
pub use loader::ManifestLoader;
pub use provider::{
    ConstraintRule, DependencyDef, ExecutableConfig, HooksDef, PlatformConfig, PlatformsDef,
    ProviderManifest, ProviderMeta, RuntimeDef, VersionSourceDef,
};
pub use satisfies::{
    RangeConstraint, RangeOp, Version, VersionConstraint, VersionRequest, VersionSatisfies,
};

/// Result type for manifest operations
pub type Result<T> = std::result::Result<T, ManifestError>;
