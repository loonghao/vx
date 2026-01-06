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
//!
//! # User Overrides
//!
//! Users can create `.override.toml` files to customize constraints:
//!
//! ```rust,ignore
//! use vx_manifest::{ProviderManifest, ProviderOverride, apply_override};
//!
//! let mut manifest = ProviderManifest::load("provider.toml")?;
//! let override_config = ProviderOverride::load("yarn.override.toml")?;
//! apply_override(&mut manifest, &override_config);
//! ```

mod ecosystem;
mod error;
mod loader;
mod provider;
mod r#override;
mod satisfies;

pub use ecosystem::Ecosystem;
pub use error::ManifestError;
pub use loader::ManifestLoader;
pub use provider::{
    ConstraintRule, DependencyDef, ExecutableConfig, HooksDef, PlatformConfig, PlatformsDef,
    ProviderManifest, ProviderMeta, RuntimeDef, VersionSourceDef,
};
pub use r#override::{apply_override, extract_provider_name, ProviderOverride, RuntimeOverride};
pub use satisfies::{
    RangeConstraint, RangeOp, Version, VersionConstraint, VersionRequest, VersionSatisfies,
};

/// Result type for manifest operations
pub type Result<T> = std::result::Result<T, ManifestError>;
