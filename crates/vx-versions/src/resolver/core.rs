//! Core version types and generic semver resolution
//!
//! This module contains the shared types used by all ecosystem-specific resolvers:
//! - [`Version`]: Parsed semantic version
//! - [`VersionConstraint`]: Version constraint types
//! - [`RangeConstraint`] / [`RangeOp`]: Range constraint primitives
//! - [`VersionRequest`]: High-level version request with constraint parsing

use crate::VersionInfo;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

/// Parsed semantic version with optional 4th segment (build/revision)
///
/// Supports version formats:
/// - Standard semver: `1.2.3`
/// - 4-segment versions: `18.0.7.61305` (common in .NET/Windows ecosystem)
/// - With prerelease: `1.2.3-beta.1`
/// - With build metadata: `17.8.5+1c7abc` (metadata is stripped)
/// - Go-style: `go1.22.0`
/// - v-prefixed: `v1.2.3`
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    /// Optional 4th segment (build/revision number), used by .NET, Windows SDK, etc.
    pub build: Option<u32>,
    pub prerelease: Option<String>,
}

impl Version {
    /// Create a new version
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
            build: None,
            prerelease: None,
        }
    }

    /// Create a new version with build segment
    pub fn with_build(major: u32, minor: u32, patch: u32, build: u32) -> Self {
        Self {
            major,
            minor,
            patch,
            build: Some(build),
            prerelease: None,
        }
    }

    /// Parse a version string, stripping common prefixes (v, go)
    pub fn parse(s: &str) -> Option<Self> {
        let s = s.strip_prefix("go").unwrap_or(s);
        let s = s.strip_prefix('v').unwrap_or(s);
        let s = s.split('+').next().unwrap_or(s);

        let (version_part, prerelease) = if let Some(idx) = s.find('-') {
            (&s[..idx], Some(s[idx + 1..].to_string()))
        } else {
            (s, None)
        };

        let parts: Vec<&str> = version_part.split('.').collect();
        if parts.is_empty() || parts.len() > 4 {
            return None;
        }

        let major = parts[0].parse().ok()?;
        let minor = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
        let patch = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(0);
        let build = if parts.len() == 4 {
            Some(parts[3].parse().ok()?)
        } else {
            None
        };

        Some(Self {
            major,
            minor,
            patch,
            build,
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
                    Ordering::Equal => match self.build.cmp(&other.build) {
                        Ordering::Equal => match (&self.prerelease, &other.prerelease) {
                            (None, None) => Ordering::Equal,
                            (Some(_), None) => Ordering::Less,
                            (None, Some(_)) => Ordering::Greater,
                            (Some(a), Some(b)) => a.cmp(b),
                        },
                        other => other,
                    },
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
        if let Some(build) = self.build {
            write!(f, ".{}", build)?;
        }
        if let Some(ref pre) = self.prerelease {
            write!(f, "-{}", pre)?;
        }
        Ok(())
    }
}

/// Version constraint types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VersionConstraint {
    /// Exact version: "3.11.11"
    Exact(Version),
    /// Partial version: "3.11" (matches latest 3.11.x)
    Partial { major: u32, minor: u32 },
    /// Major version only: "20" (matches latest 20.x.x)
    Major(u32),
    /// Latest stable version
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
    /// Compatible release: "~=1.2.3" (Python PEP 440)
    CompatibleRelease { version: Version, parts: u8 },
    /// Any version
    Any,
    /// Invalid version string
    Invalid(String),
}

impl VersionConstraint {
    /// Check if a parsed version satisfies this constraint.
    pub fn satisfies(&self, version: &Version) -> bool {
        match self {
            VersionConstraint::Exact(target) => version == target,
            VersionConstraint::Partial { major, minor } => {
                version.major == *major && version.minor == *minor
            }
            VersionConstraint::Major(major) => version.major == *major,
            VersionConstraint::Latest
            | VersionConstraint::LatestPrerelease
            | VersionConstraint::Any => true,
            VersionConstraint::Range(constraints) => {
                constraints.iter().all(|c| c.satisfies(version))
            }
            VersionConstraint::Wildcard { major, minor } => {
                version.major == *major && version.minor == *minor
            }
            VersionConstraint::Caret(target) => {
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
                if version < target {
                    return false;
                }
                version.major == target.major && version.minor == target.minor
            }
            VersionConstraint::CompatibleRelease {
                version: target,
                parts,
            } => {
                if version < target {
                    return false;
                }
                if *parts <= 2 {
                    version.major == target.major
                } else {
                    version.major == target.major && version.minor == target.minor
                }
            }
            VersionConstraint::Invalid(_) => false,
        }
    }
}

impl std::fmt::Display for VersionConstraint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VersionConstraint::Exact(v) => write!(f, "{}", v),
            VersionConstraint::Partial { major, minor } => write!(f, "{}.{}", major, minor),
            VersionConstraint::Major(major) => write!(f, "{}", major),
            VersionConstraint::Latest => write!(f, "latest"),
            VersionConstraint::LatestPrerelease => write!(f, "latest-prerelease"),
            VersionConstraint::Range(constraints) => {
                let parts: Vec<String> = constraints
                    .iter()
                    .map(|c| {
                        let op = match c.op {
                            RangeOp::Gte => ">=",
                            RangeOp::Gt => ">",
                            RangeOp::Lte => "<=",
                            RangeOp::Lt => "<",
                            RangeOp::Eq => "=",
                            RangeOp::Ne => "!=",
                            RangeOp::Caret => "^",
                            RangeOp::Tilde => "~",
                        };
                        format!("{}{}", op, c.version)
                    })
                    .collect();
                write!(f, "{}", parts.join(","))
            }
            VersionConstraint::Wildcard { major, minor } => write!(f, "{}.{}.*", major, minor),
            VersionConstraint::Caret(v) => write!(f, "^{}", v),
            VersionConstraint::Tilde(v) => write!(f, "~{}", v),
            VersionConstraint::CompatibleRelease { version, .. } => write!(f, "~={}", version),
            VersionConstraint::Any => write!(f, "*"),
            VersionConstraint::Invalid(s) => write!(f, "{}", s),
        }
    }
}

/// Range operator
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RangeOp {
    Gte,
    Gt,
    Lte,
    Lt,
    Eq,
    Ne,
    /// Caret: ^ (compatible with)
    Caret,
    /// Tilde: ~ (approximately equal)
    Tilde,
}

/// A single range constraint
#[derive(Debug, Clone, Serialize, Deserialize)]
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
            RangeOp::Gte => version >= &self.version,
            RangeOp::Gt => version > &self.version,
            RangeOp::Lte => version <= &self.version,
            RangeOp::Lt => version < &self.version,
            RangeOp::Eq => version == &self.version,
            RangeOp::Ne => version != &self.version,
            RangeOp::Caret => {
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
            RangeOp::Tilde => {
                if version < &self.version {
                    return false;
                }
                version.major == self.version.major && version.minor == self.version.minor
            }
        }
    }
}

/// Parses a version constraint string and checks version satisfaction.
///
/// Supports all `VersionConstraint` variants including ranges, caret, tilde,
/// compatible release (`~=`), wildcards, and plain version strings.
#[derive(Debug, Clone)]
pub struct VersionRequest {
    pub raw: String,
    pub constraint: VersionConstraint,
}

impl VersionRequest {
    pub fn parse(raw: impl Into<String>) -> Self {
        let raw = raw.into();
        let constraint = parse_constraint(&raw);
        Self { raw, constraint }
    }

    /// Check if a version string satisfies this constraint
    pub fn satisfies(&self, version: &str) -> bool {
        let v = match Version::parse(version) {
            Some(v) => v,
            None => return false,
        };
        self.constraint.satisfies(&v)
    }
}

/// Parse a version constraint string into a [`VersionConstraint`].
///
/// This is the generic semver parser used by all ecosystems as a baseline.
/// Ecosystem-specific parsers (nodejs, python, rust_eco, opaque) call this
/// after handling their own special cases.
pub fn parse_constraint(version_str: &str) -> VersionConstraint {
    let trimmed = version_str.trim();

    if trimmed.is_empty() || trimmed == "latest" || trimmed == "stable" || trimmed == "lts" {
        return VersionConstraint::Latest;
    }

    if trimmed == "latest-prerelease" || trimmed == "prerelease" || trimmed == "pre" {
        return VersionConstraint::LatestPrerelease;
    }

    if trimmed == "*" || trimmed == "any" {
        return VersionConstraint::Any;
    }

    if let Some(rest) = trimmed.strip_prefix('^')
        && let Some(v) = Version::parse(rest)
    {
        return VersionConstraint::Caret(v);
    }

    // ~= is Python PEP 440 compatible release, must be checked before ~
    if let Some(rest) = trimmed.strip_prefix("~=") {
        let rest = rest.trim();
        let parts_count = rest.split('.').count() as u8;
        if let Some(v) = Version::parse(rest) {
            return VersionConstraint::CompatibleRelease {
                version: v,
                parts: parts_count,
            };
        }
    }

    if let Some(rest) = trimmed.strip_prefix('~')
        && let Some(v) = Version::parse(rest)
    {
        return VersionConstraint::Tilde(v);
    }

    if trimmed.contains(">=")
        || trimmed.contains("<=")
        || trimmed.contains("==")
        || trimmed.starts_with('=')
        || trimmed.contains('>')
        || trimmed.contains('<')
        || trimmed.contains("!=")
    {
        let constraints = parse_range_constraints(trimmed);
        if !constraints.is_empty() {
            return VersionConstraint::Range(constraints);
        }
    }

    if let Some(prefix) = trimmed.strip_suffix(".*") {
        let parts: Vec<&str> = prefix.split('.').collect();
        if parts.len() == 2
            && let (Ok(major), Ok(minor)) = (parts[0].parse(), parts[1].parse())
        {
            return VersionConstraint::Wildcard { major, minor };
        }
    }

    if let Some(v) = Version::parse(trimmed) {
        let normalized = trimmed
            .strip_prefix("go")
            .unwrap_or(trimmed)
            .strip_prefix('v')
            .unwrap_or(trimmed)
            .split('+')
            .next()
            .unwrap_or(trimmed)
            .split('-')
            .next()
            .unwrap_or(trimmed);
        let parts: Vec<&str> = normalized.split('.').collect();

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

    if let Ok(major) = trimmed.parse::<u32>() {
        return VersionConstraint::Major(major);
    }

    VersionConstraint::Invalid(trimmed.to_string())
}

fn parse_range_constraints(s: &str) -> Vec<RangeConstraint> {
    let mut constraints = Vec::new();
    let mut parts = s.split([',', ' ']).filter(|p| !p.is_empty()).peekable();

    while let Some(part) = parts.next() {
        let part = part.trim();

        let (op, mut version_str) = if let Some(rest) = part.strip_prefix(">=") {
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

        if version_str.trim().is_empty() {
            version_str = parts.next().unwrap_or("");
        }

        if let Some(version) = Version::parse(version_str.trim()) {
            constraints.push(RangeConstraint { op, version });
        }
    }

    constraints
}

/// Core resolve logic: resolve a constraint against available versions.
///
/// Used by all ecosystem-specific resolvers as the final step.
pub fn resolve_constraint(
    resolver: &super::VersionResolver,
    constraint: &VersionConstraint,
    available: &[VersionInfo],
) -> Option<String> {
    if matches!(constraint, VersionConstraint::Invalid(_)) {
        return None;
    }

    // For "system" / "latest" / "any" constraints on providers that only
    // expose non-numeric versions (e.g. msvc returns ["system"]), we must
    // handle the opaque version strings before attempting semver parsing.
    let all_parseable = available
        .iter()
        .any(|v| Version::parse(&v.version).is_some());

    if !all_parseable
        && matches!(
            constraint,
            VersionConstraint::Latest | VersionConstraint::Any
        )
    {
        let lts = available.iter().find(|v| v.lts);
        return lts.or_else(|| available.first()).map(|v| v.version.clone());
    }

    let mut all_versions: Vec<(Version, &VersionInfo)> = available
        .iter()
        .filter_map(|v| {
            let parsed = Version::parse(&v.version)?;
            Some((parsed, v))
        })
        .collect();

    all_versions.sort_by(|a, b| b.0.cmp(&a.0));

    let stable_versions: Vec<_> = all_versions
        .iter()
        .filter(|(parsed, info)| {
            resolver.allow_prerelease || (!parsed.is_prerelease() && !info.prerelease)
        })
        .cloned()
        .collect();

    if matches!(constraint, VersionConstraint::Latest) && resolver.prefer_lts {
        let lts_versions: Vec<_> = stable_versions.iter().filter(|(_, v)| v.lts).collect();
        if !lts_versions.is_empty() {
            return lts_versions.first().map(|(_, v)| v.version.clone());
        }
    }

    for (parsed, info) in &stable_versions {
        if constraint.satisfies(parsed) {
            return Some(info.version.clone());
        }
    }

    if !resolver.allow_prerelease {
        let should_fallback = matches!(
            constraint,
            VersionConstraint::Partial { .. }
                | VersionConstraint::Major(_)
                | VersionConstraint::Exact(_)
                | VersionConstraint::Wildcard { .. }
        );

        if should_fallback {
            let prerelease_versions: Vec<_> = all_versions
                .iter()
                .filter(|(parsed, info)| parsed.is_prerelease() || info.prerelease)
                .cloned()
                .collect();

            for (parsed, info) in &prerelease_versions {
                if constraint.satisfies(parsed) {
                    return Some(info.version.clone());
                }
            }
        }
    }

    // Final fallback: for Latest/Any constraints, return the first available entry.
    if matches!(
        constraint,
        VersionConstraint::Latest | VersionConstraint::Any
    ) {
        return available.first().map(|v| v.version.clone());
    }

    None
}
