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
mod r#override;
mod platform;
mod provider;
mod satisfies;

pub use ecosystem::Ecosystem;
pub use error::ManifestError;
pub use loader::ManifestLoader;
pub use platform::{Arch, Os, Platform, PlatformConstraint, PlatformExclusion};
pub use provider::{
    AliasNormalize, ArchiveLayoutConfig, BinaryLayoutConfig, CacheConfig, CommandDef,
    ConstraintRule, DEFAULT_INHERIT_SYSTEM_VARS, DependencyDef, DetectionConfig,
    DirectoryNormalize, DownloadConfig, DownloadType, EffectiveNormalizeConfig, EnvConfig,
    EnvVarConfig, ExecutableConfig, ExecutableNormalize, HealthConfig, HooksConfig, HooksDef,
    InlineTestScripts, InstallStrategyDef, LayoutConfig, MachineFlagsConfig, MirrorConfig,
    MirrorStrategy, NormalizeAction, NormalizeConfig, OutputColorConfig, OutputConfig,
    PackageAlias, PinningStrategy, PlatformBinaryConfig, PlatformConfig, PlatformNormalizeConfig,
    PlatformTestCommands, PlatformsDef, ProvidedToolDef, ProviderManifest, ProviderMeta,
    RuntimeDef, SYSTEM_PATH_PREFIXES, ScriptTypeDef, ShellCompletionsConfig, ShellConfig,
    SystemDepTypeDef, SystemDependencyDef, SystemDepsConfigDef, SystemInstallConfigDef,
    TestCommand, TestConfig, TestPlatformConfig, VersionRangeConfig, VersionSourceDef,
    filter_system_path,
};

pub use r#override::{ProviderOverride, RuntimeOverride, apply_override, extract_provider_name};
pub use satisfies::{
    RangeConstraint, RangeOp, Version, VersionConstraint, VersionRequest, VersionSatisfies,
};

/// Result type for manifest operations
pub type Result<T> = std::result::Result<T, ManifestError>;
