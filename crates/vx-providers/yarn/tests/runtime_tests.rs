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

/// Test that Yarn has Node.js dependency with version constraints
#[test]
fn test_yarn_dependencies() {
    let runtime = YarnRuntime::new();
    let deps = runtime.dependencies();

    assert_eq!(deps.len(), 1, "Yarn should have exactly one dependency");

    let node_dep = &deps[0];
    assert_eq!(node_dep.name, "node");
    assert!(!node_dep.optional);

    // Check version constraints
    assert_eq!(node_dep.min_version, Some("12.0.0".to_string()));
    assert_eq!(node_dep.max_version, Some("22.99.99".to_string()));
    assert_eq!(node_dep.recommended_version, Some("20".to_string()));
}

/// Test Yarn's Node.js version compatibility
#[test]
fn test_yarn_node_version_compatibility() {
    let runtime = YarnRuntime::new();
    let deps = runtime.dependencies();
    let node_dep = &deps[0];

    // Compatible versions
    assert!(node_dep.is_version_compatible("20.10.0"));
    assert!(node_dep.is_version_compatible("18.19.0"));
    assert!(node_dep.is_version_compatible("22.0.0"));
    assert!(node_dep.is_version_compatible("12.0.0"));

    // Incompatible versions (Node.js 23+ has native module compilation issues)
    assert!(!node_dep.is_version_compatible("23.0.0"));
    assert!(!node_dep.is_version_compatible("23.11.0"));
    assert!(!node_dep.is_version_compatible("25.2.1"));
    assert!(!node_dep.is_version_compatible("11.0.0"));
}

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
