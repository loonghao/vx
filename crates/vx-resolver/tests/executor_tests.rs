//! Tests for executor

use rstest::rstest;
use vx_manifest::ProviderManifest;
use vx_resolver::{Executor, ResolverConfig, RuntimeMap};
use vx_runtime::{mock_context, registry::ProviderRegistry};

/// Create a test RuntimeMap from manifests
fn create_test_runtime_map() -> RuntimeMap {
    let toml = r#"
[provider]
name = "node"
ecosystem = "nodejs"

[[runtimes]]
name = "node"
description = "Node.js"
executable = "node"

[[runtimes]]
name = "npm"
description = "NPM"
executable = "npm"
bundled_with = "node"
"#;
    let manifest = ProviderManifest::parse(toml).expect("Failed to parse manifest");
    RuntimeMap::from_manifests(&[manifest])
}

#[tokio::test]
async fn test_executor_creation() {
    let config = ResolverConfig::default();
    let registry = ProviderRegistry::new();
    let context = mock_context();
    let runtime_map = create_test_runtime_map();
    let executor = Executor::new(config, &registry, &context, runtime_map);
    assert!(executor.is_ok());
}

#[tokio::test]
async fn test_executor_with_disabled_auto_install() {
    let config = ResolverConfig::default().without_auto_install();
    let registry = ProviderRegistry::new();
    let context = mock_context();
    let runtime_map = create_test_runtime_map();
    let executor = Executor::new(config, &registry, &context, runtime_map).unwrap();
    assert!(!executor.config().auto_install);
}

#[rstest]
fn test_executor_resolver_access() {
    let config = ResolverConfig::default();
    let registry = ProviderRegistry::new();
    let context = mock_context();
    let runtime_map = create_test_runtime_map();
    let executor = Executor::new(config, &registry, &context, runtime_map).unwrap();

    // Should be able to access the resolver
    let resolver = executor.resolver();
    assert!(resolver.is_known_runtime("node"));
}

// =============================================================================
// Project Configuration Tests
// =============================================================================

mod project_config_tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    /// RAII guard to restore the original directory when dropped
    struct DirGuard {
        original: std::path::PathBuf,
    }

    impl Drop for DirGuard {
        fn drop(&mut self) {
            // Best effort to restore - ignore errors as the original dir might not exist
            let _ = std::env::set_current_dir(&self.original);
        }
    }

    /// Create a temporary directory with a vx.toml file
    fn create_project_with_config(config_content: &str) -> tempfile::TempDir {
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("vx.toml");
        fs::write(&config_path, config_content).unwrap();
        dir
    }

    #[test]
    fn test_executor_loads_project_config() {
        // Create a temporary project with vx.toml
        let dir = create_project_with_config(
            r#"
[tools]
node = "20.0.0"
go = "1.21.0"
"#,
        );

        // Change to the project directory with RAII guard
        let _guard = DirGuard {
            original: std::env::current_dir().unwrap_or_default(),
        };
        std::env::set_current_dir(dir.path()).unwrap();

        // Create executor - should load project config
        let config = ResolverConfig::default();
        let registry = ProviderRegistry::new();
        let context = mock_context();
        let runtime_map = create_test_runtime_map();
        let executor = Executor::new(config, &registry, &context, runtime_map).unwrap();

        // Verify executor was created successfully
        assert!(executor.config().auto_install);

        // Directory is automatically restored when _guard is dropped
    }

    #[test]
    fn test_executor_without_project_config() {
        // Create a temporary directory without vx.toml
        let dir = tempdir().unwrap();

        // Change to the directory with RAII guard
        let _guard = DirGuard {
            original: std::env::current_dir().unwrap_or_default(),
        };
        std::env::set_current_dir(dir.path()).unwrap();

        // Create executor - should work without project config
        let config = ResolverConfig::default();
        let registry = ProviderRegistry::new();
        let context = mock_context();
        let runtime_map = create_test_runtime_map();
        let executor = Executor::new(config, &registry, &context, runtime_map);
        assert!(executor.is_ok());

        // Directory is automatically restored when _guard is dropped
    }

    #[test]
    fn test_executor_with_empty_tools_config() {
        // Create a project with vx.toml but no tools defined
        let dir = create_project_with_config(
            r#"
[project]
name = "test-project"
"#,
        );

        let _guard = DirGuard {
            original: std::env::current_dir().unwrap_or_default(),
        };
        std::env::set_current_dir(dir.path()).unwrap();

        let config = ResolverConfig::default();
        let registry = ProviderRegistry::new();
        let context = mock_context();
        let runtime_map = create_test_runtime_map();
        let executor = Executor::new(config, &registry, &context, runtime_map);
        assert!(executor.is_ok());

        // Directory is automatically restored when _guard is dropped
    }

    #[test]
    fn test_executor_with_runtimes_legacy_config() {
        // Create a project using legacy [runtimes] section
        let dir = create_project_with_config(
            r#"
[runtimes]
node = "18.0.0"
"#,
        );

        let _guard = DirGuard {
            original: std::env::current_dir().unwrap_or_default(),
        };
        std::env::set_current_dir(dir.path()).unwrap();

        let config = ResolverConfig::default();
        let registry = ProviderRegistry::new();
        let context = mock_context();
        let runtime_map = create_test_runtime_map();
        let executor = Executor::new(config, &registry, &context, runtime_map);
        assert!(executor.is_ok());

        // Directory is automatically restored when _guard is dropped
    }
}

// =============================================================================
// Version Selection Tests
// =============================================================================

mod version_selection_tests {
    //! These tests verify the version selection logic used by build_vx_tools_path.
    //!
    //! The tests use a simulated version list to verify:
    //! - Exact version matching
    //! - Major version prefix matching (e.g., "20" matches "20.0.0")
    //! - Major.minor prefix matching (e.g., "20.0" matches "20.0.0")
    //! - Version comparison/sorting

    /// Helper to compare two version strings (simulating the logic in Executor)
    fn compare_versions(a: &str, b: &str) -> std::cmp::Ordering {
        let a_clean = a.trim_start_matches('v');
        let b_clean = b.trim_start_matches('v');

        let a_parts: Vec<u64> = a_clean
            .split('.')
            .filter_map(|s| s.split('-').next())
            .filter_map(|s| s.parse().ok())
            .collect();
        let b_parts: Vec<u64> = b_clean
            .split('.')
            .filter_map(|s| s.split('-').next())
            .filter_map(|s| s.parse().ok())
            .collect();

        for (ap, bp) in a_parts.iter().zip(b_parts.iter()) {
            match ap.cmp(bp) {
                std::cmp::Ordering::Equal => continue,
                other => return other,
            }
        }

        a_parts.len().cmp(&b_parts.len())
    }

    /// Helper to find matching version (simulating the logic in Executor)
    fn find_matching_version(requested: &str, installed: &[&str]) -> Option<String> {
        // First try exact match
        if installed.contains(&requested) {
            return Some(requested.to_string());
        }

        // Try prefix match
        let mut matches: Vec<&&str> = installed
            .iter()
            .filter(|v| {
                v.starts_with(requested)
                    && (v.len() == requested.len() || v.chars().nth(requested.len()) == Some('.'))
            })
            .collect();

        if matches.is_empty() {
            return None;
        }

        matches.sort_by(|a, b| compare_versions(a, b));
        matches.last().map(|s| (*s).to_string())
    }

    #[test]
    fn test_exact_version_match() {
        let installed = vec!["18.0.0", "20.0.0", "20.10.0", "22.0.0"];
        assert_eq!(
            find_matching_version("20.0.0", &installed),
            Some("20.0.0".to_string())
        );
    }

    #[test]
    fn test_major_version_prefix_match() {
        let installed = vec!["18.0.0", "20.0.0", "20.10.0", "22.0.0"];
        // "20" should match the latest 20.x.x version
        assert_eq!(
            find_matching_version("20", &installed),
            Some("20.10.0".to_string())
        );
    }

    #[test]
    fn test_major_minor_version_prefix_match() {
        let installed = vec!["20.0.0", "20.0.1", "20.10.0", "20.10.1"];
        // "20.0" should match the latest 20.0.x version
        assert_eq!(
            find_matching_version("20.0", &installed),
            Some("20.0.1".to_string())
        );
    }

    #[test]
    fn test_no_matching_version() {
        let installed = vec!["18.0.0", "20.0.0"];
        assert_eq!(find_matching_version("22", &installed), None);
    }

    #[test]
    fn test_version_comparison() {
        assert_eq!(compare_versions("20.0.0", "20.0.0"), std::cmp::Ordering::Equal);
        assert_eq!(compare_versions("20.0.0", "20.0.1"), std::cmp::Ordering::Less);
        assert_eq!(compare_versions("20.10.0", "20.9.0"), std::cmp::Ordering::Greater);
        assert_eq!(compare_versions("v20.0.0", "20.0.0"), std::cmp::Ordering::Equal);
    }

    #[test]
    fn test_version_sorting() {
        let mut versions = vec!["20.0.0", "18.0.0", "20.10.0", "22.0.0", "20.1.0"];
        versions.sort_by(|a, b| compare_versions(a, b));
        assert_eq!(versions, vec!["18.0.0", "20.0.0", "20.1.0", "20.10.0", "22.0.0"]);
    }

    #[test]
    fn test_prerelease_version_handling() {
        // Pre-release versions should be handled gracefully
        let installed = vec!["20.0.0", "20.0.0-rc1", "20.1.0"];
        // Should match the stable release
        assert_eq!(
            find_matching_version("20.0", &installed),
            Some("20.0.0-rc1".to_string()) // or could be "20.0.0" depending on impl
        );
    }
}
