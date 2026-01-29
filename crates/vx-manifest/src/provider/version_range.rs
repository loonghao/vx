//! Version Range Configuration
//!
//! This module provides types for configuring version ranges in provider manifests.
//! It enables providers to define:
//! - Default version range for "latest" requests
//! - Maximum/minimum allowed versions
//! - Deprecated version ranges
//! - Versions with known issues
//! - Recommended stable version ranges

use serde::{Deserialize, Serialize};

/// Version range configuration for a runtime in provider.toml
///
/// # Example
///
/// ```toml
/// [runtimes.version_ranges]
/// default = "^5.0"           # When user specifies "latest", use ^5.0
/// maximum = "<6.0"           # Don't allow versions >= 6.0
/// minimum = ">=4.0"          # Don't allow versions < 4.0
/// deprecated = ["<4.0"]      # Warn if using deprecated versions
/// warning = ["5.0.0", "5.1.0"]  # Versions with known issues
/// recommended = "^5.4"       # Recommended stable version range
/// ```
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct VersionRangeConfig {
    /// Default version range applied when user specifies "latest"
    ///
    /// This allows providers to define what "latest" means for their tool.
    /// For example, pnpm might set this to "^9.0" to keep users on the
    /// current major version by default.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default: Option<String>,

    /// Maximum allowed version constraint
    ///
    /// Versions that don't satisfy this constraint will be rejected
    /// unless --force is used. This is useful for preventing users
    /// from accidentally upgrading to incompatible major versions.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub maximum: Option<String>,

    /// Minimum allowed version constraint
    ///
    /// Versions that don't satisfy this constraint will be rejected.
    /// This is useful for ensuring users don't use versions with
    /// known security vulnerabilities.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub minimum: Option<String>,

    /// List of deprecated version ranges
    ///
    /// If the resolved version matches any of these ranges, a deprecation
    /// warning will be shown to the user.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub deprecated: Vec<String>,

    /// List of versions with known issues
    ///
    /// If the resolved version matches any of these, a warning will be
    /// shown to the user about potential issues.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub warning: Vec<String>,

    /// Recommended stable version range
    ///
    /// This is shown to users as a suggestion when they're choosing
    /// a version. It represents the version range that the provider
    /// recommends for production use.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recommended: Option<String>,
}

impl VersionRangeConfig {
    /// Create a new empty version range config
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the default version range
    pub fn with_default(mut self, default: impl Into<String>) -> Self {
        self.default = Some(default.into());
        self
    }

    /// Set the maximum version constraint
    pub fn with_maximum(mut self, maximum: impl Into<String>) -> Self {
        self.maximum = Some(maximum.into());
        self
    }

    /// Set the minimum version constraint
    pub fn with_minimum(mut self, minimum: impl Into<String>) -> Self {
        self.minimum = Some(minimum.into());
        self
    }

    /// Add a deprecated version range
    pub fn with_deprecated(mut self, deprecated: impl Into<String>) -> Self {
        self.deprecated.push(deprecated.into());
        self
    }

    /// Add a version with known issues
    pub fn with_warning(mut self, warning: impl Into<String>) -> Self {
        self.warning.push(warning.into());
        self
    }

    /// Set the recommended version range
    pub fn with_recommended(mut self, recommended: impl Into<String>) -> Self {
        self.recommended = Some(recommended.into());
        self
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

/// Pinning strategy for version locking
///
/// When a version is resolved and locked, this determines how
/// much of the version to lock to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum PinningStrategy {
    /// Lock to major version (e.g., ^5.0.0 allows 5.x.x)
    #[default]
    Major,
    /// Lock to minor version (e.g., ~5.4.0 allows 5.4.x)
    Minor,
    /// Lock to patch version (e.g., =5.4.1)
    Patch,
    /// Lock to exact version including prerelease tags
    Exact,
    /// No locking - always resolve to latest (dangerous)
    None,
}

impl PinningStrategy {
    /// Get the display name for this strategy
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Major => "major",
            Self::Minor => "minor",
            Self::Patch => "patch",
            Self::Exact => "exact",
            Self::None => "none",
        }
    }

    /// Create a version range string from a version using this strategy
    pub fn create_range(&self, major: u64, minor: u64, patch: u64) -> String {
        match self {
            Self::Major => format!("^{}.{}.{}", major, minor, patch),
            Self::Minor => format!("~{}.{}.{}", major, minor, patch),
            Self::Patch => format!("={}.{}.{}", major, minor, patch),
            Self::Exact => format!("={}.{}.{}", major, minor, patch),
            Self::None => "latest".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_range_config_default() {
        let config = VersionRangeConfig::default();
        assert!(config.is_empty());
        assert!(config.default.is_none());
        assert!(config.maximum.is_none());
    }

    #[test]
    fn test_version_range_config_builder() {
        let config = VersionRangeConfig::new()
            .with_default("^5.0")
            .with_maximum("<6.0")
            .with_minimum(">=4.0")
            .with_deprecated("<4.0")
            .with_warning("5.0.0")
            .with_recommended("^5.4");

        assert_eq!(config.default, Some("^5.0".to_string()));
        assert_eq!(config.maximum, Some("<6.0".to_string()));
        assert_eq!(config.minimum, Some(">=4.0".to_string()));
        assert_eq!(config.deprecated, vec!["<4.0"]);
        assert_eq!(config.warning, vec!["5.0.0"]);
        assert_eq!(config.recommended, Some("^5.4".to_string()));
        assert!(!config.is_empty());
    }

    #[test]
    fn test_pinning_strategy_create_range() {
        assert_eq!(PinningStrategy::Major.create_range(5, 4, 1), "^5.4.1");
        assert_eq!(PinningStrategy::Minor.create_range(5, 4, 1), "~5.4.1");
        assert_eq!(PinningStrategy::Patch.create_range(5, 4, 1), "=5.4.1");
        assert_eq!(PinningStrategy::Exact.create_range(5, 4, 1), "=5.4.1");
        assert_eq!(PinningStrategy::None.create_range(5, 4, 1), "latest");
    }

    #[test]
    fn test_pinning_strategy_default() {
        assert_eq!(PinningStrategy::default(), PinningStrategy::Major);
    }

    #[test]
    fn test_version_range_config_serialize() {
        let config = VersionRangeConfig::new()
            .with_default("^5.0")
            .with_recommended("^5.4");

        let toml = toml::to_string(&config).unwrap();
        assert!(toml.contains("default = \"^5.0\""));
        assert!(toml.contains("recommended = \"^5.4\""));
        // Empty fields should not be serialized
        assert!(!toml.contains("maximum"));
        assert!(!toml.contains("deprecated"));
    }

    #[test]
    fn test_version_range_config_deserialize() {
        let toml = r#"
default = "^5.0"
maximum = "<6.0"
deprecated = ["<4.0", "<3.0"]
"#;
        let config: VersionRangeConfig = toml::from_str(toml).unwrap();
        assert_eq!(config.default, Some("^5.0".to_string()));
        assert_eq!(config.maximum, Some("<6.0".to_string()));
        assert_eq!(config.deprecated, vec!["<4.0", "<3.0"]);
        assert!(config.minimum.is_none());
    }
}
