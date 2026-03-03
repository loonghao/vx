//! Opaque / System ecosystem version resolution
//!
//! For providers like `msvc`, `git` (Windows), `system tools` that expose
//! non-numeric version strings (e.g. `"system"`, `"latest"`), we skip semver
//! parsing entirely and do direct string matching or return the first entry.
//!
//! Rules:
//! - If `version_str` is `"latest"` / `"*"` / `"any"` → return first (LTS-preferred) entry
//! - If `version_str` exactly matches an available version → return it
//! - Otherwise fall back to generic semver resolution (for mixed lists)

use super::VersionResolver;
use super::core::{self, VersionConstraint};
use crate::VersionInfo;

/// Resolve a version string for the System/opaque ecosystem.
pub fn resolve(
    resolver: &VersionResolver,
    version_str: &str,
    available: &[VersionInfo],
) -> Option<String> {
    let trimmed = version_str.trim();

    // For "latest" / "*" / "any" → prefer LTS, then first entry
    let lower = trimmed.to_lowercase();
    if matches!(lower.as_str(), "latest" | "*" | "any" | "stable") {
        let lts = available.iter().find(|v| v.lts);
        return lts.or_else(|| available.first()).map(|v| v.version.clone());
    }

    // Exact string match (e.g. "system" matches "system")
    if let Some(found) = available.iter().find(|v| v.version == trimmed) {
        return Some(found.version.clone());
    }

    // Case-insensitive match
    let lower = trimmed.to_lowercase();
    if let Some(found) = available.iter().find(|v| v.version.to_lowercase() == lower) {
        return Some(found.version.clone());
    }

    // Fall back to generic semver resolution for mixed lists
    let constraint = core::parse_constraint(trimmed);
    if !matches!(constraint, VersionConstraint::Invalid(_)) {
        return resolver.resolve_constraint(&constraint, available);
    }

    None
}
