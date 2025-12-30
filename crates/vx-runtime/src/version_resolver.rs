//! Version resolver for Runtime trait
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

use crate::{Ecosystem, VersionInfo};
use std::cmp::Ordering;

/// Parsed semantic version
#[derive(Debug, Clone, PartialEq, Eq)]
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
        // Handle Go-style versions (go1.22.0 -> 1.22.0)
        let s = s.strip_prefix("go").unwrap_or(s);
        // Handle v prefix
        let s = s.strip_prefix('v').unwrap_or(s);

        let (version_part, prerelease) = if let Some(idx) = s.find('-') {
            (&s[..idx], Some(s[idx + 1..].to_string()))
        } else {
            (s, None)
        };

        let parts: Vec<&str> = version_part.split('.').collect();
        if parts.is_empty() || parts.len() > 3 {
            return None;
        }

        let major = parts[0].parse().ok()?;
        let minor = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
        let patch = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(0);

        Some(Self {
            major,
            minor,
            patch,
            prerelease,
        })
    }

    /// Check if this is a prerelease version
    pub fn is_prerelease(&self) -> bool {
        self.prerelease.is_some()
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
            Ordering::Equal => match self.minor.cmp(&other.minor) {
                Ordering::Equal => match self.patch.cmp(&other.patch) {
                    Ordering::Equal => {
                        // Prereleases are less than releases
                        match (&self.prerelease, &other.prerelease) {
                            (None, None) => Ordering::Equal,
                            (Some(_), None) => Ordering::Less,
                            (None, Some(_)) => Ordering::Greater,
                            (Some(a), Some(b)) => a.cmp(b),
                        }
                    }
                    other => other,
                },
                other => other,
            },
            other => other,
        }
    }
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)?;
        if let Some(ref pre) = self.prerelease {
            write!(f, "-{}", pre)?;
        }
        Ok(())
    }
}

/// Version constraint types
#[derive(Debug, Clone)]
pub enum VersionConstraint {
    /// Exact version: "3.11.11"
    Exact(Version),
    /// Partial version: "3.11" (matches latest 3.11.x)
    Partial { major: u32, minor: u32 },
    /// Major version only: "20" (matches latest 20.x.x)
    Major(u32),
    /// Latest stable version
    Latest,
    /// Range constraints: ">=3.9,<3.12"
    Range(Vec<RangeConstraint>),
    /// Wildcard: "3.11.*"
    Wildcard { major: u32, minor: u32 },
    /// Caret constraint: "^1.2.3" (>=1.2.3, <2.0.0)
    Caret(Version),
    /// Tilde constraint: "~1.2.3" (>=1.2.3, <1.3.0)
    Tilde(Version),
    /// Any version
    Any,
    /// Invalid version string
    Invalid(String),
}

/// Range operator
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RangeOp {
    /// Greater than or equal
    Gte,
    /// Greater than
    Gt,
    /// Less than or equal
    Lte,
    /// Less than
    Lt,
    /// Equal
    Eq,
    /// Not equal
    Ne,
}

/// A single range constraint
#[derive(Debug, Clone)]
pub struct RangeConstraint {
    pub op: RangeOp,
    pub version: Version,
}

impl RangeConstraint {
    /// Check if a version satisfies this constraint
    pub fn satisfies(&self, version: &Version) -> bool {
        match self.op {
            RangeOp::Gte => version >= &self.version,
            RangeOp::Gt => version > &self.version,
            RangeOp::Lte => version <= &self.version,
            RangeOp::Lt => version < &self.version,
            RangeOp::Eq => version == &self.version,
            RangeOp::Ne => version != &self.version,
        }
    }
}

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

    /// Parse a version request string into a constraint
    pub fn parse_constraint(&self, version_str: &str) -> VersionConstraint {
        let trimmed = version_str.trim();

        // Handle special cases
        if trimmed.is_empty() || trimmed == "latest" {
            return VersionConstraint::Latest;
        }

        if trimmed == "*" {
            return VersionConstraint::Any;
        }

        // Handle caret constraint: ^1.0.0
        if let Some(rest) = trimmed.strip_prefix('^') {
            if let Some(v) = Version::parse(rest) {
                return VersionConstraint::Caret(v);
            }
        }

        // Handle tilde constraint: ~1.0.0
        if let Some(rest) = trimmed.strip_prefix('~') {
            if let Some(v) = Version::parse(rest) {
                return VersionConstraint::Tilde(v);
            }
        }

        // Handle range constraints: >=3.9,<3.12
        if trimmed.contains(">=")
            || trimmed.contains("<=")
            || trimmed.contains('>')
            || trimmed.contains('<')
            || trimmed.contains("!=")
        {
            let constraints = self.parse_range_constraints(trimmed);
            if !constraints.is_empty() {
                return VersionConstraint::Range(constraints);
            }
        }

        // Handle wildcard: 3.11.*
        if let Some(prefix) = trimmed.strip_suffix(".*") {
            let parts: Vec<&str> = prefix.split('.').collect();
            if parts.len() == 2 {
                if let (Ok(major), Ok(minor)) = (parts[0].parse(), parts[1].parse()) {
                    return VersionConstraint::Wildcard { major, minor };
                }
            }
        }

        // Try to parse as exact version
        if let Some(v) = Version::parse(trimmed) {
            // Check if it's a partial version (only major.minor)
            let parts: Vec<&str> = trimmed
                .strip_prefix("go")
                .unwrap_or(trimmed)
                .strip_prefix('v')
                .unwrap_or(trimmed)
                .split('.')
                .collect();

            if parts.len() == 2 {
                return VersionConstraint::Partial {
                    major: v.major,
                    minor: v.minor,
                };
            } else if parts.len() == 1 {
                return VersionConstraint::Major(v.major);
            }

            return VersionConstraint::Exact(v);
        }

        // Try to parse as major version only
        if let Ok(major) = trimmed.parse::<u32>() {
            return VersionConstraint::Major(major);
        }

        // Invalid version string
        VersionConstraint::Invalid(trimmed.to_string())
    }

    /// Parse range constraints from a string
    fn parse_range_constraints(&self, s: &str) -> Vec<RangeConstraint> {
        let mut constraints = Vec::new();

        // Split by comma or space
        for part in s.split([',', ' ']).filter(|p| !p.is_empty()) {
            let part = part.trim();

            let (op, version_str) = if let Some(rest) = part.strip_prefix(">=") {
                (RangeOp::Gte, rest)
            } else if let Some(rest) = part.strip_prefix("<=") {
                (RangeOp::Lte, rest)
            } else if let Some(rest) = part.strip_prefix("!=") {
                (RangeOp::Ne, rest)
            } else if let Some(rest) = part.strip_prefix('>') {
                (RangeOp::Gt, rest)
            } else if let Some(rest) = part.strip_prefix('<') {
                (RangeOp::Lt, rest)
            } else if let Some(rest) = part.strip_prefix("==") {
                (RangeOp::Eq, rest)
            } else if let Some(rest) = part.strip_prefix('=') {
                (RangeOp::Eq, rest)
            } else {
                continue;
            };

            if let Some(version) = Version::parse(version_str.trim()) {
                constraints.push(RangeConstraint { op, version });
            }
        }

        constraints
    }

    /// Resolve a version string against available versions
    pub fn resolve(
        &self,
        version_str: &str,
        available: &[VersionInfo],
        _ecosystem: &Ecosystem,
    ) -> Option<String> {
        let constraint = self.parse_constraint(version_str);
        self.resolve_constraint(&constraint, available)
    }

    /// Resolve a constraint against available versions
    pub fn resolve_constraint(
        &self,
        constraint: &VersionConstraint,
        available: &[VersionInfo],
    ) -> Option<String> {
        // Invalid constraints never match
        if matches!(constraint, VersionConstraint::Invalid(_)) {
            return None;
        }

        // Parse and filter versions
        let mut versions: Vec<(Version, &VersionInfo)> = available
            .iter()
            .filter_map(|v| {
                let parsed = Version::parse(&v.version)?;
                // Filter prereleases unless allowed
                if !self.allow_prerelease && (parsed.is_prerelease() || v.prerelease) {
                    return None;
                }
                Some((parsed, v))
            })
            .collect();

        // Sort by version descending (newest first)
        versions.sort_by(|a, b| b.0.cmp(&a.0));

        // For Latest constraint with prefer_lts, prefer LTS versions
        if matches!(constraint, VersionConstraint::Latest) && self.prefer_lts {
            let lts_versions: Vec<_> = versions.iter().filter(|(_, v)| v.lts).collect();
            if !lts_versions.is_empty() {
                return lts_versions.first().map(|(_, v)| v.version.clone());
            }
        }

        // Find matching version
        for (parsed, info) in &versions {
            if self.version_satisfies(parsed, constraint) {
                return Some(info.version.clone());
            }
        }

        None
    }

    /// Check if a version satisfies a constraint
    fn version_satisfies(&self, version: &Version, constraint: &VersionConstraint) -> bool {
        match constraint {
            VersionConstraint::Exact(target) => version == target,

            VersionConstraint::Partial { major, minor } => {
                version.major == *major && version.minor == *minor
            }

            VersionConstraint::Major(major) => version.major == *major,

            VersionConstraint::Latest | VersionConstraint::Any => true,

            VersionConstraint::Range(constraints) => {
                constraints.iter().all(|c| c.satisfies(version))
            }

            VersionConstraint::Wildcard { major, minor } => {
                version.major == *major && version.minor == *minor
            }

            VersionConstraint::Caret(target) => {
                // ^1.2.3 means >=1.2.3 <2.0.0
                // ^0.2.3 means >=0.2.3 <0.3.0
                // ^0.0.3 means >=0.0.3 <0.0.4
                if version < target {
                    return false;
                }

                if target.major > 0 {
                    version.major == target.major
                } else if target.minor > 0 {
                    version.major == 0 && version.minor == target.minor
                } else {
                    version.major == 0 && version.minor == 0 && version.patch == target.patch
                }
            }

            VersionConstraint::Tilde(target) => {
                // ~1.2.3 means >=1.2.3 <1.3.0
                if version < target {
                    return false;
                }
                version.major == target.major && version.minor == target.minor
            }

            VersionConstraint::Invalid(_) => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_versions(versions: &[&str]) -> Vec<VersionInfo> {
        versions.iter().map(|v| VersionInfo::new(*v)).collect()
    }

    #[test]
    fn test_version_parse() {
        let v = Version::parse("1.2.3").unwrap();
        assert_eq!(v.major, 1);
        assert_eq!(v.minor, 2);
        assert_eq!(v.patch, 3);

        let v = Version::parse("v1.2.3").unwrap();
        assert_eq!(v.major, 1);

        let v = Version::parse("go1.22.0").unwrap();
        assert_eq!(v.major, 1);
        assert_eq!(v.minor, 22);
    }

    #[test]
    fn test_resolve_latest() {
        let resolver = VersionResolver::new();
        let available = make_versions(&["1.0.0", "1.1.0", "2.0.0"]);

        let result = resolver.resolve("latest", &available, &Ecosystem::NodeJs);
        assert_eq!(result, Some("2.0.0".to_string()));
    }

    #[test]
    fn test_resolve_partial() {
        let resolver = VersionResolver::new();
        let available = make_versions(&["3.10.0", "3.11.0", "3.11.5", "3.11.11", "3.12.0"]);

        let result = resolver.resolve("3.11", &available, &Ecosystem::Python);
        assert_eq!(result, Some("3.11.11".to_string()));
    }

    #[test]
    fn test_resolve_major() {
        let resolver = VersionResolver::new();
        let available = make_versions(&["18.0.0", "20.0.0", "20.10.0", "22.0.0"]);

        let result = resolver.resolve("20", &available, &Ecosystem::NodeJs);
        assert_eq!(result, Some("20.10.0".to_string()));
    }

    #[test]
    fn test_resolve_exact() {
        let resolver = VersionResolver::new();
        let available = make_versions(&["1.0.0", "1.1.0", "2.0.0"]);

        let result = resolver.resolve("1.1.0", &available, &Ecosystem::NodeJs);
        assert_eq!(result, Some("1.1.0".to_string()));
    }

    #[test]
    fn test_resolve_range() {
        let resolver = VersionResolver::new();
        let available = make_versions(&["3.8.0", "3.9.0", "3.10.0", "3.11.0", "3.12.0"]);

        let result = resolver.resolve(">=3.9,<3.12", &available, &Ecosystem::Python);
        assert_eq!(result, Some("3.11.0".to_string()));
    }

    #[test]
    fn test_resolve_caret() {
        let resolver = VersionResolver::new();
        let available = make_versions(&["1.0.0", "1.1.0", "1.9.0", "2.0.0"]);

        let result = resolver.resolve("^1.0.0", &available, &Ecosystem::NodeJs);
        assert_eq!(result, Some("1.9.0".to_string()));
    }

    #[test]
    fn test_resolve_tilde() {
        let resolver = VersionResolver::new();
        let available = make_versions(&["1.0.0", "1.0.5", "1.1.0", "2.0.0"]);

        let result = resolver.resolve("~1.0.0", &available, &Ecosystem::NodeJs);
        assert_eq!(result, Some("1.0.5".to_string()));
    }

    #[test]
    fn test_resolve_wildcard() {
        let resolver = VersionResolver::new();
        let available = make_versions(&["3.10.0", "3.11.0", "3.11.5", "3.11.11", "3.12.0"]);

        let result = resolver.resolve("3.11.*", &available, &Ecosystem::Python);
        assert_eq!(result, Some("3.11.11".to_string()));
    }

    #[test]
    fn test_resolve_not_found() {
        let resolver = VersionResolver::new();
        let available = make_versions(&["1.0.0", "1.1.0"]);

        let result = resolver.resolve("2.0.0", &available, &Ecosystem::NodeJs);
        assert_eq!(result, None);
    }

    #[test]
    fn test_prerelease_filtering() {
        let resolver = VersionResolver::new();
        let mut available = make_versions(&["1.0.0", "2.0.0-beta.1", "2.0.0"]);
        available[1].prerelease = true;

        let result = resolver.resolve("latest", &available, &Ecosystem::NodeJs);
        assert_eq!(result, Some("2.0.0".to_string()));
    }

    #[test]
    fn test_lts_preference() {
        let resolver = VersionResolver::new();
        let mut available = make_versions(&["18.0.0", "20.0.0", "22.0.0"]);
        available[1].lts = true; // 20.0.0 is LTS

        let result = resolver.resolve("latest", &available, &Ecosystem::NodeJs);
        assert_eq!(result, Some("20.0.0".to_string()));
    }

    #[test]
    fn test_invalid_version_strings() {
        let resolver = VersionResolver::new();
        let available = make_versions(&["1.0.0", "2.0.0", "3.0.0"]);

        // These should all return None (invalid versions)
        assert_eq!(
            resolver.resolve("not-a-version", &available, &Ecosystem::NodeJs),
            None
        );
        assert_eq!(resolver.resolve("v", &available, &Ecosystem::NodeJs), None);
        assert_eq!(resolver.resolve("@", &available, &Ecosystem::NodeJs), None);
        assert_eq!(
            resolver.resolve("abc123", &available, &Ecosystem::NodeJs),
            None
        );
        assert_eq!(
            resolver.resolve("1.2.3.4.5", &available, &Ecosystem::NodeJs),
            None
        );
    }

    #[test]
    fn test_parse_invalid_constraint() {
        let resolver = VersionResolver::new();

        // These should be parsed as Invalid
        assert!(matches!(
            resolver.parse_constraint("not-a-version"),
            VersionConstraint::Invalid(_)
        ));
        assert!(matches!(
            resolver.parse_constraint("v"),
            VersionConstraint::Invalid(_)
        ));
        assert!(matches!(
            resolver.parse_constraint("@"),
            VersionConstraint::Invalid(_)
        ));
    }
}
