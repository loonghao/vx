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
//! - If all available versions are non-semver (pure opaque like `"system"`) and
//!   no match is found, fall back to LTS/first entry — the user-specified version
//!   is treated as a documentation hint (e.g. `msvc = "14.42"` in `vx.toml`)

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
    if !matches!(constraint, VersionConstraint::Invalid(_))
        && let Some(resolved) = resolver.resolve_constraint(&constraint, available)
    {
        return Some(resolved);
    }

    // Final fallback for pure opaque providers (e.g. msvc with only "system"):
    // If none of the available versions are parseable as semver, the user-specified
    // version (e.g. "14.42") is treated as a documentation hint and we resolve to
    // the LTS/first available version. This allows `msvc = "14.42"` in vx.toml to
    // work even though the provider only exposes "system".
    let has_any_semver = available
        .iter()
        .any(|v| super::core::Version::parse(&v.version).is_some());

    if !has_any_semver {
        tracing::debug!(
            requested = trimmed,
            "opaque resolver: no semver versions available, treating '{}' as hint and falling back to first available",
            trimmed,
        );
        let lts = available.iter().find(|v| v.lts);
        return lts.or_else(|| available.first()).map(|v| v.version.clone());
    }

    None
}
