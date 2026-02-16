//! Manifest-driven provider registry (split architecture)
//!
//! This module provides a clean separation of concerns for manifest-driven
//! provider registration, split into three sub-modules:
//!
//! - [`loader`]: Manifest loading from disk, embedded sources, and in-memory
//! - [`index`]: Fast runtime/alias/provider metadata lookups via pre-built indices
//! - [`builder`]: Provider construction from manifests + factories, with `BuildResult`
//!
//! # Architecture (RFC 0029)
//!
//! ```text
//! ManifestLoader → Vec<ProviderManifest>
//!                          ↓
//!              ┌───────────┴───────────┐
//!              ↓                       ↓
//!       ManifestIndex           ProviderBuilder
//!   (metadata queries)     (factory → Provider)
//!              ↓                       ↓
//!      RuntimeMetadata          BuildResult {
//!      alias resolution           registry,
//!      platform checks            warnings,
//!                                 errors,
//!                               }
//! ```

pub mod builder;
pub mod index;
pub mod loader;

pub use builder::{BuildError, BuildErrorKind, BuildResult, BuildWarning, ProviderBuilder};
pub use index::{ManifestIndex, ProviderMetadata, RuntimeMetadata};
pub use loader::ManifestStore;
