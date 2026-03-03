//! Node.js ecosystem version resolution
//!
//! Special handling:
//! - `lts` → prefer LTS versions (same as `latest` with `prefer_lts=true`)
//! - `lts/iron`, `lts/hydrogen` → LTS codename aliases (treated as `lts`)
//! - `v20`, `v20.0.0` → strip `v` prefix (handled by `Version::parse`)
//! - `20` → major version (latest 20.x.x)

use super::VersionResolver;
use super::core;
use crate::VersionInfo;

/// Resolve a version string for the Node.js ecosystem.
pub fn resolve(
    resolver: &VersionResolver,
    version_str: &str,
    available: &[VersionInfo],
) -> Option<String> {
    let trimmed = version_str.trim();

    // Handle lts codename aliases: "lts/iron", "lts/hydrogen", etc.
    // Treat them all as "lts" (prefer LTS versions).
    let lower = trimmed.to_lowercase();
    let effective = if lower.starts_with("lts/") {
        "lts"
    } else {
        trimmed
    };

    let constraint = core::parse_constraint(effective);
    resolver.resolve_constraint(&constraint, available)
}
