//! Yarn runtime tests

use rstest::rstest;
use vx_provider_yarn::{YarnProvider, YarnRuntime};
use vx_runtime::{Arch, Ecosystem, Os, Platform, Provider, Runtime};

#[rstest]
fn test_yarn_runtime_name() {
    let runtime = YarnRuntime::new();
    assert_eq!(runtime.name(), "yarn");
}

#[rstest]
fn test_yarn_runtime_ecosystem() {
    let runtime = YarnRuntime::new();
    assert_eq!(runtime.ecosystem(), Ecosystem::NodeJs);
}

#[rstest]
fn test_yarn_runtime_description() {
    let runtime = YarnRuntime::new();
    assert_eq!(
        runtime.description(),
        "Fast, reliable, and secure dependency management"
    );
}

#[rstest]
fn test_yarn_provider_name() {
    let provider = YarnProvider::new();
    assert_eq!(provider.name(), "yarn");
}

#[rstest]
fn test_yarn_provider_runtimes() {
    let provider = YarnProvider::new();
    let runtimes = provider.runtimes();
    assert_eq!(runtimes.len(), 1);
    assert_eq!(runtimes[0].name(), "yarn");
}

#[rstest]
fn test_yarn_provider_supports() {
    let provider = YarnProvider::new();
    assert!(provider.supports("yarn"));
    assert!(!provider.supports("npm"));
}

#[rstest]
fn test_yarn_provider_get_runtime() {
    let provider = YarnProvider::new();

    let yarn = provider.get_runtime("yarn");
    assert!(yarn.is_some());
    assert_eq!(yarn.unwrap().name(), "yarn");

    let unknown = provider.get_runtime("unknown");
    assert!(unknown.is_none());
}

// ============================================================================
// Dependency tests
// ============================================================================
// Note: Yarn's dependencies are now managed by vx_runtime::ConstraintsRegistry
// which provides version-aware constraints loaded from provider.toml:
// - Yarn 1.x: Node.js 12-22 (native module compatibility)
// - Yarn 2.x-3.x: Node.js 16+
// - Yarn 4.x: Node.js 18+
//
// See crates/vx-providers/yarn/provider.toml for constraint definitions.
// The ConstraintsRegistry tests are in crates/vx-runtime/tests/constraints_tests.rs

// ============================================================================
// Executable path tests
// ============================================================================

/// Test that Yarn executable path is correct for all platforms
/// Yarn archives extract to `yarn-v{version}/bin/` on all platforms
/// Windows uses .cmd extension
#[rstest]
#[case("1.22.19", Os::Windows, Arch::X86_64, "yarn-v1.22.19/bin/yarn.cmd")]
#[case("1.22.19", Os::Windows, Arch::Aarch64, "yarn-v1.22.19/bin/yarn.cmd")]
#[case("1.22.19", Os::Linux, Arch::X86_64, "yarn-v1.22.19/bin/yarn")]
#[case("1.22.19", Os::MacOS, Arch::Aarch64, "yarn-v1.22.19/bin/yarn")]
fn test_yarn_executable_relative_path(
    #[case] version: &str,
    #[case] os: Os,
    #[case] arch: Arch,
    #[case] expected: &str,
) {
    let runtime = YarnRuntime::new();
    let platform = Platform::new(os, arch);
    let path = runtime.executable_relative_path(version, &platform);
    assert_eq!(path, expected);
}

/// Test executable_extensions returns .cmd first for Windows
#[test]
fn test_yarn_executable_extensions() {
    let runtime = YarnRuntime::new();
    let extensions = runtime.executable_extensions();
    assert_eq!(extensions, &[".cmd", ".exe"]);
}

/// Test executable_name returns "yarn"
#[test]
fn test_yarn_executable_name() {
    let runtime = YarnRuntime::new();
    assert_eq!(runtime.executable_name(), "yarn");
}

// ============================================================================
// Yarn 2.x+ (Berry) handling tests
// ============================================================================

/// Test that Yarn 2.x+ is not directly installable
#[rstest]
#[case("2.4.3", false)]
#[case("3.6.0", false)]
#[case("4.0.0", false)]
#[case("1.22.19", true)]
#[case("1.0.0", true)]
fn test_yarn_version_installable(#[case] version: &str, #[case] expected_installable: bool) {
    use vx_provider_yarn::YarnUrlBuilder;
    let is_installable = YarnUrlBuilder::is_directly_installable(version);
    assert_eq!(
        is_installable, expected_installable,
        "Yarn {} should {}be directly installable",
        version,
        if expected_installable { "" } else { "not " }
    );
}

/// Test that Yarn 2.x+ returns None for download_url
#[rstest]
#[case("1.22.19", true)]
#[case("2.4.3", false)]
#[case("3.6.0", false)]
fn test_yarn_download_url(#[case] version: &str, #[case] should_have_url: bool) {
    use vx_provider_yarn::YarnUrlBuilder;
    let url = YarnUrlBuilder::download_url(version);
    if should_have_url {
        assert!(
            url.is_some(),
            "Yarn {} should have a download URL",
            version
        );
        assert!(url.unwrap().contains("github.com"));
    } else {
        assert!(
            url.is_none(),
            "Yarn {} should NOT have a direct download URL (requires corepack)",
            version
        );
    }
}

/// Test executable_dir_path returns None for Yarn 2.x+
#[rstest]
#[case("1.22.19", Some("yarn-v1.22.19/bin"))]
#[case("2.4.3", None)]
#[case("3.6.0", None)]
fn test_yarn_executable_dir_path(
    #[case] version: &str,
    #[case] expected: Option<&str>,
) {
    let runtime = YarnRuntime::new();
    let platform = Platform::new(Os::Linux, Arch::X86_64);
    let path = runtime.executable_dir_path(version, &platform);
    assert_eq!(path.as_deref(), expected);
}

// ============================================================================
// RFC 0028: Proxy-Managed Runtimes Tests
// ============================================================================

/// Test that `is_version_installable` returns correct values for different yarn versions
/// - Yarn 1.x: true (directly installable via GitHub releases)
/// - Yarn 2.x+: false (requires corepack)
#[rstest]
#[case("1.22.19", true)]
#[case("1.22.0", true)]
#[case("1.0.0", true)]
#[case("2.0.0", false)]
#[case("2.4.3", false)]
#[case("3.0.0", false)]
#[case("3.6.0", false)]
#[case("4.0.0", false)]
#[case("4.5.1", false)]
fn test_yarn_is_version_installable(#[case] version: &str, #[case] expected: bool) {
    let runtime = YarnRuntime::new();
    let result = runtime.is_version_installable(version);
    assert_eq!(
        result, expected,
        "is_version_installable({}) should return {}",
        version, expected
    );
}

/// Test that YarnRuntime::uses_corepack correctly identifies versions
#[rstest]
#[case("1.22.19", false)]
#[case("2.4.3", true)]
#[case("3.6.0", true)]
#[case("4.0.0", true)]
fn test_yarn_uses_corepack(#[case] version: &str, #[case] expected: bool) {
    assert_eq!(
        YarnRuntime::uses_corepack(version),
        expected,
        "uses_corepack({}) should return {}",
        version,
        expected
    );
}

/// Test that version metadata correctly identifies install method
#[tokio::test]
async fn test_yarn_fetch_versions_metadata() {
    // This test would require a mock RuntimeContext
    // For now, we test the static logic
    
    // Yarn 1.x should have "direct" install method
    let v1_installable = YarnRuntime::new().is_version_installable("1.22.19");
    assert!(v1_installable, "Yarn 1.x should be directly installable");
    
    // Yarn 2.x+ should have "corepack" install method
    let v2_installable = YarnRuntime::new().is_version_installable("2.4.3");
    assert!(!v2_installable, "Yarn 2.x should NOT be directly installable");
    
    let v4_installable = YarnRuntime::new().is_version_installable("4.0.0");
    assert!(!v4_installable, "Yarn 4.x should NOT be directly installable");
}
