//! Version Range Resolution
//!
//! This module provides utilities for applying version range configurations
//! from providers to resolve version requests.

use crate::version::{Version, VersionRequest};
use serde::{Deserialize, Serialize};

/// Result of checking version bounds against provider configuration
#[derive(Debug, Clone, Default)]
pub struct BoundsCheckResult {
    /// Warning messages (non-fatal)
    pub warnings: Vec<String>,
    /// Error messages (fatal)
    pub errors: Vec<String>,
}

impl BoundsCheckResult {
    /// Create a new empty result
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if the result is OK (no errors)
    pub fn is_ok(&self) -> bool {
        self.errors.is_empty()
    }

    /// Check if there are any warnings
    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }

    /// Add a warning
    pub fn add_warning(&mut self, warning: impl Into<String>) {
        self.warnings.push(warning.into());
    }

    /// Add an error
    pub fn add_error(&mut self, error: impl Into<String>) {
        self.errors.push(error.into());
    }
}

/// Version range configuration for checking bounds
///
/// This is a re-export of the manifest type with additional methods
/// for version resolution.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct VersionRangeConfig {
    /// Default version range applied when user specifies "latest"
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default: Option<String>,

    /// Maximum allowed version constraint
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub maximum: Option<String>,

    /// Minimum allowed version constraint
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub minimum: Option<String>,

    /// List of deprecated version ranges
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub deprecated: Vec<String>,

    /// List of versions with known issues
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub warning: Vec<String>,

    /// Recommended stable version range
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recommended: Option<String>,
}

impl VersionRangeConfig {
    /// Create a new empty config
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if any configuration is set
    pub fn is_empty(&self) -> bool {
        self.default.is_none()
            && self.maximum.is_none()
            && self.minimum.is_none()
            && self.deprecated.is_empty()
            && self.warning.is_empty()
            && self.recommended.is_none()
    }
}

/// Version range resolver for applying provider configurations
pub struct VersionRangeResolver;

impl VersionRangeResolver {
    /// Apply provider's version range configuration to a version request
    ///
    /// If the request is "latest" and the provider has a default range,
    /// the default range is applied instead.
    pub fn apply_provider_config(
        request: &VersionRequest,
        config: &VersionRangeConfig,
    ) -> ApplyConfigResult {
        // If request is "latest" and provider has a default, use the default
        if request.is_latest()
            && let Some(default) = &config.default
        {
            return ApplyConfigResult {
                request: VersionRequest::parse(default),
                applied_default: Some(default.clone()),
                original_was_latest: true,
            };
        }

        ApplyConfigResult {
            request: request.clone(),
            applied_default: None,
            original_was_latest: request.is_latest(),
        }
    }

    /// Check if a version satisfies the provider's bounds
    pub fn check_bounds(version: &Version, config: &VersionRangeConfig) -> BoundsCheckResult {
        let mut result = BoundsCheckResult::new();

        // Check maximum version constraint
        if let Some(max) = &config.maximum
            && let Some(constraint) = Self::parse_single_constraint(max)
            && !constraint.matches(version)
        {
            result.add_error(format!(
                "Version {} exceeds maximum allowed {}",
                version, max
            ));
        }

        // Check minimum version constraint
        if let Some(min) = &config.minimum
            && let Some(constraint) = Self::parse_single_constraint(min)
            && !constraint.matches(version)
        {
            result.add_error(format!(
                "Version {} is below minimum required {}",
                version, min
            ));
        }

        // Check deprecated versions
        for dep in &config.deprecated {
            if let Some(constraint) = Self::parse_single_constraint(dep)
                && constraint.matches(version)
            {
                result.add_warning(format!(
                    "Version {} is deprecated (matches {})",
                    version, dep
                ));
            }
        }

        // Check warning versions
        for warn in &config.warning {
            if let Some(constraint) = Self::parse_single_constraint(warn)
                && constraint.matches(version)
            {
                result.add_warning(format!(
                    "Version {} has known issues (matches {})",
                    version, warn
                ));
            }
        }

        result
    }

    /// Parse a single version constraint from a string
    fn parse_single_constraint(s: &str) -> Option<SimpleConstraint> {
        let s = s.trim();

        // Try different operator prefixes
        let operators = [
            (">=", ConstraintOp::Ge),
            ("<=", ConstraintOp::Le),
            ("!=", ConstraintOp::Ne),
            (">", ConstraintOp::Gt),
            ("<", ConstraintOp::Lt),
            ("=", ConstraintOp::Eq),
        ];

        for (prefix, op) in operators {
            if let Some(version_str) = s.strip_prefix(prefix)
                && let Some(version) = Version::parse(version_str.trim())
            {
                return Some(SimpleConstraint { op, version });
            }
        }

        // Try parsing as exact version
        if let Some(version) = Version::parse(s) {
            return Some(SimpleConstraint {
                op: ConstraintOp::Eq,
                version,
            });
        }

        None
    }
}

/// Result of applying provider configuration to a version request
#[derive(Debug, Clone)]
pub struct ApplyConfigResult {
    /// The resulting version request (may be modified)
    pub request: VersionRequest,
    /// The default range that was applied, if any
    pub applied_default: Option<String>,
    /// Whether the original request was "latest"
    pub original_was_latest: bool,
}

/// Simple constraint for bounds checking
#[derive(Debug, Clone)]
struct SimpleConstraint {
    op: ConstraintOp,
    version: Version,
}

impl SimpleConstraint {
    fn matches(&self, v: &Version) -> bool {
        match self.op {
            ConstraintOp::Eq => v == &self.version,
            ConstraintOp::Ne => v != &self.version,
            ConstraintOp::Gt => v > &self.version,
            ConstraintOp::Ge => v >= &self.version,
            ConstraintOp::Lt => v < &self.version,
            ConstraintOp::Le => v <= &self.version,
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum ConstraintOp {
    Eq,
    Ne,
    Gt,
    Ge,
    Lt,
    Le,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_latest_with_default() {
        let request = VersionRequest::latest();
        let config = VersionRangeConfig {
            default: Some("^5.0".to_string()),
            ..Default::default()
        };

        let result = VersionRangeResolver::apply_provider_config(&request, &config);
        assert!(result.original_was_latest);
        assert_eq!(result.applied_default, Some("^5.0".to_string()));
        assert_eq!(result.request.raw, "^5.0");
    }

    #[test]
    fn test_apply_latest_without_default() {
        let request = VersionRequest::latest();
        let config = VersionRangeConfig::default();

        let result = VersionRangeResolver::apply_provider_config(&request, &config);
        assert!(result.original_was_latest);
        assert!(result.applied_default.is_none());
        assert!(result.request.is_latest());
    }

    #[test]
    fn test_apply_specific_version_ignores_default() {
        let request = VersionRequest::parse("^4.0");
        let config = VersionRangeConfig {
            default: Some("^5.0".to_string()),
            ..Default::default()
        };

        let result = VersionRangeResolver::apply_provider_config(&request, &config);
        assert!(!result.original_was_latest);
        assert!(result.applied_default.is_none());
        assert_eq!(result.request.raw, "^4.0");
    }

    #[test]
    fn test_check_bounds_maximum() {
        let version = Version::new(6, 0, 0);
        let config = VersionRangeConfig {
            maximum: Some("<6.0".to_string()),
            ..Default::default()
        };

        let result = VersionRangeResolver::check_bounds(&version, &config);
        assert!(!result.is_ok());
        assert!(result.errors[0].contains("exceeds maximum"));
    }

    #[test]
    fn test_check_bounds_minimum() {
        let version = Version::new(3, 0, 0);
        let config = VersionRangeConfig {
            minimum: Some(">=4.0".to_string()),
            ..Default::default()
        };

        let result = VersionRangeResolver::check_bounds(&version, &config);
        assert!(!result.is_ok());
        assert!(result.errors[0].contains("below minimum"));
    }

    #[test]
    fn test_check_bounds_deprecated() {
        let version = Version::new(3, 5, 0);
        let config = VersionRangeConfig {
            deprecated: vec!["<4.0".to_string()],
            ..Default::default()
        };

        let result = VersionRangeResolver::check_bounds(&version, &config);
        assert!(result.is_ok()); // Deprecation is a warning, not error
        assert!(result.has_warnings());
        assert!(result.warnings[0].contains("deprecated"));
    }

    #[test]
    fn test_check_bounds_warning() {
        let version = Version::new(5, 0, 0);
        let config = VersionRangeConfig {
            warning: vec!["5.0.0".to_string()],
            ..Default::default()
        };

        let result = VersionRangeResolver::check_bounds(&version, &config);
        assert!(result.is_ok()); // Warning is not fatal
        assert!(result.has_warnings());
        assert!(result.warnings[0].contains("known issues"));
    }

    #[test]
    fn test_check_bounds_all_ok() {
        let version = Version::new(5, 4, 0);
        let config = VersionRangeConfig {
            minimum: Some(">=4.0".to_string()),
            maximum: Some("<6.0".to_string()),
            deprecated: vec!["<4.0".to_string()],
            ..Default::default()
        };

        let result = VersionRangeResolver::check_bounds(&version, &config);
        assert!(result.is_ok());
        assert!(!result.has_warnings());
    }
}
