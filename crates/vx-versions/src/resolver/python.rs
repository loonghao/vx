//! Python ecosystem version resolution
//!
//! Special handling (PEP 440):
//! - `~=3.11` → compatible release: `>=3.11, <4.0` (2-part: major compatible)
//! - `~=3.11.0` → compatible release: `>=3.11.0, <3.12` (3-part: minor compatible)
//! - `>=3.9,<3.12` → range constraints (standard)
//! - `3.11` → partial version (latest 3.11.x)
//!
//! The `~=` (compatible release) operator is Python-specific and is already
//! handled by `core::parse_constraint` via `VersionConstraint::CompatibleRelease`.
//! This module simply delegates to core after any Python-specific normalization.

use super::VersionResolver;
use super::core;
use crate::VersionInfo;

/// Resolve a version string for the Python ecosystem.
pub fn resolve(
    resolver: &VersionResolver,
    version_str: &str,
    available: &[VersionInfo],
) -> Option<String> {
    let trimmed = version_str.trim();

    // Python-specific aliases
    let lower = trimmed.to_lowercase();
    let effective = match lower.as_str() {
        "python" | "python3" => "latest",
        _ => trimmed,
    };

    let constraint = core::parse_constraint(effective);
    resolver.resolve_constraint(&constraint, available)
}
