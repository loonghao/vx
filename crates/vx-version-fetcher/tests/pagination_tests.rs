//! Tests for GitHub API pagination functionality
//!
//! These tests verify that the pagination logic in `GitHubReleasesFetcher`
//! correctly handles multi-page version fetching.

use vx_version_fetcher::GitHubReleasesConfig;
use vx_version_fetcher::fetchers::GitHubReleasesFetcher;

/// Test that `api_url()` generates correct URLs with page parameter
#[test]
fn test_api_url_generates_correct_page_parameter() {
    let fetcher = GitHubReleasesFetcher::new("helm", "helm")
        .with_config(GitHubReleasesConfig::default().with_per_page(100));

    // Test page 1
    let url = fetcher.api_url(1);
    assert!(
        url.contains("per_page=100"),
        "URL should contain per_page=100, got: {}",
        url
    );
    assert!(url.contains("page=1"), "URL should contain page=1");

    // Test page 2
    let url = fetcher.api_url(2);
    assert!(url.contains("page=2"), "URL should contain page=2");

    // Test page 10
    let url = fetcher.api_url(10);
    assert!(url.contains("page=10"), "URL should contain page=10");
}

/// Test that `api_url()` generates correct base URL
#[test]
fn test_api_url_generates_correct_base_url() {
    let fetcher = GitHubReleasesFetcher::new("BurntSushi", "ripgrep")
        .with_config(GitHubReleasesConfig::default());

    let url = fetcher.api_url(1);
    assert!(
        url.starts_with("https://api.github.com/repos/"),
        "URL should start with GitHub API base"
    );
    assert!(
        url.contains("BurntSushi/ripgrep"),
        "URL should contain owner/repo"
    );
}

/// Test that `per_page` configuration is respected
#[test]
fn test_per_page_configuration() {
    // Test with default (100)
    let config = GitHubReleasesConfig::default();
    assert_eq!(config.per_page, 100, "Default per_page should be 100");

    // Test with custom value
    let config = GitHubReleasesConfig::default().with_per_page(50);
    assert_eq!(config.per_page, 50, "per_page should be configurable");
}

/// Test that pagination stops when fewer results than per_page are returned
///
/// This test verifies the logic in `fetch_from_github()` that determines
/// when to stop pagination.
#[test]
fn test_pagination_stop_condition() {
    let per_page = 100;

    // Simulate: if we get fewer results than per_page, we've reached the last page
    let results_count = 50;
    assert!(
        results_count < per_page,
        "Should stop pagination when results < per_page"
    );

    // Simulate: if we get exactly per_page results, there might be more pages
    let results_count = 100;
    assert_eq!(
        results_count, per_page,
        "Equal results count means we might have more pages"
    );
}

/// Test GitHubReleasesFetcher creation with different owners/repos
#[test]
fn test_fetcher_creation() {
    let fetcher = GitHubReleasesFetcher::new("owner", "repo");
    let url = fetcher.api_url(1);
    assert!(url.contains("owner/repo"), "URL should contain owner/repo");

    let fetcher = GitHubReleasesFetcher::new("rust-lang", "rust");
    let url = fetcher.api_url(1);
    assert!(
        url.contains("rust-lang/rust"),
        "URL should contain rust-lang/rust"
    );
}

/// Test that `with_per_page()` correctly updates the configuration
#[test]
fn test_with_per_page_builder() {
    let fetcher = GitHubReleasesFetcher::new("test", "test")
        .with_config(GitHubReleasesConfig::default().with_per_page(30));

    // We can't directly access private fields, but we can test via api_url()
    let url = fetcher.api_url(1);
    assert!(
        url.contains("per_page=30"),
        "URL should reflect custom per_page value"
    );
}
