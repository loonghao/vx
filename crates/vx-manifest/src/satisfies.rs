//! Version satisfaction checking
//!
//! This module provides version constraint parsing and satisfaction checking
//! using standard semver syntax.

use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::fmt;

/// Semantic version structure
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub prerelease: Option<String>,
}

impl Version {
    /// Create a new version
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
            prerelease: None,
        }
    }

    /// Parse a version string
    pub fn parse(s: &str) -> Option<Self> {
        let s = s.strip_prefix('v').unwrap_or(s);

        // Split off prerelease
        let (version, prerelease) = if let Some(idx) = s.find('-') {
            (&s[..idx], Some(s[idx + 1..].to_string()))
        } else {
            (s, None)
        };

        // Remove build metadata
        let version = if let Some(idx) = version.find('+') {
            &version[..idx]
        } else {
            version
        };

        let parts: Vec<&str> = version.split('.').collect();

        let major = parts.first()?.parse().ok()?;
        let minor = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
        let patch = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(0);

        Some(Self {
            major,
            minor,
            patch,
            prerelease,
        })
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)?;
        if let Some(ref pre) = self.prerelease {
            write!(f, "-{}", pre)?;
        }
        Ok(())
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.major.cmp(&other.major) {
            Ordering::Equal => {}
            ord => return ord,
        }
        match self.minor.cmp(&other.minor) {
            Ordering::Equal => {}
            ord => return ord,
        }
        match self.patch.cmp(&other.patch) {
            Ordering::Equal => {}
            ord => return ord,
        }

        // Prerelease versions have lower precedence
        match (&self.prerelease, &other.prerelease) {
            (None, None) => Ordering::Equal,
            (Some(_), None) => Ordering::Less,
            (None, Some(_)) => Ordering::Greater,
            (Some(a), Some(b)) => a.cmp(b),
        }
    }
}

/// Version constraint types
#[derive(Debug, Clone)]
pub enum VersionConstraint {
    /// Any version
    Any,
    /// Exact version: "1.2.3"
    Exact(Version),
    /// Partial version: "1.2" (matches 1.2.x)
    Partial { major: u32, minor: u32 },
    /// Major version only: "1" (matches 1.x.x)
    Major(u32),
    /// Caret constraint: "^1.2.3" (>=1.2.3, <2.0.0)
    Caret(Version),
    /// Tilde constraint: "~1.2.3" (>=1.2.3, <1.3.0)
    Tilde(Version),
    /// Range constraints: ">=1.2, <2.0"
    Range(Vec<RangeConstraint>),
}

/// Range constraint
#[derive(Debug, Clone)]
pub struct RangeConstraint {
    pub op: RangeOp,
    pub version: Version,
}

impl RangeConstraint {
    pub fn new(op: RangeOp, version: Version) -> Self {
        Self { op, version }
    }

    pub fn satisfies(&self, version: &Version) -> bool {
        match self.op {
            RangeOp::Eq => version == &self.version,
            RangeOp::Ne => version != &self.version,
            RangeOp::Gt => version > &self.version,
            RangeOp::Ge => version >= &self.version,
            RangeOp::Lt => version < &self.version,
            RangeOp::Le => version <= &self.version,
        }
    }
}

/// Range operators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RangeOp {
    Eq,
    Ne,
    Gt,
    Ge,
    Lt,
    Le,
}

/// Version request - parses and checks version constraints
#[derive(Debug, Clone)]
pub struct VersionRequest {
    pub raw: String,
    pub constraint: VersionConstraint,
}

impl VersionRequest {
    /// Parse a version constraint string
    pub fn parse(raw: impl Into<String>) -> Self {
        let raw = raw.into();
        let constraint = Self::parse_constraint(&raw);
        Self { raw, constraint }
    }

    fn parse_constraint(raw: &str) -> VersionConstraint {
        let raw = raw.trim();

        // Handle special keywords
        match raw.to_lowercase().as_str() {
            "*" | "any" | "latest" | "stable" => return VersionConstraint::Any,
            _ => {}
        }

        // Handle caret constraint: ^1.2.3
        if let Some(version_str) = raw.strip_prefix('^') {
            if let Some(version) = Version::parse(version_str) {
                return VersionConstraint::Caret(version);
            }
        }

        // Handle tilde constraint: ~1.2.3 or ~=1.2.3
        if let Some(version_str) = raw.strip_prefix("~=") {
            if let Some(version) = Version::parse(version_str) {
                return VersionConstraint::Tilde(version);
            }
        }
        if let Some(version_str) = raw.strip_prefix('~') {
            if let Some(version) = Version::parse(version_str) {
                return VersionConstraint::Tilde(version);
            }
        }

        // Handle wildcard: 3.11.*
        if let Some(prefix) = raw.strip_suffix(".*") {
            let parts: Vec<&str> = prefix.split('.').collect();
            if parts.len() == 2 {
                if let (Ok(major), Ok(minor)) = (parts[0].parse(), parts[1].parse()) {
                    return VersionConstraint::Partial { major, minor };
                }
            }
        }

        // Handle range constraints: >=3.9,<3.12
        if raw.contains(',')
            || raw.starts_with(">=")
            || raw.starts_with("<=")
            || raw.starts_with('>')
            || raw.starts_with('<')
            || raw.starts_with("!=")
            || raw.starts_with('=')
        {
            let constraints = Self::parse_range_constraints(raw);
            if !constraints.is_empty() {
                return VersionConstraint::Range(constraints);
            }
        }

        // Handle partial versions: 3.11 or 3
        let parts: Vec<&str> = raw.split('.').collect();
        match parts.len() {
            1 => {
                if let Ok(major) = parts[0].parse() {
                    return VersionConstraint::Major(major);
                }
            }
            2 => {
                if let (Ok(major), Ok(minor)) = (parts[0].parse(), parts[1].parse()) {
                    return VersionConstraint::Partial { major, minor };
                }
            }
            _ => {
                if let Some(version) = Version::parse(raw) {
                    return VersionConstraint::Exact(version);
                }
            }
        }

        VersionConstraint::Any
    }

    fn parse_range_constraints(raw: &str) -> Vec<RangeConstraint> {
        let mut constraints = Vec::new();

        for part in raw.split(',') {
            let part = part.trim();
            if part.is_empty() {
                continue;
            }

            if let Some(constraint) = Self::parse_single_range(part) {
                constraints.push(constraint);
            }
        }

        constraints
    }

    fn parse_single_range(s: &str) -> Option<RangeConstraint> {
        let s = s.trim();

        let operators = [
            (">=", RangeOp::Ge),
            ("<=", RangeOp::Le),
            ("!=", RangeOp::Ne),
            (">", RangeOp::Gt),
            ("<", RangeOp::Lt),
            ("=", RangeOp::Eq),
        ];

        for (prefix, op) in operators {
            if let Some(version_str) = s.strip_prefix(prefix) {
                if let Some(version) = Version::parse(version_str.trim()) {
                    return Some(RangeConstraint::new(op, version));
                }
            }
        }

        None
    }

    /// Check if a version string satisfies this constraint
    pub fn satisfies(&self, version: &str) -> bool {
        let v = match Version::parse(version) {
            Some(v) => v,
            None => return false,
        };

        match &self.constraint {
            VersionConstraint::Any => true,
            VersionConstraint::Exact(target) => &v == target,
            VersionConstraint::Partial { major, minor } => v.major == *major && v.minor == *minor,
            VersionConstraint::Major(major) => v.major == *major,
            VersionConstraint::Caret(target) => {
                // ^1.2.3 means >=1.2.3, <2.0.0
                // ^0.2.3 means >=0.2.3, <0.3.0
                // ^0.0.3 means >=0.0.3, <0.0.4
                if v < *target {
                    return false;
                }
                if target.major > 0 {
                    v.major == target.major
                } else if target.minor > 0 {
                    v.major == 0 && v.minor == target.minor
                } else {
                    v.major == 0 && v.minor == 0 && v.patch == target.patch
                }
            }
            VersionConstraint::Tilde(target) => {
                // ~1.2.3 means >=1.2.3, <1.3.0
                v >= *target && v.major == target.major && v.minor == target.minor
            }
            VersionConstraint::Range(constraints) => constraints.iter().all(|c| c.satisfies(&v)),
        }
    }
}

/// Extension trait for version satisfaction checking
pub trait VersionSatisfies {
    /// Check if a version string satisfies this constraint
    fn satisfies(&self, version: &str) -> bool;
}

impl VersionSatisfies for VersionRequest {
    fn satisfies(&self, version: &str) -> bool {
        VersionRequest::satisfies(self, version)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_parse() {
        assert_eq!(Version::parse("1.2.3"), Some(Version::new(1, 2, 3)));
        assert_eq!(Version::parse("v1.2.3"), Some(Version::new(1, 2, 3)));
        assert_eq!(Version::parse("1.2"), Some(Version::new(1, 2, 0)));
        assert_eq!(Version::parse("1"), Some(Version::new(1, 0, 0)));
    }

    #[test]
    fn test_satisfies_exact() {
        let req = VersionRequest::parse("1.2.3");
        assert!(req.satisfies("1.2.3"));
        assert!(!req.satisfies("1.2.4"));
    }

    #[test]
    fn test_satisfies_partial() {
        let req = VersionRequest::parse("1.2");
        assert!(req.satisfies("1.2.0"));
        assert!(req.satisfies("1.2.3"));
        assert!(!req.satisfies("1.3.0"));
    }

    #[test]
    fn test_satisfies_major() {
        let req = VersionRequest::parse("1");
        assert!(req.satisfies("1.0.0"));
        assert!(req.satisfies("1.99.99"));
        assert!(!req.satisfies("2.0.0"));
    }

    #[test]
    fn test_satisfies_caret() {
        let req = VersionRequest::parse("^1.2.3");
        assert!(req.satisfies("1.2.3"));
        assert!(req.satisfies("1.9.0"));
        assert!(!req.satisfies("2.0.0"));
        assert!(!req.satisfies("1.2.2"));
    }

    #[test]
    fn test_satisfies_tilde() {
        let req = VersionRequest::parse("~1.2.3");
        assert!(req.satisfies("1.2.3"));
        assert!(req.satisfies("1.2.99"));
        assert!(!req.satisfies("1.3.0"));
    }

    #[test]
    fn test_satisfies_range() {
        let req = VersionRequest::parse(">=12, <23");
        assert!(req.satisfies("12.0.0"));
        assert!(req.satisfies("20.0.0"));
        assert!(!req.satisfies("11.0.0"));
        assert!(!req.satisfies("23.0.0"));
    }

    #[test]
    fn test_satisfies_any() {
        let req = VersionRequest::parse("*");
        assert!(req.satisfies("1.0.0"));
        assert!(req.satisfies("99.99.99"));
    }
}
