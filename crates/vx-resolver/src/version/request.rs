//! Version request parsing

use super::constraint::{RangeConstraint, RangeOp, Version, VersionConstraint};
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
        let constraint = Self::parse_constraint(&raw);
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

    /// Parse the constraint from a raw string
    fn parse_constraint(raw: &str) -> VersionConstraint {
        let raw = raw.trim();

        // Handle special keywords
        match raw.to_lowercase().as_str() {
            "latest" | "stable" => return VersionConstraint::Latest,
            "latest-prerelease" | "prerelease" | "pre" => {
                return VersionConstraint::LatestPrerelease;
            }
            "*" | "any" => return VersionConstraint::Any,
            _ => {}
        }

        // Handle caret constraint: ^1.2.3
        if let Some(version_str) = raw.strip_prefix('^')
            && let Some(version) = Version::parse(version_str)
        {
            return VersionConstraint::Caret(version);
        }

        // Handle tilde constraint: ~1.2.3 or ~=1.2.3
        if let Some(version_str) = raw.strip_prefix("~=")
            && let Some(version) = Version::parse(version_str)
        {
            return VersionConstraint::Tilde(version);
        }
        if let Some(version_str) = raw.strip_prefix('~')
            && let Some(version) = Version::parse(version_str)
        {
            return VersionConstraint::Tilde(version);
        }

        // Handle wildcard: 3.11.*
        if let Some(prefix) = raw.strip_suffix(".*") {
            let parts: Vec<&str> = prefix.split('.').collect();
            if parts.len() == 2
                && let (Ok(major), Ok(minor)) = (parts[0].parse(), parts[1].parse())
            {
                return VersionConstraint::Wildcard { major, minor };
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
                // Major version only: "3"
                if let Ok(major) = parts[0].parse() {
                    return VersionConstraint::Major(major);
                }
            }
            2 => {
                // Partial version: "3.11"
                if let (Ok(major), Ok(minor)) = (parts[0].parse(), parts[1].parse()) {
                    return VersionConstraint::Partial { major, minor };
                }
            }
            _ => {
                // Exact version: "3.11.11"
                if let Some(version) = Version::parse(raw) {
                    return VersionConstraint::Exact(version);
                }
            }
        }

        // Default to latest if we can't parse
        VersionConstraint::Latest
    }

    /// Parse range constraints from a string like ">=3.9,<3.12"
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

    /// Parse a single range constraint like ">=3.9"
    fn parse_single_range(s: &str) -> Option<RangeConstraint> {
        let s = s.trim();

        // Try each operator from longest to shortest
        let operators = [
            (">=", RangeOp::Ge),
            ("<=", RangeOp::Le),
            ("!=", RangeOp::Ne),
            ("~=", RangeOp::Tilde),
            (">", RangeOp::Gt),
            ("<", RangeOp::Lt),
            ("=", RangeOp::Eq),
            ("^", RangeOp::Caret),
            ("~", RangeOp::Tilde),
        ];

        for (prefix, op) in operators {
            if let Some(version_str) = s.strip_prefix(prefix)
                && let Some(version) = Version::parse(version_str.trim())
            {
                return Some(RangeConstraint::new(op, version));
            }
        }

        None
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_latest() {
        let req = VersionRequest::parse("latest");
        assert!(matches!(req.constraint, VersionConstraint::Latest));

        let req = VersionRequest::parse("stable");
        assert!(matches!(req.constraint, VersionConstraint::Latest));
    }

    #[test]
    fn test_parse_exact() {
        let req = VersionRequest::parse("3.11.11");
        assert!(matches!(req.constraint, VersionConstraint::Exact(_)));
        if let VersionConstraint::Exact(v) = req.constraint {
            assert_eq!(v.major, 3);
            assert_eq!(v.minor, 11);
            assert_eq!(v.patch, 11);
        }
    }

    #[test]
    fn test_parse_partial() {
        let req = VersionRequest::parse("3.11");
        assert!(matches!(
            req.constraint,
            VersionConstraint::Partial {
                major: 3,
                minor: 11
            }
        ));

        let req = VersionRequest::parse("3");
        assert!(matches!(req.constraint, VersionConstraint::Major(3)));
    }

    #[test]
    fn test_parse_caret() {
        let req = VersionRequest::parse("^1.2.3");
        assert!(matches!(req.constraint, VersionConstraint::Caret(_)));
    }

    #[test]
    fn test_parse_tilde() {
        let req = VersionRequest::parse("~1.2.3");
        assert!(matches!(req.constraint, VersionConstraint::Tilde(_)));

        let req = VersionRequest::parse("~=1.2.3");
        assert!(matches!(req.constraint, VersionConstraint::Tilde(_)));
    }

    #[test]
    fn test_parse_wildcard() {
        let req = VersionRequest::parse("3.11.*");
        assert!(matches!(
            req.constraint,
            VersionConstraint::Wildcard {
                major: 3,
                minor: 11
            }
        ));
    }

    #[test]
    fn test_parse_range() {
        let req = VersionRequest::parse(">=3.9,<3.12");
        if let VersionConstraint::Range(constraints) = req.constraint {
            assert_eq!(constraints.len(), 2);
            assert!(matches!(constraints[0].op, RangeOp::Ge));
            assert!(matches!(constraints[1].op, RangeOp::Lt));
        } else {
            panic!("Expected Range constraint");
        }
    }
}
