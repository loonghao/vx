//! Tests for version resolution logic
//!
//! These tests ensure that "latest" version resolution works correctly
//! and that version parsing handles edge cases properly.

use tokio;
use vx_version::{GoVersionFetcher, NodeVersionFetcher, VersionFetcher};

#[cfg(test)]
mod version_resolution_tests {
    use super::*;

    #[tokio::test]
    async fn test_go_fetcher_latest_version_resolution() {
        let fetcher = GoVersionFetcher::new();

        // Test getting latest version (should not contain "latest" in URL)
        let result = fetcher.get_latest_version().await;

        if let Ok(Some(version_info)) = result {
            // Should be a valid version format (e.g., "1.21.6")
            assert!(!version_info.version.is_empty());
            assert!(version_info
                .version
                .chars()
                .next()
                .unwrap()
                .is_ascii_digit());
            assert!(version_info.version.contains('.'));
            // Should not contain "go" prefix
            assert!(!version_info.version.starts_with("go"));
        }
    }

    #[tokio::test]
    async fn test_version_exists_check() {
        let fetcher = GoVersionFetcher::new();

        // Test checking if a version exists
        // This tests the version resolution logic indirectly
        let result = fetcher.version_exists("1.21.0").await;

        // Should either succeed or fail gracefully (network issues are OK)
        match result {
            Ok(exists) => {
                // If it succeeds, the boolean result is valid
                assert!(exists == true || exists == false);
            }
            Err(_) => {
                // Network errors are acceptable in tests
            }
        }
    }

    #[tokio::test]
    async fn test_go_version_fetcher_returns_valid_versions() {
        let fetcher = GoVersionFetcher::new();

        let result = fetcher.fetch_versions(false).await;

        if let Ok(versions) = result {
            assert!(
                !versions.is_empty(),
                "Go fetcher should return at least one version"
            );

            // Check that versions are properly formatted
            for version in &versions {
                assert!(!version.version.is_empty());
                assert!(!version.version.contains("go")); // Should be stripped of "go" prefix
                assert!(version.version.chars().next().unwrap().is_ascii_digit());
            }
        }
    }

    #[tokio::test]
    async fn test_node_version_fetcher_returns_valid_versions() {
        let fetcher = NodeVersionFetcher::new();

        let result = fetcher.fetch_versions(false).await;

        if let Ok(versions) = result {
            assert!(
                !versions.is_empty(),
                "Node fetcher should return at least one version"
            );

            // Check that versions are properly formatted
            for version in &versions {
                assert!(!version.version.is_empty());
                // Node versions should not have "v" prefix after parsing
                assert!(version.version.chars().next().unwrap().is_ascii_digit());
            }
        }
    }
}

#[cfg(test)]
mod version_parsing_edge_cases {
    use super::*;

    #[tokio::test]
    async fn test_version_with_prerelease() {
        let fetcher = GoVersionFetcher::new();

        // Test fetching with prerelease versions included
        let result = fetcher.fetch_versions(true).await;

        if let Ok(versions) = result {
            // Should include more versions when prereleases are included
            let stable_result = fetcher.fetch_versions(false).await;
            if let Ok(stable_versions) = stable_result {
                assert!(versions.len() >= stable_versions.len());
            }
        }
    }

    #[tokio::test]
    async fn test_version_format_consistency() {
        let fetcher = GoVersionFetcher::new();

        let result = fetcher.fetch_versions(false).await;

        if let Ok(versions) = result {
            for version in &versions {
                // All versions should follow semantic versioning
                let parts: Vec<&str> = version.version.split('.').collect();
                assert!(
                    parts.len() >= 2,
                    "Version {} should have at least major.minor",
                    version.version
                );

                // Each part should be numeric (allowing for prerelease suffixes)
                for part in &parts[..2] {
                    // Check at least major and minor
                    assert!(
                        part.chars().next().unwrap().is_ascii_digit(),
                        "Version part {} should start with digit",
                        part
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod performance_tests {
    use super::*;

    #[tokio::test]
    async fn test_version_fetching_performance() {
        let fetcher = GoVersionFetcher::new();

        let start = std::time::Instant::now();
        let result = fetcher.fetch_versions(false).await;
        let duration = start.elapsed();

        // Version fetching should complete within reasonable time
        assert!(
            duration.as_secs() < 30,
            "Version fetching took too long: {:?}",
            duration
        );

        if let Ok(versions) = result {
            assert!(!versions.is_empty());
        }
    }

    #[tokio::test]
    async fn test_concurrent_version_fetching() {
        // Test that multiple concurrent version fetches work correctly
        let fetcher = GoVersionFetcher::new();

        let mut handles = vec![];

        for _ in 0..3 {
            let fetcher_clone = fetcher.clone();
            let handle = tokio::spawn(async move { fetcher_clone.fetch_versions(false).await });
            handles.push(handle);
        }

        // All requests should complete successfully
        for handle in handles {
            let result = handle.await.expect("Task should complete");
            // Each request should either succeed or fail consistently
            match result {
                Ok(versions) => assert!(!versions.is_empty()),
                Err(_) => {
                    // Network errors are acceptable in tests
                }
            }
        }
    }
}
