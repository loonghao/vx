//! Tests for GitHub CLI runtime

use rstest::*;
use vx_runtime::{Arch, Os, Platform, Runtime};

use vx_provider_gh::{GitHubRuntime, GitHubUrlBuilder};

#[test]
fn test_runtime_name() {
    let runtime = GitHubRuntime::new();
    assert_eq!(runtime.name(), "gh");
}

#[test]
fn test_runtime_description() {
    let runtime = GitHubRuntime::new();
    assert!(runtime.description().contains("GitHub CLI"));
}

#[test]
fn test_runtime_aliases() {
    let runtime = GitHubRuntime::new();
    assert!(runtime.aliases().contains(&"github"));
}

#[test]
fn test_runtime_ecosystem() {
    let runtime = GitHubRuntime::new();
    assert_eq!(format!("{:?}", runtime.ecosystem()), "System");
}

#[rstest]
#[case("2.45.0", Platform::new(Os::Windows, Arch::X86_64),
    "https://github.com/cli/cli/releases/download/v2.45.0/gh_2.45.0_windows_amd64.zip")]
#[case("2.44.0", Platform::new(Os::Linux, Arch::X86_64),
    "https://github.com/cli/cli/releases/download/v2.44.0/gh_2.44.0_linux_amd64.tar.gz")]
#[case("2.43.0", Platform::new(Os::MacOS, Arch::X86_64),
    "https://github.com/cli/cli/releases/download/v2.43.0/gh_2.43.0_macOS_amd64.zip")]
#[case("2.42.0", Platform::new(Os::MacOS, Arch::Aarch64),
    "https://github.com/cli/cli/releases/download/v2.42.0/gh_2.42.0_macOS_arm64.zip")]
fn test_url_builder(#[case] version: &str, #[case] platform: Platform, #[case] expected_url: &str) {
    let url = GitHubUrlBuilder::download_url(version, &platform);
    assert_eq!(url, expected_url);
}

#[rstest]
#[case(Platform::new(Os::Windows, Arch::X86_64), "2.45.0", "bin/gh.exe")]
#[case(Platform::new(Os::Linux, Arch::X86_64), "2.44.0", "gh_2.44.0_linux_amd64/bin/gh")]
#[case(Platform::new(Os::MacOS, Arch::X86_64), "2.43.0", "gh_2.43.0_macOS_amd64/bin/gh")]
fn test_executable_relative_path(
    #[case] platform: Platform,
    #[case] version: &str,
    #[case] expected_path: &str,
) {
    let runtime = GitHubRuntime::new();
    let path = runtime.executable_relative_path(version, &platform);
    assert_eq!(path, expected_path);
}
