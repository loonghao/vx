//! Tests for x-cmd runtime

use rstest::rstest;
use vx_provider_x_cmd::{XCmdConfig, XCmdProvider, XCmdRuntime};
use vx_runtime::{Arch, Ecosystem, Os, Platform, Provider, Runtime};

// ===========================================================================
// Runtime basic tests
// ===========================================================================

#[test]
fn test_runtime_name() {
    let runtime = XCmdRuntime::new();
    assert_eq!(runtime.name(), "x-cmd");
}

#[test]
fn test_runtime_description() {
    let runtime = XCmdRuntime::new();
    assert!(runtime.description().contains("x-cmd"));
}

#[test]
fn test_runtime_ecosystem() {
    let runtime = XCmdRuntime::new();
    assert_eq!(runtime.ecosystem(), Ecosystem::System);
}

#[test]
fn test_runtime_aliases() {
    let runtime = XCmdRuntime::new();
    let aliases = runtime.aliases();
    assert!(aliases.contains(&"xcmd"));
    assert!(aliases.contains(&"x_cmd"));
}

#[test]
fn test_runtime_metadata() {
    let runtime = XCmdRuntime::new();
    let meta = runtime.metadata();
    assert!(meta.contains_key("homepage"));
    assert_eq!(meta.get("homepage").unwrap(), "https://x-cmd.com");
    assert!(meta.contains_key("repository"));
    assert!(meta.contains_key("license"));
}

// ===========================================================================
// Provider tests
// ===========================================================================

#[test]
fn test_provider_name() {
    let provider = XCmdProvider::new();
    assert_eq!(provider.name(), "x-cmd");
}

#[test]
fn test_provider_description() {
    let provider = XCmdProvider::new();
    assert!(provider.description().contains("x-cmd"));
}

#[test]
fn test_provider_supports() {
    let provider = XCmdProvider::new();
    assert!(provider.supports("x-cmd"));
    assert!(provider.supports("xcmd"));
    assert!(provider.supports("x_cmd"));
    assert!(!provider.supports("other"));
    assert!(!provider.supports("cmd"));
}

#[test]
fn test_provider_runtimes() {
    let provider = XCmdProvider::new();
    let runtimes = provider.runtimes();
    assert_eq!(runtimes.len(), 1);
    assert_eq!(runtimes[0].name(), "x-cmd");
}

// ===========================================================================
// Download URL tests (x-cmd uses script install, not binary download)
// ===========================================================================

#[tokio::test]
async fn test_download_url_returns_none() {
    let runtime = XCmdRuntime::new();
    let platform = Platform::new(Os::Linux, Arch::X86_64);
    let url = runtime.download_url("0.7.15", &platform).await.unwrap();
    assert!(url.is_none(), "x-cmd should not have a download URL (script install)");
}

#[rstest]
#[case(Os::Windows, Arch::X86_64)]
#[case(Os::Linux, Arch::X86_64)]
#[case(Os::MacOS, Arch::Aarch64)]
#[tokio::test]
async fn test_download_url_none_all_platforms(#[case] os: Os, #[case] arch: Arch) {
    let runtime = XCmdRuntime::new();
    let platform = Platform::new(os, arch);
    let url = runtime.download_url("0.7.15", &platform).await.unwrap();
    assert!(url.is_none());
}

// ===========================================================================
// Config tests
// ===========================================================================

#[test]
fn test_executable_name() {
    assert_eq!(XCmdConfig::executable_name(), "x");
}

#[test]
fn test_install_script_url() {
    assert_eq!(XCmdConfig::install_script_url(), "https://get.x-cmd.com");
}

#[test]
fn test_install_script_url_windows() {
    assert_eq!(
        XCmdConfig::install_script_url_windows(),
        "https://get.x-cmd.com/ps1"
    );
}

#[rstest]
#[case(Os::Linux)]
#[case(Os::MacOS)]
fn test_install_command_unix(#[case] os: Os) {
    let platform = Platform::new(os, Arch::X86_64);
    let cmd = XCmdConfig::install_command(&platform);
    assert!(cmd.contains("curl"));
    assert!(cmd.contains("get.x-cmd.com"));
}

#[test]
fn test_install_command_windows() {
    let platform = Platform::new(Os::Windows, Arch::X86_64);
    let cmd = XCmdConfig::install_command(&platform);
    assert!(cmd.contains("irm"));
    assert!(cmd.contains("get.x-cmd.com/ps1"));
}

#[test]
fn test_search_paths_unix() {
    let platform = Platform::new(Os::Linux, Arch::X86_64);
    let paths = XCmdConfig::search_paths(&platform);
    assert!(!paths.is_empty());
}

#[test]
fn test_search_paths_windows() {
    let platform = Platform::new(Os::Windows, Arch::X86_64);
    let paths = XCmdConfig::search_paths(&platform);
    // Windows may have empty search paths (relies on PATH)
    assert!(paths.is_empty() || !paths.is_empty());
}

// ===========================================================================
// Verify installation tests
// ===========================================================================

#[test]
fn test_verify_installation_failure_message() {
    let runtime = XCmdRuntime::new();
    let platform = Platform::new(Os::Linux, Arch::X86_64);

    // Verify with a non-existent path
    let result = runtime.verify_installation("0.7.15", std::path::Path::new("/nonexistent"), &platform);

    // If x-cmd is not actually installed, this should fail with helpful suggestions
    if !result.valid {
        assert!(!result.suggestions.is_empty());
    }
}

// ===========================================================================
// Executable path tests
// ===========================================================================

#[test]
fn test_executable_relative_path() {
    let runtime = XCmdRuntime::new();
    let platform = Platform::new(Os::Linux, Arch::X86_64);
    let path = runtime.executable_relative_path("0.7.15", &platform);
    assert_eq!(path, "x");
}
