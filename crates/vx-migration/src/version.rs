//! Version parsing and comparison utilities.

use crate::error::{MigrationError, MigrationResult};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::fmt;
use std::str::FromStr;

/// Semantic version
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Version {
    pub major: u64,
    pub minor: u64,
    pub patch: u64,
    pub pre: Option<String>,
}

impl Version {
    /// Create a new version
    pub fn new(major: u64, minor: u64, patch: u64) -> Self {
        Self {
            major,
            minor,
            patch,
            pre: None,
        }
    }

    /// Create a version with prerelease
    pub fn with_pre(major: u64, minor: u64, patch: u64, pre: impl Into<String>) -> Self {
        Self {
            major,
            minor,
            patch,
            pre: Some(pre.into()),
        }
    }

    /// Parse from semver::Version
    pub fn from_semver(v: &semver::Version) -> Self {
        Self {
            major: v.major,
            minor: v.minor,
            patch: v.patch,
            pre: if v.pre.is_empty() {
                None
            } else {
                Some(v.pre.to_string())
            },
        }
    }

    /// Convert to semver::Version
    pub fn to_semver(&self) -> MigrationResult<semver::Version> {
        let version_str = if let Some(pre) = &self.pre {
            format!("{}.{}.{}-{}", self.major, self.minor, self.patch, pre)
        } else {
            format!("{}.{}.{}", self.major, self.minor, self.patch)
        };
        semver::Version::parse(&version_str)
            .map_err(|e| MigrationError::Version(format!("Invalid version: {}", e)))
    }
}

impl Default for Version {
    fn default() -> Self {
        Self::new(0, 0, 0)
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(pre) = &self.pre {
            write!(f, "{}.{}.{}-{}", self.major, self.minor, self.patch, pre)
        } else {
            write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
        }
    }
}

impl FromStr for Version {
    type Err = MigrationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Remove 'v' prefix if present
        let s = s.strip_prefix('v').unwrap_or(s);

        let semver = semver::Version::parse(s)
            .map_err(|e| MigrationError::Version(format!("Invalid version '{}': {}", s, e)))?;

        Ok(Self::from_semver(&semver))
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
        match (&self.pre, &other.pre) {
            (None, None) => Ordering::Equal,
            (Some(_), None) => Ordering::Less,
            (None, Some(_)) => Ordering::Greater,
            (Some(a), Some(b)) => a.cmp(b),
        }
    }
}

/// Version range for matching
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VersionRange {
    /// Minimum version (inclusive)
    pub min: Option<Version>,
    /// Maximum version (exclusive)
    pub max: Option<Version>,
    /// Include minimum
    pub min_inclusive: bool,
    /// Include maximum
    pub max_inclusive: bool,
}

impl VersionRange {
    /// Create a range that matches any version
    pub fn any() -> Self {
        Self {
            min: None,
            max: None,
            min_inclusive: true,
            max_inclusive: true,
        }
    }

    /// Create a range for exact version match
    pub fn exact(version: Version) -> Self {
        Self {
            min: Some(version.clone()),
            max: Some(version),
            min_inclusive: true,
            max_inclusive: true,
        }
    }

    /// Create a range >= min
    pub fn gte(min: Version) -> Self {
        Self {
            min: Some(min),
            max: None,
            min_inclusive: true,
            max_inclusive: true,
        }
    }

    /// Create a range < max
    pub fn lt(max: Version) -> Self {
        Self {
            min: None,
            max: Some(max),
            min_inclusive: true,
            max_inclusive: false,
        }
    }

    /// Create a range [min, max)
    pub fn range(min: Version, max: Version) -> Self {
        Self {
            min: Some(min),
            max: Some(max),
            min_inclusive: true,
            max_inclusive: false,
        }
    }

    /// Check if version matches this range
    pub fn matches(&self, version: &Version) -> bool {
        if let Some(min) = &self.min {
            let cmp = version.cmp(min);
            if self.min_inclusive {
                if cmp == Ordering::Less {
                    return false;
                }
            } else if cmp != Ordering::Greater {
                return false;
            }
        }

        if let Some(max) = &self.max {
            let cmp = version.cmp(max);
            if self.max_inclusive {
                if cmp == Ordering::Greater {
                    return false;
                }
            } else if cmp != Ordering::Less {
                return false;
            }
        }

        true
    }
}

impl Default for VersionRange {
    fn default() -> Self {
        Self::any()
    }
}

impl fmt::Display for VersionRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (&self.min, &self.max) {
            (None, None) => write!(f, "*"),
            (Some(min), None) => {
                if self.min_inclusive {
                    write!(f, ">={}", min)
                } else {
                    write!(f, ">{}", min)
                }
            }
            (None, Some(max)) => {
                if self.max_inclusive {
                    write!(f, "<={}", max)
                } else {
                    write!(f, "<{}", max)
                }
            }
            (Some(min), Some(max)) if min == max => write!(f, "={}", min),
            (Some(min), Some(max)) => {
                let min_op = if self.min_inclusive { ">=" } else { ">" };
                let max_op = if self.max_inclusive { "<=" } else { "<" };
                write!(f, "{}{}, {}{}", min_op, min, max_op, max)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_parse() {
        let v = "1.2.3".parse::<Version>().unwrap();
        assert_eq!(v.major, 1);
        assert_eq!(v.minor, 2);
        assert_eq!(v.patch, 3);
        assert_eq!(v.pre, None);

        let v = "v1.2.3-beta.1".parse::<Version>().unwrap();
        assert_eq!(v.major, 1);
        assert_eq!(v.pre, Some("beta.1".to_string()));
    }

    #[test]
    fn test_version_ordering() {
        let v1 = Version::new(1, 0, 0);
        let v2 = Version::new(1, 1, 0);
        let v3 = Version::new(2, 0, 0);
        let v4 = Version::with_pre(1, 0, 0, "alpha");

        assert!(v1 < v2);
        assert!(v2 < v3);
        assert!(v4 < v1); // prerelease < release
    }

    #[test]
    fn test_version_range() {
        let range = VersionRange::range(Version::new(1, 0, 0), Version::new(2, 0, 0));

        assert!(range.matches(&Version::new(1, 0, 0)));
        assert!(range.matches(&Version::new(1, 5, 0)));
        assert!(!range.matches(&Version::new(0, 9, 0)));
        assert!(!range.matches(&Version::new(2, 0, 0))); // exclusive
    }
}
