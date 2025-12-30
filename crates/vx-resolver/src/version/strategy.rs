//! Version resolution strategies for different ecosystems

use super::constraint::{RangeConstraint, Version, VersionConstraint};
use super::resolved::ResolvedVersion;
use crate::runtime_spec::Ecosystem;
use std::cmp::Ordering;
use vx_runtime::VersionInfo;

/// Version resolution strategy trait
///
/// Each ecosystem can have its own implementation to handle
/// ecosystem-specific version semantics.
pub trait VersionStrategy: Send + Sync {
    /// Get the ecosystem this strategy handles
    fn ecosystem(&self) -> Ecosystem;

    /// Check if a version satisfies a constraint
    fn satisfies(&self, version: &Version, constraint: &VersionConstraint) -> bool;

    /// Select the best matching version from available versions
    fn select_best_match(
        &self,
        constraint: &VersionConstraint,
        available: &[VersionInfo],
    ) -> Option<ResolvedVersion>;

    /// Compare two versions
    fn compare(&self, a: &Version, b: &Version) -> Ordering;

    /// Normalize a version string
    fn normalize(&self, version: &str) -> String;
}

/// Default semver strategy (works for most tools)
pub struct SemverStrategy {
    ecosystem: Ecosystem,
}

impl SemverStrategy {
    /// Create a new semver strategy
    pub fn new(ecosystem: Ecosystem) -> Self {
        Self { ecosystem }
    }

    /// Create a strategy for a generic ecosystem
    pub fn generic() -> Self {
        Self {
            ecosystem: Ecosystem::Node,
        }
    }
}

impl Default for SemverStrategy {
    fn default() -> Self {
        Self::generic()
    }
}

impl VersionStrategy for SemverStrategy {
    fn ecosystem(&self) -> Ecosystem {
        self.ecosystem
    }

    fn satisfies(&self, version: &Version, constraint: &VersionConstraint) -> bool {
        match constraint {
            VersionConstraint::Exact(v) => version == v,
            VersionConstraint::Partial { major, minor } => {
                version.major == *major && version.minor == *minor
            }
            VersionConstraint::Major(major) => version.major == *major,
            VersionConstraint::Latest => !version.is_prerelease(),
            VersionConstraint::LatestPrerelease => true,
            VersionConstraint::Range(constraints) => {
                constraints.iter().all(|c| c.satisfies(version))
            }
            VersionConstraint::Wildcard { major, minor } => {
                version.major == *major && version.minor == *minor
            }
            VersionConstraint::Caret(base) => {
                let constraint =
                    RangeConstraint::new(super::constraint::RangeOp::Caret, base.clone());
                constraint.satisfies(version)
            }
            VersionConstraint::Tilde(base) => {
                let constraint =
                    RangeConstraint::new(super::constraint::RangeOp::Tilde, base.clone());
                constraint.satisfies(version)
            }
            VersionConstraint::Any => true,
        }
    }

    fn select_best_match(
        &self,
        constraint: &VersionConstraint,
        available: &[VersionInfo],
    ) -> Option<ResolvedVersion> {
        // Parse and filter versions that satisfy the constraint
        let mut matching: Vec<(Version, &VersionInfo)> = available
            .iter()
            .filter_map(|info| {
                let version = Version::parse(&info.version)?;
                if self.satisfies(&version, constraint) {
                    Some((version, info))
                } else {
                    None
                }
            })
            .collect();

        // For Latest constraint, filter out prereleases
        if matches!(constraint, VersionConstraint::Latest) {
            matching.retain(|(v, info)| !v.is_prerelease() && !info.prerelease);
        }

        // Sort by version (descending) to get the latest
        matching.sort_by(|(a, _), (b, _)| self.compare(b, a));

        // Return the best match
        matching.first().map(|(version, info)| {
            let mut resolved = ResolvedVersion::new(version.clone(), constraint.to_string());
            if let Some(url) = &info.download_url {
                resolved = resolved.with_metadata("download_url", url.clone());
            }
            if let Some(checksum) = &info.checksum {
                resolved = resolved.with_metadata("checksum", checksum.clone());
            }
            if info.lts {
                resolved = resolved.with_metadata("lts", "true");
            }
            resolved
        })
    }

    fn compare(&self, a: &Version, b: &Version) -> Ordering {
        a.cmp(b)
    }

    fn normalize(&self, version: &str) -> String {
        // Strip 'v' prefix if present
        let version = version.strip_prefix('v').unwrap_or(version);
        version.to_string()
    }
}

/// Python PEP 440 strategy
pub struct Pep440Strategy;

impl Default for Pep440Strategy {
    fn default() -> Self {
        Self::new()
    }
}

impl Pep440Strategy {
    pub fn new() -> Self {
        Self
    }
}

impl VersionStrategy for Pep440Strategy {
    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::Python
    }

    fn satisfies(&self, version: &Version, constraint: &VersionConstraint) -> bool {
        // Use the same logic as semver for now
        // TODO: Implement full PEP 440 semantics
        SemverStrategy::new(Ecosystem::Python).satisfies(version, constraint)
    }

    fn select_best_match(
        &self,
        constraint: &VersionConstraint,
        available: &[VersionInfo],
    ) -> Option<ResolvedVersion> {
        SemverStrategy::new(Ecosystem::Python).select_best_match(constraint, available)
    }

    fn compare(&self, a: &Version, b: &Version) -> Ordering {
        a.cmp(b)
    }

    fn normalize(&self, version: &str) -> String {
        // Python versions don't typically have 'v' prefix
        version.to_string()
    }
}

/// Go version strategy (handles go1.22 format)
pub struct GoVersionStrategy;

impl Default for GoVersionStrategy {
    fn default() -> Self {
        Self::new()
    }
}

impl GoVersionStrategy {
    pub fn new() -> Self {
        Self
    }
}

impl VersionStrategy for GoVersionStrategy {
    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::Go
    }

    fn satisfies(&self, version: &Version, constraint: &VersionConstraint) -> bool {
        SemverStrategy::new(Ecosystem::Go).satisfies(version, constraint)
    }

    fn select_best_match(
        &self,
        constraint: &VersionConstraint,
        available: &[VersionInfo],
    ) -> Option<ResolvedVersion> {
        SemverStrategy::new(Ecosystem::Go).select_best_match(constraint, available)
    }

    fn compare(&self, a: &Version, b: &Version) -> Ordering {
        a.cmp(b)
    }

    fn normalize(&self, version: &str) -> String {
        // Go versions are like "go1.22" or "1.22"
        let version = version.strip_prefix("go").unwrap_or(version);
        version.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_version_info(version: &str) -> VersionInfo {
        VersionInfo::new(version)
    }

    #[test]
    fn test_semver_satisfies_exact() {
        let strategy = SemverStrategy::generic();
        let v = Version::new(1, 2, 3);
        let constraint = VersionConstraint::Exact(Version::new(1, 2, 3));
        assert!(strategy.satisfies(&v, &constraint));

        let constraint = VersionConstraint::Exact(Version::new(1, 2, 4));
        assert!(!strategy.satisfies(&v, &constraint));
    }

    #[test]
    fn test_semver_satisfies_partial() {
        let strategy = SemverStrategy::generic();
        let v = Version::new(3, 11, 5);
        let constraint = VersionConstraint::Partial {
            major: 3,
            minor: 11,
        };
        assert!(strategy.satisfies(&v, &constraint));

        let v = Version::new(3, 12, 0);
        assert!(!strategy.satisfies(&v, &constraint));
    }

    #[test]
    fn test_semver_select_best_match_latest() {
        let strategy = SemverStrategy::generic();
        let available = vec![
            make_version_info("1.0.0"),
            make_version_info("1.1.0"),
            make_version_info("2.0.0"),
            VersionInfo::new("3.0.0-alpha").with_prerelease(true),
        ];

        let result = strategy.select_best_match(&VersionConstraint::Latest, &available);
        assert!(result.is_some());
        assert_eq!(result.unwrap().version, Version::new(2, 0, 0));
    }

    #[test]
    fn test_semver_select_best_match_partial() {
        let strategy = SemverStrategy::generic();
        let available = vec![
            make_version_info("3.10.0"),
            make_version_info("3.11.0"),
            make_version_info("3.11.5"),
            make_version_info("3.11.11"),
            make_version_info("3.12.0"),
        ];

        let result = strategy.select_best_match(
            &VersionConstraint::Partial {
                major: 3,
                minor: 11,
            },
            &available,
        );
        assert!(result.is_some());
        assert_eq!(result.unwrap().version, Version::new(3, 11, 11));
    }

    #[test]
    fn test_go_normalize() {
        let strategy = GoVersionStrategy::new();
        assert_eq!(strategy.normalize("go1.22"), "1.22");
        assert_eq!(strategy.normalize("1.22"), "1.22");
    }
}
