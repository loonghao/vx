//! Rust ecosystem version resolution
//!
//! Special handling:
//! - `stable` → latest stable version (same as `latest`)
//! - `beta` → latest beta/prerelease version
//! - `nightly` → latest nightly/prerelease version
//! - `1.75.0` → exact version (standard semver)

use super::VersionResolver;
use super::core::{self, VersionConstraint};
use crate::VersionInfo;

/// Resolve a version string for the Rust ecosystem.
pub fn resolve(
    resolver: &VersionResolver,
    version_str: &str,
    available: &[VersionInfo],
) -> Option<String> {
    let trimmed = version_str.trim();

    let lower = trimmed.to_lowercase();
    let constraint = match lower.as_str() {
        "stable" => VersionConstraint::Latest,
        "beta" | "nightly" => VersionConstraint::LatestPrerelease,
        _ => core::parse_constraint(trimmed),
    };

    resolver.resolve_constraint(&constraint, available)
}
