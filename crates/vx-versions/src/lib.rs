//! VX Versions - Core version types and resolution logic
//!
//! This crate provides the shared version domain types used across vx:
//!
//! - [`VersionInfo`]: Version metadata (version string, prerelease, LTS, etc.)
//! - [`Version`]: Parsed semantic version with comparison support
//! - [`VersionConstraint`]: Version constraint types (exact, range, caret, tilde, etc.)
//! - [`VersionResolver`]: Resolves version strings against available versions
//! - [`VersionCache`]: High-performance bincode-based version cache
//!
//! # Dependency Direction
//!
//! ```text
//! vx-version-fetcher ──┐
//!                       ├──> vx-versions (this crate)
//! vx-runtime ──────────┘
//! ```
//!
//! This crate has no dependency on `vx-runtime` or `vx-version-fetcher`,
//! breaking the previous circular dependency.

pub mod cache;
pub mod ecosystem;
pub mod fetch_context;
pub mod info;
pub mod resolver;
// resolver/ directory contains: mod.rs, core.rs, nodejs.rs, python.rs, rust_eco.rs, opaque.rs

// Re-exports
pub use cache::{
    CACHE_SCHEMA_VERSION, CacheData, CacheEntry, CacheMetadata, CacheMode, CacheStats,
    CompactVersion, DEFAULT_CACHE_TTL, VersionCache, github_release_to_compact,
};
pub use ecosystem::Ecosystem;
pub use fetch_context::FetchContext;
pub use info::VersionInfo;
pub use resolver::{
    RangeConstraint, RangeOp, Version, VersionConstraint, VersionRequest, VersionResolver,
};
// Export parse_constraint for use by other crates (e.g. vx-resolver)
pub use resolver::core::parse_constraint;
