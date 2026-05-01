//! Version resolver
//!
//! Defines the version resolution traits and types.
//!
//! This module provides version resolution logic that supports:
//! - Exact versions: "3.11.11"
//! - Partial versions: "3.11" (matches latest 3.11.x)
//! - Major versions: "20" (matches latest 20.x.x)
//! - Latest: "latest"
//! - Range constraints: ">=3.9,<3.12"
//! - Caret constraints: "^1.0.0"
//! - Tilde constraints: "~1.0.0"
//! - Wildcards: "3.11.*"
//!
//! # Ecosystem-aware resolution
//!
//! Each ecosystem has its own version parsing rules:
//! - **NodeJs**: `lts`, `lts/iron`, `v20` prefix stripping
//! - **Python**: PEP 440 `~=` compatible release
//! - **Go**: `go` prefix stripping
//! - **Rust**: `stable`, `beta`, `nightly` aliases
//! - **System/opaque**: non-numeric versions (e.g. `system`)
//! - **Generic**: standard semver

pub mod core;
mod nodejs;
mod opaque;
mod python;
mod rust_eco;

// Re-export all public types from core
pub use core::{RangeConstraint, RangeOp, Version, VersionConstraint, VersionRequest};

use crate::{Ecosystem, VersionInfo};

/// Version resolver
pub struct VersionResolver {
    /// Whether to prefer LTS versions
    pub prefer_lts: bool,
    /// Whether to allow prerelease versions
    pub allow_prerelease: bool,
}

impl Default for VersionResolver {
    fn default() -> Self {
        Self {
            prefer_lts: true,
            allow_prerelease: false,
        }
    }
}

impl VersionResolver {
    /// Create a new resolver
    pub fn new() -> Self {
        Self::default()
    }

    /// Parse a version request string into a constraint, ecosystem-aware
    pub fn parse_constraint(&self, version_str: &str) -> VersionConstraint {
        core::parse_constraint(version_str)
    }

    /// Resolve a version string against available versions, ecosystem-aware
    pub fn resolve(
        &self,
        version_str: &str,
        available: &[VersionInfo],
        ecosystem: &Ecosystem,
    ) -> Option<String> {
        match ecosystem {
            Ecosystem::NodeJs => nodejs::resolve(self, version_str, available),
            Ecosystem::Python => python::resolve(self, version_str, available),
            Ecosystem::Rust => rust_eco::resolve(self, version_str, available),
            Ecosystem::System => opaque::resolve(self, version_str, available),
            _ => {
                // Generic semver resolution for Go, Java, Dotnet, Git, Generic, etc.
                let constraint = core::parse_constraint(version_str);
                self.resolve_constraint(&constraint, available)
            }
        }
    }

    /// Resolve a constraint against available versions
    pub fn resolve_constraint(
        &self,
        constraint: &VersionConstraint,
        available: &[VersionInfo],
    ) -> Option<String> {
        core::resolve_constraint(self, constraint, available)
    }
}
