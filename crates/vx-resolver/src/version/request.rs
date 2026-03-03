//! Version request parsing
//!
//! [`VersionRequest`] wraps a raw version string and its parsed [`VersionConstraint`].
//! Constraint parsing is delegated to `vx_versions::resolver::core::parse_constraint`
//! so that all crates share identical parsing semantics.

use super::constraint::{Version, VersionConstraint};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Version request - represents what the user specified in vx.toml
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionRequest {
    /// Original version string (e.g., "3.11", ">=3.9,<3.12", "latest")
    pub raw: String,
    /// Parsed constraint
    pub constraint: VersionConstraint,
}

impl VersionRequest {
    /// Create a new version request from a raw string
    pub fn parse(raw: impl Into<String>) -> Self {
        let raw = raw.into();
        // Delegate to vx-versions for unified parsing semantics across all crates
        let constraint = vx_versions::resolver::core::parse_constraint(&raw);
        Self { raw, constraint }
    }

    /// Create a request for the latest version
    pub fn latest() -> Self {
        Self {
            raw: "latest".to_string(),
            constraint: VersionConstraint::Latest,
        }
    }

    /// Create a request for an exact version
    pub fn exact(version: Version) -> Self {
        Self {
            raw: version.to_string(),
            constraint: VersionConstraint::Exact(version),
        }
    }

    /// Check if this request matches "latest"
    pub fn is_latest(&self) -> bool {
        matches!(
            self.constraint,
            VersionConstraint::Latest | VersionConstraint::LatestPrerelease
        )
    }

    /// Check if this request is for a specific version
    pub fn is_exact(&self) -> bool {
        matches!(self.constraint, VersionConstraint::Exact(_))
    }

    /// Check if this request is a partial version (e.g., "3.11")
    pub fn is_partial(&self) -> bool {
        matches!(
            self.constraint,
            VersionConstraint::Partial { .. } | VersionConstraint::Major(_)
        )
    }
}

impl fmt::Display for VersionRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.raw)
    }
}

impl Default for VersionRequest {
    fn default() -> Self {
        Self::latest()
    }
}

impl From<&str> for VersionRequest {
    fn from(s: &str) -> Self {
        Self::parse(s)
    }
}

impl From<String> for VersionRequest {
    fn from(s: String) -> Self {
        Self::parse(s)
    }
}
