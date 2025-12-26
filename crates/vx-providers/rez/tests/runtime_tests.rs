//! Unit tests for the Rez provider
//!
//! Tests are placed in a separate tests/ directory following project conventions.

use rstest::rstest;
use vx_provider_rez::{create_provider, RezProvider, RezRuntime};
use vx_runtime::{Arch, Ecosystem, InstallMethod, Os, PackageRuntime, Platform, Provider, Runtime};

// ============================================================================
// Provider Tests
// ============================================================================

#[test]
fn test_provider_name() {
    let provider = RezProvider::new();
    assert_eq!(provider.name(), "rez");
}

#[test]
fn test_provider_description() {
    let provider = RezProvider::new();
    assert!(!provider.description().is_empty());
    assert!(provider.description().contains("Rez") || provider.description().contains("rez"));
}

#[test]
fn test_provider_supports() {
    let provider = RezProvider::new();
    assert!(provider.supports("rez"));
    assert!(provider.supports("rez-env"));
    assert!(provider.supports("rez-build"));
    assert!(provider.supports("rez-release"));
    assert!(!provider.supports("python"));
    assert!(!provider.supports("pip"));
}

#[test]
fn test_provider_runtimes() {
    let provider = RezProvider::new();
    let runtimes = provider.runtimes();
    assert_eq!(runtimes.len(), 1);
    assert_eq!(runtimes[0].name(), "rez");
}

#[test]
fn test_provider_get_runtime() {
    let provider = RezProvider::new();
    assert!(provider.get_runtime("rez").is_some());
    assert!(provider.get_runtime("python").is_none());
}

#[test]
fn test_create_provider() {
    let provider = create_provider();
    assert_eq!(provider.name(), "rez");
}

// ============================================================================
// Runtime Tests
// ============================================================================

#[test]
fn test_runtime_name() {
    let runtime = RezRuntime::new();
    assert_eq!(runtime.name(), "rez");
}

#[test]
fn test_runtime_description() {
    let runtime = RezRuntime::new();
    let desc = runtime.description();
    assert!(!desc.is_empty());
    assert!(desc.contains("Rez") || desc.contains("rez"));
}

#[test]
fn test_runtime_aliases() {
    let runtime = RezRuntime::new();
    let aliases = runtime.aliases();
    assert!(!aliases.is_empty());
    assert!(aliases.contains(&"rez-env"));
    assert!(aliases.contains(&"rez-build"));
    assert!(aliases.contains(&"rez-release"));
}

#[test]
fn test_runtime_ecosystem() {
    let runtime = RezRuntime::new();
    assert_eq!(runtime.ecosystem(), Ecosystem::Python);
}

#[test]
fn test_runtime_metadata() {
    let runtime = RezRuntime::new();
    let meta = runtime.metadata();
    assert!(!meta.is_empty());
    assert!(meta.contains_key("homepage"));
    assert!(meta.contains_key("repository"));
    assert!(meta.contains_key("install_method"));
    assert_eq!(meta.get("install_method").unwrap(), "pip");
    assert_eq!(meta.get("pip_package").unwrap(), "rez");
    assert!(meta
        .get("repository")
        .unwrap()
        .contains("AcademySoftwareFoundation"));
}

// ============================================================================
// PackageRuntime Tests
// ============================================================================

#[test]
fn test_install_method() {
    let runtime = RezRuntime::new();
    let method = runtime.install_method();
    match method {
        InstallMethod::PipPackage { package_name, .. } => {
            assert_eq!(package_name, "rez");
        }
        _ => panic!("Expected InstallMethod::PipPackage"),
    }
}

#[test]
fn test_required_runtime() {
    let runtime = RezRuntime::new();
    assert_eq!(runtime.required_runtime(), "uv");
}

#[test]
fn test_required_runtime_version() {
    let runtime = RezRuntime::new();
    assert!(runtime.required_runtime_version().is_none());
}

// ============================================================================
// Executable Path Tests
// ============================================================================

#[rstest]
#[case(Os::Windows, Arch::X86_64, "venv/Scripts/rez.exe")]
#[case(Os::Windows, Arch::Aarch64, "venv/Scripts/rez.exe")]
#[case(Os::MacOS, Arch::X86_64, "venv/bin/rez")]
#[case(Os::MacOS, Arch::Aarch64, "venv/bin/rez")]
#[case(Os::Linux, Arch::X86_64, "venv/bin/rez")]
#[case(Os::Linux, Arch::Aarch64, "venv/bin/rez")]
fn test_executable_relative_path(#[case] os: Os, #[case] arch: Arch, #[case] expected: &str) {
    let runtime = RezRuntime::new();
    let platform = Platform::new(os, arch);
    let path = runtime.executable_relative_path("2.114.0", &platform);
    assert_eq!(path, expected);
}

// ============================================================================
// Platform Support Tests
// ============================================================================

#[rstest]
#[case(Os::Windows, Arch::X86_64)]
#[case(Os::Windows, Arch::Aarch64)]
#[case(Os::MacOS, Arch::X86_64)]
#[case(Os::MacOS, Arch::Aarch64)]
#[case(Os::Linux, Arch::X86_64)]
#[case(Os::Linux, Arch::Aarch64)]
fn test_platform_support(#[case] os: Os, #[case] arch: Arch) {
    let runtime = RezRuntime::new();
    let platform = Platform::new(os, arch);
    // pip packages should work on all platforms
    let path = runtime.executable_relative_path("2.114.0", &platform);
    assert!(!path.is_empty());
}
