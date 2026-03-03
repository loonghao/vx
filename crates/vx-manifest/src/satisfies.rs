//! Version satisfaction checking - re-exported from vx-versions
//!
//! The canonical implementation lives in `vx-versions`.
//! This module re-exports all public types for backward compatibility.

pub use vx_versions::{RangeConstraint, RangeOp, Version, VersionConstraint, VersionRequest};

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
