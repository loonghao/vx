//! Tests for RFC 0023: Version Range Configuration in Provider Manifests
//!
//! These tests verify the VersionRangeConfig type and its integration
//! with RuntimeDef in provider manifests.

use vx_manifest::{PinningStrategy, ProviderManifest, VersionRangeConfig};

// ============================================
// VersionRangeConfig Type Tests
// ============================================

mod version_range_config {
    use super::*;

    #[test]
    fn test_default() {
        let config = VersionRangeConfig::new();
        assert!(config.is_empty());
    }

    #[test]
    fn test_with_default() {
        let config = VersionRangeConfig::new().with_default("^5.0");
        assert_eq!(config.default, Some("^5.0".to_string()));
        assert!(!config.is_empty());
    }

    #[test]
    fn test_with_maximum() {
        let config = VersionRangeConfig::new().with_maximum("<6.0");
        assert_eq!(config.maximum, Some("<6.0".to_string()));
    }

    #[test]
    fn test_with_minimum() {
        let config = VersionRangeConfig::new().with_minimum(">=4.0");
        assert_eq!(config.minimum, Some(">=4.0".to_string()));
    }

    #[test]
    fn test_with_deprecated() {
        let config = VersionRangeConfig::new()
            .with_deprecated("<4.0")
            .with_deprecated("<3.0");
        assert_eq!(config.deprecated, vec!["<4.0", "<3.0"]);
    }

    #[test]
    fn test_with_warning() {
        let config = VersionRangeConfig::new()
            .with_warning("5.0.0")
            .with_warning("5.1.0");
        assert_eq!(config.warning, vec!["5.0.0", "5.1.0"]);
    }

    #[test]
    fn test_with_recommended() {
        let config = VersionRangeConfig::new().with_recommended("^5.4");
        assert_eq!(config.recommended, Some("^5.4".to_string()));
    }

    #[test]
    fn test_full_config() {
        let config = VersionRangeConfig::new()
            .with_default("^5.0")
            .with_maximum("<6.0")
            .with_minimum(">=4.0")
            .with_deprecated("<4.0")
            .with_warning("5.0.0")
            .with_recommended("^5.4");

        assert!(!config.is_empty());
        assert_eq!(config.default, Some("^5.0".to_string()));
        assert_eq!(config.maximum, Some("<6.0".to_string()));
        assert_eq!(config.minimum, Some(">=4.0".to_string()));
        assert_eq!(config.deprecated, vec!["<4.0"]);
        assert_eq!(config.warning, vec!["5.0.0"]);
        assert_eq!(config.recommended, Some("^5.4".to_string()));
    }
}

// ============================================
// PinningStrategy Tests
// ============================================

mod pinning_strategy {
    use super::*;

    #[test]
    fn test_default_is_major() {
        assert_eq!(PinningStrategy::default(), PinningStrategy::Major);
    }

    #[test]
    fn test_display_name() {
        assert_eq!(PinningStrategy::Major.display_name(), "major");
        assert_eq!(PinningStrategy::Minor.display_name(), "minor");
        assert_eq!(PinningStrategy::Patch.display_name(), "patch");
        assert_eq!(PinningStrategy::Exact.display_name(), "exact");
        assert_eq!(PinningStrategy::None.display_name(), "none");
    }

    #[test]
    fn test_create_range_major() {
        assert_eq!(PinningStrategy::Major.create_range(5, 4, 1), "^5.4.1");
    }

    #[test]
    fn test_create_range_minor() {
        assert_eq!(PinningStrategy::Minor.create_range(5, 4, 1), "~5.4.1");
    }

    #[test]
    fn test_create_range_patch() {
        assert_eq!(PinningStrategy::Patch.create_range(5, 4, 1), "=5.4.1");
    }

    #[test]
    fn test_create_range_exact() {
        assert_eq!(PinningStrategy::Exact.create_range(5, 4, 1), "=5.4.1");
    }

    #[test]
    fn test_create_range_none() {
        assert_eq!(PinningStrategy::None.create_range(5, 4, 1), "latest");
    }
}

// ============================================
// Serialization Tests
// ============================================

mod serialization {
    use super::*;

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

    #[test]
    fn test_pinning_strategy_serialize() {
        let strategy = PinningStrategy::Minor;
        let json = serde_json::to_string(&strategy).unwrap();
        assert_eq!(json, "\"minor\"");
    }

    #[test]
    fn test_pinning_strategy_deserialize() {
        let strategy: PinningStrategy = serde_json::from_str("\"patch\"").unwrap();
        assert_eq!(strategy, PinningStrategy::Patch);
    }
}

// ============================================
// RuntimeDef Integration Tests
// ============================================

mod runtime_def_integration {
    use super::*;

    #[test]
    fn test_parse_runtime_with_version_ranges() {
        let toml = r#"
[provider]
name = "test-tool"

[[runtimes]]
name = "test"
executable = "test"

[runtimes.version_ranges]
default = "^5.0"
maximum = "<6.0"
minimum = ">=4.0"
deprecated = ["<4.0"]
warning = ["5.0.0", "5.1.0"]
recommended = "^5.4"
"#;
        let manifest = ProviderManifest::parse(toml).unwrap();
        let runtime = &manifest.runtimes[0];

        let ranges = runtime.version_ranges.as_ref().unwrap();
        assert_eq!(ranges.default, Some("^5.0".to_string()));
        assert_eq!(ranges.maximum, Some("<6.0".to_string()));
        assert_eq!(ranges.minimum, Some(">=4.0".to_string()));
        assert_eq!(ranges.deprecated, vec!["<4.0"]);
        assert_eq!(ranges.warning, vec!["5.0.0", "5.1.0"]);
        assert_eq!(ranges.recommended, Some("^5.4".to_string()));
    }

    #[test]
    fn test_parse_runtime_without_version_ranges() {
        let toml = r#"
[provider]
name = "test-tool"

[[runtimes]]
name = "test"
executable = "test"
"#;
        let manifest = ProviderManifest::parse(toml).unwrap();
        let runtime = &manifest.runtimes[0];

        assert!(runtime.version_ranges.is_none());
    }

    #[test]
    fn test_parse_runtime_with_partial_version_ranges() {
        let toml = r#"
[provider]
name = "test-tool"

[[runtimes]]
name = "test"
executable = "test"

[runtimes.version_ranges]
default = "^9.0"
"#;
        let manifest = ProviderManifest::parse(toml).unwrap();
        let runtime = &manifest.runtimes[0];

        let ranges = runtime.version_ranges.as_ref().unwrap();
        assert_eq!(ranges.default, Some("^9.0".to_string()));
        assert!(ranges.maximum.is_none());
        assert!(ranges.minimum.is_none());
        assert!(ranges.deprecated.is_empty());
        assert!(ranges.warning.is_empty());
        assert!(ranges.recommended.is_none());
    }
}

// ============================================
// Real Provider Tests (using existing manifests)
// ============================================

mod real_provider_tests {
    use super::*;

    #[test]
    fn test_node_manifest_can_have_version_ranges() {
        // Test that existing node manifest can be extended with version_ranges
        let toml = include_str!("../../vx-providers/node/provider.toml");

        // Parse should succeed (version_ranges is optional)
        let manifest = ProviderManifest::parse(toml).expect("Failed to parse node manifest");
        assert_eq!(manifest.provider.name, "node");

        // Check that each runtime can potentially have version_ranges
        for runtime in &manifest.runtimes {
            // version_ranges is optional, so this should not fail
            let _ = runtime.version_ranges.as_ref();
        }
    }

    #[test]
    fn test_pnpm_manifest_can_have_version_ranges() {
        let toml = include_str!("../../vx-providers/pnpm/provider.toml");
        let manifest = ProviderManifest::parse(toml).expect("Failed to parse pnpm manifest");
        assert_eq!(manifest.provider.name, "pnpm");
    }
}
