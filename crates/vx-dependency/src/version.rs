//! Version handling and semantic version matching

use crate::{Error, Result};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::fmt;

/// Semantic version representation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Version {
    /// Major version
    pub major: u64,
    /// Minor version
    pub minor: u64,
    /// Patch version
    pub patch: u64,
    /// Pre-release identifier
    pub prerelease: Option<String>,
    /// Build metadata
    pub build: Option<String>,
}

/// Version range specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionRange {
    /// Range expression (e.g., ">=1.0.0", "^2.1.0", "~1.2.3")
    pub expression: String,
    /// Parsed range components
    pub components: Vec<VersionRangeComponent>,
}

/// Component of a version range
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionRangeComponent {
    /// Operator (>=, >, <=, <, =, ^, ~)
    pub operator: VersionOperator,
    /// Target version
    pub version: Version,
}

/// Version comparison operators
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum VersionOperator {
    /// Exact match (=)
    Equal,
    /// Greater than (>)
    GreaterThan,
    /// Greater than or equal (>=)
    GreaterThanOrEqual,
    /// Less than (<)
    LessThan,
    /// Less than or equal (<=)
    LessThanOrEqual,
    /// Compatible release (^) - allows patch and minor updates
    Caret,
    /// Tilde range (~) - allows patch updates only
    Tilde,
}

/// Version matcher for checking constraints
pub struct VersionMatcher;

impl Version {
    /// Parse a version string
    pub fn parse(version_str: &str) -> Result<Self> {
        let version_str = version_str.trim();

        // Handle 'v' prefix
        let version_str = version_str.strip_prefix('v').unwrap_or(version_str);

        // Split by '+' to separate build metadata
        let (version_part, build) = if let Some(pos) = version_str.find('+') {
            let (v, b) = version_str.split_at(pos);
            (v, Some(b[1..].to_string()))
        } else {
            (version_str, None)
        };

        // Split by '-' to separate prerelease
        let (core_version, prerelease) = if let Some(pos) = version_part.find('-') {
            let (v, p) = version_part.split_at(pos);
            (v, Some(p[1..].to_string()))
        } else {
            (version_part, None)
        };

        // Parse major.minor.patch
        let parts: Vec<&str> = core_version.split('.').collect();
        if parts.len() < 2 {
            return Err(Error::InvalidVersionConstraint {
                constraint: version_str.to_string(),
            });
        }

        let major = parts[0]
            .parse()
            .map_err(|_| Error::InvalidVersionConstraint {
                constraint: version_str.to_string(),
            })?;

        let minor = parts[1]
            .parse()
            .map_err(|_| Error::InvalidVersionConstraint {
                constraint: version_str.to_string(),
            })?;

        let patch = if parts.len() > 2 {
            parts[2]
                .parse()
                .map_err(|_| Error::InvalidVersionConstraint {
                    constraint: version_str.to_string(),
                })?
        } else {
            0
        };

        Ok(Version {
            major,
            minor,
            patch,
            prerelease,
            build,
        })
    }

    /// Check if this is a prerelease version
    pub fn is_prerelease(&self) -> bool {
        self.prerelease.is_some()
    }

    /// Get the core version without prerelease/build metadata
    pub fn core_version(&self) -> String {
        format!("{}.{}.{}", self.major, self.minor, self.patch)
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)?;

        if let Some(ref prerelease) = self.prerelease {
            write!(f, "-{}", prerelease)?;
        }

        if let Some(ref build) = self.build {
            write!(f, "+{}", build)?;
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
        // Compare major.minor.patch
        match self.major.cmp(&other.major) {
            Ordering::Equal => {}
            other => return other,
        }

        match self.minor.cmp(&other.minor) {
            Ordering::Equal => {}
            other => return other,
        }

        match self.patch.cmp(&other.patch) {
            Ordering::Equal => {}
            other => return other,
        }

        // Handle prerelease comparison
        match (&self.prerelease, &other.prerelease) {
            (None, None) => Ordering::Equal,
            (Some(_), None) => Ordering::Less, // Prerelease < release
            (None, Some(_)) => Ordering::Greater, // Release > prerelease
            (Some(a), Some(b)) => a.cmp(b),    // Compare prerelease strings
        }
    }
}

impl VersionRange {
    /// Parse a version range expression
    pub fn parse(expression: &str) -> Result<Self> {
        let expression = expression.trim();
        let components = Self::parse_components(expression)?;

        Ok(VersionRange {
            expression: expression.to_string(),
            components,
        })
    }

    /// Check if a version satisfies this range
    pub fn satisfies(&self, version: &Version) -> bool {
        self.components
            .iter()
            .all(|component| component.satisfies(version))
    }

    fn parse_components(expression: &str) -> Result<Vec<VersionRangeComponent>> {
        // Simple implementation - can be extended for complex ranges
        let mut components = Vec::new();

        // Handle single constraint for now
        let (operator, version_str) = if expression.starts_with(">=") {
            (VersionOperator::GreaterThanOrEqual, &expression[2..])
        } else if expression.starts_with("<=") {
            (VersionOperator::LessThanOrEqual, &expression[2..])
        } else if expression.starts_with('>') {
            (VersionOperator::GreaterThan, &expression[1..])
        } else if expression.starts_with('<') {
            (VersionOperator::LessThan, &expression[1..])
        } else if expression.starts_with('^') {
            (VersionOperator::Caret, &expression[1..])
        } else if expression.starts_with('~') {
            (VersionOperator::Tilde, &expression[1..])
        } else if expression.starts_with('=') {
            (VersionOperator::Equal, &expression[1..])
        } else {
            (VersionOperator::Equal, expression)
        };

        let version = Version::parse(version_str.trim())?;
        components.push(VersionRangeComponent { operator, version });

        Ok(components)
    }
}

impl VersionRangeComponent {
    /// Check if a version satisfies this component
    pub fn satisfies(&self, version: &Version) -> bool {
        match self.operator {
            VersionOperator::Equal => version == &self.version,
            VersionOperator::GreaterThan => version > &self.version,
            VersionOperator::GreaterThanOrEqual => version >= &self.version,
            VersionOperator::LessThan => version < &self.version,
            VersionOperator::LessThanOrEqual => version <= &self.version,
            VersionOperator::Caret => self.satisfies_caret(version),
            VersionOperator::Tilde => self.satisfies_tilde(version),
        }
    }

    fn satisfies_caret(&self, version: &Version) -> bool {
        // ^1.2.3 := >=1.2.3 <2.0.0 (compatible release)
        if version < &self.version {
            return false;
        }

        if self.version.major == 0 {
            // ^0.2.3 := >=0.2.3 <0.3.0
            version.major == self.version.major && version.minor == self.version.minor
        } else {
            // ^1.2.3 := >=1.2.3 <2.0.0
            version.major == self.version.major
        }
    }

    fn satisfies_tilde(&self, version: &Version) -> bool {
        // ~1.2.3 := >=1.2.3 <1.3.0 (patch updates only)
        version >= &self.version
            && version.major == self.version.major
            && version.minor == self.version.minor
    }
}

impl VersionMatcher {
    /// Check if a version satisfies a constraint expression
    pub fn matches(version_str: &str, constraint: &str) -> Result<bool> {
        let version = Version::parse(version_str)?;
        let range = VersionRange::parse(constraint)?;
        Ok(range.satisfies(&version))
    }

    /// Find the best matching version from a list
    pub fn find_best_match(versions: &[String], constraint: &str) -> Result<Option<String>> {
        let range = VersionRange::parse(constraint)?;
        let mut matching_versions = Vec::new();

        for version_str in versions {
            if let Ok(version) = Version::parse(version_str) {
                if range.satisfies(&version) {
                    matching_versions.push((version, version_str.clone()));
                }
            }
        }

        // Sort by version (highest first) and return the best match
        matching_versions.sort_by(|a, b| b.0.cmp(&a.0));
        Ok(matching_versions
            .first()
            .map(|(_, version_str)| version_str.clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_parsing() {
        let v = Version::parse("1.2.3").unwrap();
        assert_eq!(v.major, 1);
        assert_eq!(v.minor, 2);
        assert_eq!(v.patch, 3);
        assert!(v.prerelease.is_none());

        let v_pre = Version::parse("2.0.0-beta.1").unwrap();
        assert_eq!(v_pre.major, 2);
        assert_eq!(v_pre.prerelease, Some("beta.1".to_string()));
        assert!(v_pre.is_prerelease());

        let v_build = Version::parse("1.0.0+build.123").unwrap();
        assert_eq!(v_build.build, Some("build.123".to_string()));
    }

    #[test]
    fn test_version_comparison() {
        let v1 = Version::parse("1.0.0").unwrap();
        let v2 = Version::parse("1.0.1").unwrap();
        let v3 = Version::parse("1.1.0").unwrap();
        let v_pre = Version::parse("1.0.0-beta").unwrap();

        assert!(v1 < v2);
        assert!(v2 < v3);
        assert!(v_pre < v1); // Prerelease < release
    }

    #[test]
    fn test_version_range_parsing() {
        let range = VersionRange::parse(">=1.0.0").unwrap();
        assert_eq!(range.components.len(), 1);
        assert_eq!(
            range.components[0].operator,
            VersionOperator::GreaterThanOrEqual
        );

        let caret_range = VersionRange::parse("^1.2.3").unwrap();
        assert_eq!(caret_range.components[0].operator, VersionOperator::Caret);
    }

    #[test]
    fn test_version_matching() {
        assert!(VersionMatcher::matches("1.2.3", ">=1.0.0").unwrap());
        assert!(!VersionMatcher::matches("0.9.0", ">=1.0.0").unwrap());

        assert!(VersionMatcher::matches("1.2.5", "^1.2.3").unwrap());
        assert!(!VersionMatcher::matches("2.0.0", "^1.2.3").unwrap());

        assert!(VersionMatcher::matches("1.2.5", "~1.2.3").unwrap());
        assert!(!VersionMatcher::matches("1.3.0", "~1.2.3").unwrap());
    }

    #[test]
    fn test_find_best_match() {
        let versions = vec![
            "1.0.0".to_string(),
            "1.1.0".to_string(),
            "1.2.0".to_string(),
            "2.0.0".to_string(),
        ];

        let best = VersionMatcher::find_best_match(&versions, "^1.0.0").unwrap();
        assert_eq!(best, Some("1.2.0".to_string())); // Highest compatible version

        let best_exact = VersionMatcher::find_best_match(&versions, "=1.1.0").unwrap();
        assert_eq!(best_exact, Some("1.1.0".to_string()));
    }
}
