//! Tests for vcpkg provider

use rstest::rstest;
use vx_provider_vcpkg::{VcpkgProvider, VcpkgRuntime, native_packages};
use vx_runtime::{Ecosystem, Provider, Runtime};

#[test]
fn test_provider_creation() {
    let provider = VcpkgProvider::new();
    assert_eq!(provider.name(), "vcpkg");
    assert!(provider.description().contains("C++"));
}

#[test]
fn test_provider_supports() {
    let provider = VcpkgProvider::new();
    assert!(provider.supports("vcpkg"));
    assert!(!provider.supports("npm"));
}

#[test]
fn test_runtime_creation() {
    let runtime = VcpkgRuntime::new();
    assert_eq!(runtime.name(), "vcpkg");
    assert_eq!(runtime.executable_name(), "vcpkg");
}

#[test]
fn test_runtime_aliases() {
    let runtime = VcpkgRuntime::new();
    let aliases = runtime.aliases();
    assert!(aliases.contains(&"vcpkg-cli"));
}

#[test]
fn test_runtime_ecosystem() {
    let runtime = VcpkgRuntime::new();
    let ecosystem = runtime.ecosystem();
    assert!(matches!(ecosystem, Ecosystem::Custom(name) if name == "cpp"));
}

#[test]
fn test_runtime_metadata() {
    let runtime = VcpkgRuntime::new();
    let metadata = runtime.metadata();

    assert_eq!(
        metadata.get("homepage"),
        Some(&"https://vcpkg.io/".to_string())
    );
    assert_eq!(metadata.get("language"), Some(&"C++".to_string()));
    assert_eq!(
        metadata.get("category"),
        Some(&"package-manager".to_string())
    );
    assert!(metadata.contains_key("default_triplet"));
}

#[rstest]
#[case("x64-windows")]
#[case("x64-linux")]
#[case("x64-osx")]
#[case("arm64-windows")]
#[case("arm64-linux")]
#[case("arm64-osx")]
fn test_triplet_format(#[case] triplet: &str) {
    // Verify triplet format is correct
    let parts: Vec<&str> = triplet.split('-').collect();
    assert_eq!(parts.len(), 2, "Triplet should have format arch-os");
}

#[test]
fn test_native_packages_constants() {
    assert_eq!(native_packages::WINPTY, "winpty");
    assert_eq!(native_packages::SQLITE3, "sqlite3");
    assert_eq!(native_packages::OPENSSL, "openssl");
}

#[test]
fn test_supported_platforms() {
    let runtime = VcpkgRuntime::new();
    let platforms = runtime.supported_platforms();

    // vcpkg should support at least Windows, Linux, macOS
    assert!(!platforms.is_empty());
}
