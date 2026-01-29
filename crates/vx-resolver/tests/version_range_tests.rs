//! Tests for RFC 0023: Version Range Locking System
//!
//! These tests verify the version range configuration and resolution functionality.

use vx_resolver::{
    BoundsCheckResult, Version, VersionRangeConfig, VersionRangeResolver, VersionRequest,
};

// ============================================
// VersionRangeConfig Tests
// ============================================

#[test]
fn test_version_range_config_default() {
    let config = VersionRangeConfig::default();
    assert!(config.is_empty());
    assert!(config.default.is_none());
    assert!(config.maximum.is_none());
    assert!(config.minimum.is_none());
    assert!(config.deprecated.is_empty());
    assert!(config.warning.is_empty());
    assert!(config.recommended.is_none());
}

#[test]
fn test_version_range_config_with_values() {
    let config = VersionRangeConfig {
        default: Some("^5.0".to_string()),
        maximum: Some("<6.0".to_string()),
        minimum: Some(">=4.0".to_string()),
        deprecated: vec!["<4.0".to_string()],
        warning: vec!["5.0.0".to_string()],
        recommended: Some("^5.4".to_string()),
    };

    assert!(!config.is_empty());
    assert_eq!(config.default, Some("^5.0".to_string()));
    assert_eq!(config.maximum, Some("<6.0".to_string()));
    assert_eq!(config.minimum, Some(">=4.0".to_string()));
    assert_eq!(config.deprecated, vec!["<4.0"]);
    assert_eq!(config.warning, vec!["5.0.0"]);
    assert_eq!(config.recommended, Some("^5.4".to_string()));
}

// ============================================
// VersionRangeResolver Tests
// ============================================

mod apply_provider_config {
    use super::*;

    #[test]
    fn test_latest_with_default_range() {
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
    fn test_latest_without_default_range() {
        let request = VersionRequest::latest();
        let config = VersionRangeConfig::default();

        let result = VersionRangeResolver::apply_provider_config(&request, &config);

        assert!(result.original_was_latest);
        assert!(result.applied_default.is_none());
        assert!(result.request.is_latest());
    }

    #[test]
    fn test_specific_version_ignores_default() {
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
    fn test_partial_version_ignores_default() {
        let request = VersionRequest::parse("5.4");
        let config = VersionRangeConfig {
            default: Some("^5.0".to_string()),
            ..Default::default()
        };

        let result = VersionRangeResolver::apply_provider_config(&request, &config);

        assert!(!result.original_was_latest);
        assert!(result.applied_default.is_none());
        assert_eq!(result.request.raw, "5.4");
    }

    #[test]
    fn test_exact_version_ignores_default() {
        let request = VersionRequest::parse("5.4.1");
        let config = VersionRangeConfig {
            default: Some("^5.0".to_string()),
            ..Default::default()
        };

        let result = VersionRangeResolver::apply_provider_config(&request, &config);

        assert!(!result.original_was_latest);
        assert!(result.applied_default.is_none());
        assert_eq!(result.request.raw, "5.4.1");
    }
}

mod check_bounds {
    use super::*;

    #[test]
    fn test_version_within_bounds() {
        let version = Version::new(5, 4, 0);
        let config = VersionRangeConfig {
            minimum: Some(">=4.0".to_string()),
            maximum: Some("<6.0".to_string()),
            ..Default::default()
        };

        let result = VersionRangeResolver::check_bounds(&version, &config);

        assert!(result.is_ok());
        assert!(!result.has_warnings());
    }

    #[test]
    fn test_version_exceeds_maximum() {
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
    fn test_version_below_minimum() {
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
    fn test_version_in_deprecated_range() {
        let version = Version::new(3, 5, 0);
        let config = VersionRangeConfig {
            deprecated: vec!["<4.0".to_string()],
            ..Default::default()
        };

        let result = VersionRangeResolver::check_bounds(&version, &config);

        // Deprecation is a warning, not an error
        assert!(result.is_ok());
        assert!(result.has_warnings());
        assert!(result.warnings[0].contains("deprecated"));
    }

    #[test]
    fn test_version_with_known_issues() {
        let version = Version::new(5, 0, 0);
        let config = VersionRangeConfig {
            warning: vec!["5.0.0".to_string()],
            ..Default::default()
        };

        let result = VersionRangeResolver::check_bounds(&version, &config);

        // Warning is not fatal
        assert!(result.is_ok());
        assert!(result.has_warnings());
        assert!(result.warnings[0].contains("known issues"));
    }

    #[test]
    fn test_multiple_deprecated_ranges() {
        let version = Version::new(3, 0, 0);
        let config = VersionRangeConfig {
            deprecated: vec!["<4.0".to_string(), "<3.5".to_string()],
            ..Default::default()
        };

        let result = VersionRangeResolver::check_bounds(&version, &config);

        assert!(result.is_ok());
        // Version matches both deprecated ranges
        assert_eq!(result.warnings.len(), 2);
    }

    #[test]
    fn test_multiple_constraint_violations() {
        let version = Version::new(7, 0, 0);
        let config = VersionRangeConfig {
            maximum: Some("<6.0".to_string()),
            deprecated: vec![">=6.0".to_string()],
            warning: vec!["7.0.0".to_string()],
            ..Default::default()
        };

        let result = VersionRangeResolver::check_bounds(&version, &config);

        // Has error (exceeds max), and warnings (deprecated + known issues)
        assert!(!result.is_ok());
        assert_eq!(result.errors.len(), 1);
        assert_eq!(result.warnings.len(), 2);
    }

    #[test]
    fn test_le_constraint() {
        let version = Version::new(6, 0, 0);
        let config = VersionRangeConfig {
            maximum: Some("<=6.0.0".to_string()),
            ..Default::default()
        };

        let result = VersionRangeResolver::check_bounds(&version, &config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_ge_constraint() {
        let version = Version::new(4, 0, 0);
        let config = VersionRangeConfig {
            minimum: Some(">=4.0.0".to_string()),
            ..Default::default()
        };

        let result = VersionRangeResolver::check_bounds(&version, &config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_ne_constraint_no_warning() {
        let version = Version::new(5, 0, 0);
        let config = VersionRangeConfig {
            warning: vec!["!=5.0.0".to_string()],
            ..Default::default()
        };

        // 5.0.0 == 5.0.0, so it does NOT match the !=5.0.0 constraint, no warning
        let result = VersionRangeResolver::check_bounds(&version, &config);
        assert!(result.is_ok());
        assert!(!result.has_warnings());
    }
}

// ============================================
// BoundsCheckResult Tests
// ============================================

mod bounds_check_result {
    use super::*;

    #[test]
    fn test_new_result_is_ok() {
        let result = BoundsCheckResult::new();
        assert!(result.is_ok());
        assert!(!result.has_warnings());
    }

    #[test]
    fn test_add_warning() {
        let mut result = BoundsCheckResult::new();
        result.add_warning("This is a warning");

        assert!(result.is_ok()); // Warnings don't affect is_ok
        assert!(result.has_warnings());
        assert_eq!(result.warnings.len(), 1);
    }

    #[test]
    fn test_add_error() {
        let mut result = BoundsCheckResult::new();
        result.add_error("This is an error");

        assert!(!result.is_ok());
        assert_eq!(result.errors.len(), 1);
    }

    #[test]
    fn test_mixed_warnings_and_errors() {
        let mut result = BoundsCheckResult::new();
        result.add_warning("Warning 1");
        result.add_error("Error 1");
        result.add_warning("Warning 2");

        assert!(!result.is_ok());
        assert!(result.has_warnings());
        assert_eq!(result.warnings.len(), 2);
        assert_eq!(result.errors.len(), 1);
    }
}

// ============================================
// Serialization Tests
// ============================================

mod serialization {
    use super::*;

    #[test]
    fn test_version_range_config_toml_roundtrip() {
        let config = VersionRangeConfig {
            default: Some("^5.0".to_string()),
            maximum: Some("<6.0".to_string()),
            minimum: Some(">=4.0".to_string()),
            deprecated: vec!["<4.0".to_string()],
            warning: vec!["5.0.0".to_string()],
            recommended: Some("^5.4".to_string()),
        };

        let toml_str = toml::to_string(&config).unwrap();
        let parsed: VersionRangeConfig = toml::from_str(&toml_str).unwrap();

        assert_eq!(config.default, parsed.default);
        assert_eq!(config.maximum, parsed.maximum);
        assert_eq!(config.minimum, parsed.minimum);
        assert_eq!(config.deprecated, parsed.deprecated);
        assert_eq!(config.warning, parsed.warning);
        assert_eq!(config.recommended, parsed.recommended);
    }

    #[test]
    fn test_version_range_config_skip_empty_fields() {
        let config = VersionRangeConfig {
            default: Some("^5.0".to_string()),
            ..Default::default()
        };

        let toml_str = toml::to_string(&config).unwrap();

        assert!(toml_str.contains("default"));
        assert!(!toml_str.contains("maximum"));
        assert!(!toml_str.contains("minimum"));
        assert!(!toml_str.contains("deprecated"));
        assert!(!toml_str.contains("warning"));
        assert!(!toml_str.contains("recommended"));
    }

    #[test]
    fn test_version_range_config_parse_minimal() {
        let toml_str = r#"default = "^5.0""#;
        let config: VersionRangeConfig = toml::from_str(toml_str).unwrap();

        assert_eq!(config.default, Some("^5.0".to_string()));
        assert!(config.maximum.is_none());
        assert!(config.deprecated.is_empty());
    }
}
