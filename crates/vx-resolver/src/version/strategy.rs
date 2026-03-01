//! Version resolution strategies for different ecosystems

use super::constraint::{RangeConstraint, RangeOp, Version, VersionConstraint};
use super::resolved::ResolvedVersion;
use crate::runtime_spec::Ecosystem;
use std::cmp::Ordering;
use vx_versions::VersionInfo;

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
            ecosystem: Ecosystem::NodeJs,
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
        self.ecosystem.clone()
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
                let constraint = RangeConstraint::new(RangeOp::Caret, base.clone());
                constraint.satisfies(version)
            }
            VersionConstraint::Tilde(base) => {
                let constraint = RangeConstraint::new(RangeOp::Tilde, base.clone());
                constraint.satisfies(version)
            }
            VersionConstraint::Any => true,
            VersionConstraint::Invalid(_) => false,
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

/// Git for Windows version strategy (handles 2.53.0.windows.1 format)
///
/// Git for Windows uses a special versioning scheme:
/// - `2.53.0.windows.1` means Git 2.53.0, Windows build 1
/// - `2.47.1.windows.2` means Git 2.47.1, Windows build 2
///
/// This strategy normalizes these versions for comparison and matching.
pub struct GitVersionStrategy;

impl Default for GitVersionStrategy {
    fn default() -> Self {
        Self::new()
    }
}

impl GitVersionStrategy {
    pub fn new() -> Self {
        Self
    }

    /// Normalize a Git for Windows version string.
    /// Examples:
    /// - `2.53.0.windows.1` -> `2.53.0`
    /// - `2.47.1.windows.2` -> `2.47.1`
    /// - `v2.53.0.windows.1` -> `2.53.0`
    /// - `2.53.0` -> `2.53.0` (already normalized)
    fn normalize_version(version: &str) -> String {
        // Strip 'v' prefix if present
        let version = version.strip_prefix('v').unwrap_or(version);

        // Remove .windows.X suffix
        // Pattern: major.minor.patch.windows.build
        if let Some(pos) = version.find(".windows.") {
            version[..pos].to_string()
        } else {
            version.to_string()
        }
    }

    /// Check if an available version matches a normalized requested version.
    /// E.g., requested "2.53.0" matches available "2.53.0.windows.1"
    fn version_matches_normalized(available: &str, normalized_requested: &str) -> bool {
        let normalized_available = Self::normalize_version(available);
        normalized_available == normalized_requested
    }
}

impl VersionStrategy for GitVersionStrategy {
    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::Git
    }

    fn satisfies(&self, version: &Version, constraint: &VersionConstraint) -> bool {
        // Use semver logic - the Version should already be normalized
        SemverStrategy::new(Ecosystem::Git).satisfies(version, constraint)
    }

    fn select_best_match(
        &self,
        constraint: &VersionConstraint,
        available: &[VersionInfo],
    ) -> Option<ResolvedVersion> {
        // Git for Windows uses special versioning like "2.53.0.windows.1"
        // We need to handle this specially to preserve the original version string

        match constraint {
            VersionConstraint::Exact(v) => {
                // Try to find a version that normalizes to the requested version
                let normalized_requested = format!("{}.{}.{}", v.major, v.minor, v.patch);

                // Find all matching versions and pick the latest
                let mut matches: Vec<&VersionInfo> = available
                    .iter()
                    .filter(|info| {
                        Self::version_matches_normalized(&info.version, &normalized_requested)
                    })
                    .collect();

                if matches.is_empty() {
                    return None;
                }

                // Sort by raw version string (descending) to get the latest windows build
                matches.sort_by(|a, b| b.version.cmp(&a.version));

                let best = matches.first()?;
                let resolved_version = Version::parse(&best.version)?;
                Some(ResolvedVersion::with_original(
                    resolved_version,
                    &best.version,
                    constraint.to_string(),
                ))
            }
            VersionConstraint::Partial { major, minor } => {
                // Find versions that match major.minor
                let prefix = format!("{}.{}.", major, minor);

                let mut matches: Vec<&VersionInfo> = available
                    .iter()
                    .filter(|info| {
                        let normalized = Self::normalize_version(&info.version);
                        normalized.starts_with(&prefix)
                    })
                    .collect();

                if matches.is_empty() {
                    return None;
                }

                // Sort by normalized version (descending), using semver comparison
                matches.sort_by(|a, b| {
                    let a_norm = Self::normalize_version(&a.version);
                    let b_norm = Self::normalize_version(&b.version);
                    match (Version::parse(&a_norm), Version::parse(&b_norm)) {
                        (Some(va), Some(vb)) => vb.cmp(&va),
                        _ => b_norm.cmp(&a_norm),
                    }
                });

                let best = matches.first()?;
                let resolved_version = Version::parse(&best.version)?;
                Some(ResolvedVersion::with_original(
                    resolved_version,
                    &best.version,
                    constraint.to_string(),
                ))
            }
            VersionConstraint::Major(major) => {
                // Find versions that match major
                let prefix = format!("{}.", major);

                let mut matches: Vec<&VersionInfo> = available
                    .iter()
                    .filter(|info| {
                        let normalized = Self::normalize_version(&info.version);
                        normalized.starts_with(&prefix)
                    })
                    .collect();

                if matches.is_empty() {
                    return None;
                }

                // Sort by normalized version (descending)
                matches.sort_by(|a, b| {
                    let a_norm = Self::normalize_version(&a.version);
                    let b_norm = Self::normalize_version(&b.version);
                    match (Version::parse(&a_norm), Version::parse(&b_norm)) {
                        (Some(va), Some(vb)) => vb.cmp(&va),
                        _ => b_norm.cmp(&a_norm),
                    }
                });

                let best = matches.first()?;
                let resolved_version = Version::parse(&best.version)?;
                Some(ResolvedVersion::with_original(
                    resolved_version,
                    &best.version,
                    constraint.to_string(),
                ))
            }
            VersionConstraint::Latest => {
                // Find the latest version (excluding prereleases)
                let mut stable: Vec<&VersionInfo> =
                    available.iter().filter(|info| !info.prerelease).collect();

                if stable.is_empty() {
                    stable = available.iter().collect();
                }

                // Sort by normalized version (descending)
                stable.sort_by(|a, b| {
                    let a_norm = Self::normalize_version(&a.version);
                    let b_norm = Self::normalize_version(&b.version);
                    // Try semver comparison first
                    match (Version::parse(&a_norm), Version::parse(&b_norm)) {
                        (Some(va), Some(vb)) => vb.cmp(&va),
                        _ => b_norm.cmp(&a_norm),
                    }
                });

                let best = stable.first()?;
                let resolved_version = Version::parse(&best.version)?;
                Some(ResolvedVersion::with_original(
                    resolved_version,
                    &best.version,
                    constraint.to_string(),
                ))
            }
            _ => {
                // For other constraints, use semver with normalized versions
                // but still try to preserve original version strings
                let mut normalized_available: Vec<(usize, VersionInfo)> = available
                    .iter()
                    .enumerate()
                    .map(|(i, info)| {
                        let normalized = Self::normalize_version(&info.version);
                        let mut new_info = info.clone();
                        new_info.version = normalized;
                        (i, new_info)
                    })
                    .collect();

                // Sort by normalized version
                normalized_available.sort_by(|a, b| {
                    match (Version::parse(&a.1.version), Version::parse(&b.1.version)) {
                        (Some(va), Some(vb)) => vb.cmp(&va),
                        _ => b.1.version.cmp(&a.1.version),
                    }
                });

                let best_idx = normalized_available.first()?.0;
                let best = &available[best_idx];
                let resolved_version = Version::parse(&best.version)?;
                Some(ResolvedVersion::with_original(
                    resolved_version,
                    &best.version,
                    constraint.to_string(),
                ))
            }
        }
    }

    fn compare(&self, a: &Version, b: &Version) -> Ordering {
        a.cmp(b)
    }

    fn normalize(&self, version: &str) -> String {
        Self::normalize_version(version)
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

    #[test]
    fn test_git_normalize() {
        let strategy = GitVersionStrategy::new();
        assert_eq!(strategy.normalize("2.53.0.windows.1"), "2.53.0");
        assert_eq!(strategy.normalize("v2.53.0.windows.1"), "2.53.0");
        assert_eq!(strategy.normalize("2.47.1.windows.2"), "2.47.1");
        assert_eq!(strategy.normalize("2.53.0"), "2.53.0");
    }

    #[test]
    fn test_git_select_best_match_exact() {
        let strategy = GitVersionStrategy::new();
        let available = vec![
            make_version_info("2.52.0.windows.1"),
            make_version_info("2.53.0.windows.1"),
            make_version_info("2.53.0.windows.2"),
            make_version_info("2.54.0.windows.1"),
        ];

        // Request "2.53.0" should match "2.53.0.windows.2" (latest windows build)
        let result = strategy.select_best_match(
            &VersionConstraint::Exact(Version::new(2, 53, 0)),
            &available,
        );
        assert!(result.is_some());
        let resolved = result.unwrap();
        assert_eq!(resolved.version_string(), "2.53.0.windows.2");
    }

    #[test]
    fn test_git_select_best_match_partial() {
        let strategy = GitVersionStrategy::new();
        let available = vec![
            make_version_info("2.52.0.windows.1"),
            make_version_info("2.53.0.windows.1"),
            make_version_info("2.53.1.windows.1"),
            make_version_info("2.54.0.windows.1"),
        ];

        // Request "2.53" should match "2.53.1.windows.1" (latest 2.53.x)
        let result = strategy.select_best_match(
            &VersionConstraint::Partial {
                major: 2,
                minor: 53,
            },
            &available,
        );
        assert!(result.is_some());
        let resolved = result.unwrap();
        assert_eq!(resolved.version_string(), "2.53.1.windows.1");
    }

    #[test]
    fn test_git_select_best_match_latest() {
        let strategy = GitVersionStrategy::new();
        let available = vec![
            make_version_info("2.52.0.windows.1"),
            make_version_info("2.53.0.windows.1"),
            make_version_info("2.54.0.windows.1"),
        ];

        let result = strategy.select_best_match(&VersionConstraint::Latest, &available);
        assert!(result.is_some());
        let resolved = result.unwrap();
        assert_eq!(resolved.version_string(), "2.54.0.windows.1");
    }
}
