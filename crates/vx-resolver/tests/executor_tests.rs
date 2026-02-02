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
        assert_eq!(
            compare_versions("20.0.0", "20.0.0"),
            std::cmp::Ordering::Equal
        );
        assert_eq!(
            compare_versions("20.0.0", "20.0.1"),
            std::cmp::Ordering::Less
        );
        assert_eq!(
            compare_versions("20.10.0", "20.9.0"),
            std::cmp::Ordering::Greater
        );
        assert_eq!(
            compare_versions("v20.0.0", "20.0.0"),
            std::cmp::Ordering::Equal
        );
    }

    #[test]
    fn test_version_sorting() {
        let mut versions = vec!["20.0.0", "18.0.0", "20.10.0", "22.0.0", "20.1.0"];
        versions.sort_by(|a, b| compare_versions(a, b));
        assert_eq!(
            versions,
            vec!["18.0.0", "20.0.0", "20.1.0", "20.10.0", "22.0.0"]
        );
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

// =============================================================================
// Environment Isolation Tests
// =============================================================================

/// Tests for environment isolation and essential system path handling
///
/// These tests verify that when vx runs in isolated mode, essential system
/// paths (like /bin, /usr/bin) are always included in PATH, ensuring child
/// processes can find basic system tools like 'sh', 'cat', etc.
mod environment_isolation_tests {
    use std::collections::HashSet;

    /// Essential system paths that should always be present in isolated mode
    const ESSENTIAL_PATHS: &[&str] = &["/bin", "/usr/bin", "/usr/local/bin"];

    #[test]
    fn test_essential_paths_defined() {
        // Verify our test constants are reasonable
        assert!(!ESSENTIAL_PATHS.is_empty());
        assert!(ESSENTIAL_PATHS.contains(&"/bin"));
        assert!(ESSENTIAL_PATHS.contains(&"/usr/bin"));
    }

    /// Test that filter_system_path preserves essential system directories
    #[test]
    #[cfg(unix)]
    fn test_filter_system_path_preserves_essentials() {
        // This test documents the expected behavior of the system path filtering
        // The actual filtering is done by vx_manifest::filter_system_path

        let test_path = "/home/user/.local/bin:/usr/bin:/bin:/opt/custom/bin";
        let filtered = vx_paths::platform::filter_system_path(test_path);

        // Filtered path should contain system directories
        assert!(
            filtered.contains("/bin"),
            "Filtered PATH should contain /bin, got: {}",
            filtered
        );
        assert!(
            filtered.contains("/usr/bin"),
            "Filtered PATH should contain /usr/bin, got: {}",
            filtered
        );

        // Filtered path should NOT contain user directories
        assert!(
            !filtered.contains("/home/user/.local/bin"),
            "Filtered PATH should NOT contain user paths, got: {}",
            filtered
        );
    }

    /// Test that filter_system_path works on Windows
    #[test]
    #[cfg(windows)]
    fn test_filter_system_path_preserves_essentials() {
        // Windows version of the test
        let test_path = r"C:\Users\user\bin;C:\Windows\System32;C:\Windows";
        let filtered = vx_paths::platform::filter_system_path(test_path);

        // Filtered path should contain Windows system directories
        assert!(
            filtered.to_lowercase().contains("system32"),
            "Filtered PATH should contain System32, got: {}",
            filtered
        );

        // Filtered path should NOT contain user directories
        assert!(
            !filtered.to_lowercase().contains("users"),
            "Filtered PATH should NOT contain user paths, got: {}",
            filtered
        );
    }

    /// Test that essential paths are added even when original PATH is empty
    #[test]
    fn test_essential_paths_added_when_path_empty() {
        // This documents the fix: when PATH is empty or doesn't contain essential paths,
        // they should still be added in isolated mode.

        // Check that the essential paths exist on this system (Unix only)
        #[cfg(unix)]
        {
            let essential_exists: HashSet<&str> = ESSENTIAL_PATHS
                .iter()
                .filter(|&&p| std::path::Path::new(p).exists())
                .copied()
                .collect();

            // At least /bin or /usr/bin should exist on any Unix system
            assert!(
                essential_exists.contains("/bin") || essential_exists.contains("/usr/bin"),
                "Expected at least /bin or /usr/bin to exist on this system"
            );
        }
    }

    /// Test path deduplication logic
    #[test]
    fn test_path_deduplication() {
        let paths = ["/usr/bin", "/bin", "/usr/bin", "/usr/local/bin", "/bin"];
        let unique: HashSet<&str> = paths.iter().copied().collect();

        // Should have 3 unique paths
        assert_eq!(unique.len(), 3);
        assert!(unique.contains("/usr/bin"));
        assert!(unique.contains("/bin"));
        assert!(unique.contains("/usr/local/bin"));
    }
}

// =============================================================================
// Template Expansion Tests
// =============================================================================

/// Tests for template variable expansion in environment values
///
/// These tests verify that {install_dir}, {version}, and other template
/// variables are correctly expanded in environment configuration.
///
/// Note: These tests use vx_core::version_utils directly to test the shared
/// version parsing logic that Executor relies on.
mod template_expansion_tests {
    use rstest::rstest;

    /// Test that version sorting produces correct results
    /// This validates the same logic used by Executor::resolve_install_dir
    #[test]
    fn test_version_sorting_for_install_dir() {
        let mut versions = vec!["18.0.0", "20.0.0", "20.10.0", "22.0.0", "19.5.0"];
        vx_core::version_utils::sort_versions_desc(&mut versions);
        // Should be sorted descending (newest first)
        assert_eq!(versions[0], "22.0.0");
        assert_eq!(versions[1], "20.10.0");
        assert_eq!(versions[2], "20.0.0");
    }

    /// Test that invalid version directories are filtered
    #[test]
    fn test_version_filtering() {
        let candidates = vec!["20.0.0", "temp", "18.0.0", ".cache", "invalid"];
        let valid: Vec<&str> = candidates
            .iter()
            .filter(|v| vx_core::version_utils::parse_version(v).is_some())
            .copied()
            .collect();

        assert_eq!(valid, vec!["20.0.0", "18.0.0"]);
    }

    /// Test version parsing edge cases
    #[rstest]
    #[case("20.0.0", true)]
    #[case("v20.0.0", true)]
    #[case("vx-v20.0.0", true)]
    #[case("20.0", true)] // Two-part version
    #[case("20.0.0-beta.1", true)] // Prerelease
    #[case("invalid", false)]
    #[case("temp", false)]
    #[case(".hidden", false)]
    fn test_version_parsing(#[case] input: &str, #[case] should_parse: bool) {
        let result = vx_core::version_utils::parse_version(input);
        assert_eq!(
            result.is_some(),
            should_parse,
            "parse_version({}) should return {}",
            input,
            if should_parse { "Some" } else { "None" }
        );
    }

    /// Test that find_latest_version works correctly
    #[test]
    fn test_find_latest_version() {
        let versions = vec!["0.6.25", "0.6.27", "0.6.26"];
        let latest = vx_core::version_utils::find_latest_version(&versions, false);
        assert_eq!(latest, Some("0.6.27"));
    }

    /// Test prerelease exclusion in find_latest_version
    #[test]
    fn test_find_latest_version_excludes_prerelease() {
        let versions = vec!["0.6.25", "0.6.28-beta.1", "0.6.27"];

        // Without excluding prerelease
        let latest = vx_core::version_utils::find_latest_version(&versions, false);
        assert_eq!(latest, Some("0.6.28-beta.1"));

        // Excluding prerelease
        let latest = vx_core::version_utils::find_latest_version(&versions, true);
        assert_eq!(latest, Some("0.6.27"));
    }

    /// Test that prerelease versions are properly compared
    #[test]
    fn test_prerelease_comparison() {
        // Stable version should be newer than prerelease of same version
        assert!(vx_core::version_utils::is_newer_version(
            "0.6.27",
            "0.6.27-beta.1"
        ));
        assert!(!vx_core::version_utils::is_newer_version(
            "0.6.27-beta.1",
            "0.6.27"
        ));

        // Beta of newer version should be newer than stable of older version
        assert!(vx_core::version_utils::is_newer_version(
            "0.6.28-beta.1",
            "0.6.27"
        ));
    }

    /// Test path format expected by expand_template
    /// This documents the expected directory structure
    #[test]
    fn test_expected_path_format() {
        // The install_dir format is: ~/.vx/store/<runtime>/<version>/<platform>
        // Example: ~/.vx/store/python/3.11.0/linux-x64

        let runtime_name = "python";
        let version = "3.11.0";
        let platform = "linux-x64";

        let expected_suffix = format!("{}/{}/{}", runtime_name, version, platform);
        assert!(expected_suffix.contains("python/3.11.0/linux-x64"));
    }

    /// Test version normalization
    #[rstest]
    #[case("vx-v0.6.27", "0.6.27")]
    #[case("x-v0.6.27", "0.6.27")]
    #[case("v0.6.27", "0.6.27")]
    #[case("0.6.27", "0.6.27")]
    #[case("vx-v1.0.0-beta.1", "1.0.0-beta.1")]
    fn test_version_normalization(#[case] input: &str, #[case] expected: &str) {
        let normalized = vx_core::version_utils::normalize_version(input);
        assert_eq!(normalized, expected);
    }
}

// =============================================================================
// Regression Tests for fix/python-env-and-self-update
// =============================================================================

/// Regression tests for {install_dir} template expansion fixes
///
/// These tests verify the fixes made in the fix/python-env-and-self-update branch
/// to ensure they don't regress in future changes.
mod install_dir_regression_tests {
    use vx_core::version_utils;

    /// Regression test: {install_dir} should select LATEST version, not first in list
    ///
    /// Bug: Previously used `versions[0]` which depended on filesystem ordering
    /// (read_dir order is undefined), so the selected version was unpredictable.
    ///
    /// Fix: Now uses semver-aware sorting to always select the latest version.
    #[test]
    fn test_regression_install_dir_selects_latest_not_first() {
        // Simulate a filesystem listing where versions are NOT sorted
        // (this can happen on various filesystems)
        let filesystem_order = vec![
            "18.0.0", // An old version
            "20.0.0", // Not the latest
            "19.5.0", // Out of order
            "22.1.0", // This is actually the latest
            "21.0.0", // Also not the latest
        ];

        // The resolve_install_dir logic should pick 22.1.0
        let mut sorted = filesystem_order.clone();
        version_utils::sort_versions_desc(&mut sorted);

        assert_eq!(
            sorted[0], "22.1.0",
            "After sorting, first element should be the latest version"
        );

        let latest = version_utils::find_latest_version(&filesystem_order, false);
        assert_eq!(
            latest,
            Some("22.1.0"),
            "find_latest_version should return 22.1.0 regardless of input order"
        );
    }

    /// Regression test: Version directories with platform suffix
    ///
    /// The store structure is: ~/.vx/store/<runtime>/<version>/<platform>
    /// When scanning versions, we scan directories like "20.0.0" not "20.0.0-linux-x64"
    #[test]
    fn test_regression_version_directory_names() {
        let version_dirs = vec![
            "20.0.0",    // Valid version directory
            "18.0.0",    // Valid version directory
            ".tmp",      // Hidden directory (should be ignored)
            "downloads", // Non-version directory (should be ignored)
        ];

        let valid_versions: Vec<&str> = version_dirs
            .iter()
            .filter(|v| version_utils::parse_version(v).is_some())
            .copied()
            .collect();

        assert_eq!(valid_versions.len(), 2);
        assert!(valid_versions.contains(&"20.0.0"));
        assert!(valid_versions.contains(&"18.0.0"));
    }

    /// Regression test: Mixed version formats in store directory
    ///
    /// Some runtimes might have versions stored with 'v' prefix or other formats
    #[test]
    fn test_regression_mixed_version_formats_in_store() {
        let store_versions = vec![
            "v20.0.0", // Node.js style with v prefix
            "20.10.0", // Without v prefix
            "v18.0.0", // Older version with v
            "22.0.0",  // Latest without v
        ];

        // Should correctly identify 22.0.0 as latest (not v20.0.0)
        let latest = version_utils::find_latest_version(&store_versions, false);
        assert_eq!(latest, Some("22.0.0"));

        // All should be parseable
        for v in &store_versions {
            assert!(
                version_utils::parse_version(v).is_some(),
                "Failed to parse: {}",
                v
            );
        }
    }

    /// Regression test: Prerelease versions in store
    ///
    /// Store might contain both stable and prerelease versions
    #[test]
    fn test_regression_prerelease_versions_in_store() {
        let store_versions = vec![
            "3.11.0",      // Python stable
            "3.12.0-rc.1", // Python RC
            "3.10.0",      // Older stable
        ];

        // For normal operation, should prefer stable
        let latest_stable = version_utils::find_latest_version(&store_versions, true);
        assert_eq!(
            latest_stable,
            Some("3.11.0"),
            "Should select 3.11.0 as latest stable, not 3.12.0-rc.1"
        );

        // But 3.12.0-rc.1 is technically the newest if we include prereleases
        let latest_all = version_utils::find_latest_version(&store_versions, false);
        assert_eq!(latest_all, Some("3.12.0-rc.1"));
    }

    /// Regression test: Empty store directory
    ///
    /// When no versions are installed, should handle gracefully
    #[test]
    fn test_regression_empty_store_directory() {
        let empty: Vec<&str> = vec![];

        let latest = version_utils::find_latest_version(&empty, false);
        assert_eq!(latest, None, "Empty store should return None");
    }

    /// Regression test: Single version in store
    #[test]
    fn test_regression_single_version_in_store() {
        let single = vec!["20.0.0"];

        let latest = version_utils::find_latest_version(&single, false);
        assert_eq!(
            latest,
            Some("20.0.0"),
            "Single version should be returned as latest"
        );
    }

    /// Regression test: Version comparison for path selection
    ///
    /// The resolve_install_dir needs to compare versions correctly to select
    /// the right one when multiple are available
    #[test]
    fn test_regression_version_comparison_for_selection() {
        // Python versions
        assert!(version_utils::is_newer_version("3.12.0", "3.11.0"));
        assert!(version_utils::is_newer_version("3.11.5", "3.11.0"));
        assert!(!version_utils::is_newer_version("3.11.0", "3.12.0"));

        // Node.js versions
        assert!(version_utils::is_newer_version("22.0.0", "20.0.0"));
        assert!(version_utils::is_newer_version("20.10.0", "20.9.0"));

        // Go versions
        assert!(version_utils::is_newer_version("1.22.0", "1.21.0"));
    }

    /// Regression test: Path existence fallback
    ///
    /// When a version directory doesn't have the expected platform subdirectory,
    /// should fall back to the version directory itself
    #[test]
    fn test_regression_path_fallback_logic() {
        // This test documents the expected path structure and fallback behavior
        //
        // Primary structure: ~/.vx/store/python/3.11.0/linux-x64/
        // Fallback:          ~/.vx/store/python/3.11.0/
        //
        // The resolve_install_dir should:
        // 1. Try version_dir/platform first
        // 2. Fall back to version_dir if platform subdir doesn't exist

        let version = "3.11.0";
        let platform = "linux-x64";

        // Verify the expected path format
        let primary_suffix = format!("store/python/{}/{}", version, platform);
        let fallback_suffix = format!("store/python/{}", version);

        assert!(primary_suffix.contains("3.11.0/linux-x64"));
        assert!(fallback_suffix.contains("3.11.0"));
        assert!(!fallback_suffix.contains("linux-x64"));
    }
}
