//! Version constraint types

use serde::{Deserialize, Serialize};
use std::fmt;

/// Semantic version structure
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub prerelease: Option<String>,
    pub build: Option<String>,
}

impl Version {
    /// Create a new version
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
            prerelease: None,
            build: None,
        }
    }

    /// Create a version with prerelease tag
    pub fn with_prerelease(mut self, prerelease: impl Into<String>) -> Self {
        self.prerelease = Some(prerelease.into());
        self
    }

    /// Create a version with build metadata
    pub fn with_build(mut self, build: impl Into<String>) -> Self {
        self.build = Some(build.into());
        self
    }

    /// Parse a version string
    pub fn parse(s: &str) -> Option<Self> {
        let s = s.strip_prefix('v').unwrap_or(s);

        // Split off build metadata first
        let (version_pre, build) = if let Some(idx) = s.find('+') {
            (&s[..idx], Some(s[idx + 1..].to_string()))
        } else {
            (s, None)
        };

        // Split off prerelease
        let (version, prerelease) = if let Some(idx) = version_pre.find('-') {
            (
                &version_pre[..idx],
                Some(version_pre[idx + 1..].to_string()),
            )
        } else {
            (version_pre, None)
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
            build,
        })
    }

    /// Check if this is a prerelease version
    pub fn is_prerelease(&self) -> bool {
        self.prerelease.is_some()
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)?;
        if let Some(ref pre) = self.prerelease {
            write!(f, "-{}", pre)?;
        }
        if let Some(ref build) = self.build {
            write!(f, "+{}", build)?;
        }
        Ok(())
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.major.cmp(&other.major) {
            std::cmp::Ordering::Equal => {}
            ord => return ord,
        }
        match self.minor.cmp(&other.minor) {
            std::cmp::Ordering::Equal => {}
            ord => return ord,
        }
        match self.patch.cmp(&other.patch) {
            std::cmp::Ordering::Equal => {}
            ord => return ord,
        }

        // Prerelease versions have lower precedence than normal versions
        match (&self.prerelease, &other.prerelease) {
            (None, None) => std::cmp::Ordering::Equal,
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (Some(a), Some(b)) => a.cmp(b),
        }
    }
}

/// Version constraint types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub enum VersionConstraint {
    /// Exact version: "3.11.11"
    Exact(Version),
    /// Partial version: "3.11" (matches latest 3.11.x)
    Partial { major: u32, minor: u32 },
    /// Major version only: "3" (matches latest 3.x.x)
    Major(u32),
    /// Latest stable version
    #[default]
    Latest,
    /// Latest prerelease version
    LatestPrerelease,
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
}

impl fmt::Display for VersionConstraint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Exact(v) => write!(f, "{}", v),
            Self::Partial { major, minor } => write!(f, "{}.{}", major, minor),
            Self::Major(major) => write!(f, "{}", major),
            Self::Latest => write!(f, "latest"),
            Self::LatestPrerelease => write!(f, "latest-prerelease"),
            Self::Range(constraints) => {
                let parts: Vec<String> = constraints.iter().map(|c| c.to_string()).collect();
                write!(f, "{}", parts.join(","))
            }
            Self::Wildcard { major, minor } => write!(f, "{}.{}.*", major, minor),
            Self::Caret(v) => write!(f, "^{}", v),
            Self::Tilde(v) => write!(f, "~{}", v),
            Self::Any => write!(f, "*"),
        }
    }
}

/// Range constraint
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RangeConstraint {
    pub op: RangeOp,
    pub version: Version,
}

impl RangeConstraint {
    /// Create a new range constraint
    pub fn new(op: RangeOp, version: Version) -> Self {
        Self { op, version }
    }

    /// Check if a version satisfies this constraint
    pub fn satisfies(&self, version: &Version) -> bool {
        match self.op {
            RangeOp::Eq => version == &self.version,
            RangeOp::Ne => version != &self.version,
            RangeOp::Gt => version > &self.version,
            RangeOp::Ge => version >= &self.version,
            RangeOp::Lt => version < &self.version,
            RangeOp::Le => version <= &self.version,
            RangeOp::Tilde => {
                // ~1.2.3 means >=1.2.3, <1.3.0
                version >= &self.version
                    && version.major == self.version.major
                    && version.minor == self.version.minor
            }
            RangeOp::Caret => {
                // ^1.2.3 means >=1.2.3, <2.0.0
                // ^0.2.3 means >=0.2.3, <0.3.0
                // ^0.0.3 means >=0.0.3, <0.0.4
                if version < &self.version {
                    return false;
                }
                if self.version.major > 0 {
                    version.major == self.version.major
                } else if self.version.minor > 0 {
                    version.major == 0 && version.minor == self.version.minor
                } else {
                    version.major == 0 && version.minor == 0 && version.patch == self.version.patch
                }
            }
        }
    }
}

impl fmt::Display for RangeConstraint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.op, self.version)
    }
}

/// Range operators
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RangeOp {
    /// Equal: =
    Eq,
    /// Not equal: !=
    Ne,
    /// Greater than: >
    Gt,
    /// Greater than or equal: >=
    Ge,
    /// Less than: <
    Lt,
    /// Less than or equal: <=
    Le,
    /// Tilde: ~= (compatible release)
    Tilde,
    /// Caret: ^ (compatible with)
    Caret,
}

impl fmt::Display for RangeOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Eq => write!(f, "="),
            Self::Ne => write!(f, "!="),
            Self::Gt => write!(f, ">"),
            Self::Ge => write!(f, ">="),
            Self::Lt => write!(f, "<"),
            Self::Le => write!(f, "<="),
            Self::Tilde => write!(f, "~"),
            Self::Caret => write!(f, "^"),
        }
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
        assert_eq!(
            Version::parse("1.2.3-alpha"),
            Some(Version::new(1, 2, 3).with_prerelease("alpha"))
        );
        assert_eq!(
            Version::parse("1.2.3+build"),
            Some(Version::new(1, 2, 3).with_build("build"))
        );
    }

    #[test]
    fn test_version_ordering() {
        assert!(Version::new(2, 0, 0) > Version::new(1, 0, 0));
        assert!(Version::new(1, 1, 0) > Version::new(1, 0, 0));
        assert!(Version::new(1, 0, 1) > Version::new(1, 0, 0));
        assert!(Version::new(1, 0, 0) > Version::new(1, 0, 0).with_prerelease("alpha"));
    }

    #[test]
    fn test_range_constraint_satisfies() {
        let v1_2_3 = Version::new(1, 2, 3);
        let v1_2_4 = Version::new(1, 2, 4);
        let v1_3_0 = Version::new(1, 3, 0);
        let v2_0_0 = Version::new(2, 0, 0);

        // Tilde: ~1.2.3 means >=1.2.3, <1.3.0
        let tilde = RangeConstraint::new(RangeOp::Tilde, v1_2_3.clone());
        assert!(tilde.satisfies(&v1_2_3));
        assert!(tilde.satisfies(&v1_2_4));
        assert!(!tilde.satisfies(&v1_3_0));

        // Caret: ^1.2.3 means >=1.2.3, <2.0.0
        let caret = RangeConstraint::new(RangeOp::Caret, v1_2_3.clone());
        assert!(caret.satisfies(&v1_2_3));
        assert!(caret.satisfies(&v1_2_4));
        assert!(caret.satisfies(&v1_3_0));
        assert!(!caret.satisfies(&v2_0_0));
    }
}
