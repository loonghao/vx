//! Unit tests for the pre-commit provider
//!
//! Tests are placed in a separate tests/ directory following project conventions.

use rstest::rstest;
use vx_provider_pre_commit::{PreCommitProvider, PreCommitRuntime, create_provider};
use vx_runtime::{Arch, Ecosystem, InstallMethod, Os, PackageRuntime, Platform, Provider, Runtime};

// ============================================================================
// Provider Tests
// ============================================================================

#[test]
fn test_provider_name() {
    let provider = PreCommitProvider::new();
    assert_eq!(provider.name(), "pre-commit");
}

#[test]
fn test_provider_description() {
    let provider = PreCommitProvider::new();
    assert!(!provider.description().is_empty());
    assert!(provider.description().contains("pre-commit"));
}

#[test]
fn test_provider_supports() {
    let provider = PreCommitProvider::new();
    assert!(provider.supports("pre-commit"));
    assert!(!provider.supports("git"));
    assert!(!provider.supports("python"));
}

#[test]
fn test_provider_runtimes() {
    let provider = PreCommitProvider::new();
    let runtimes = provider.runtimes();
    assert_eq!(runtimes.len(), 1);
    assert_eq!(runtimes[0].name(), "pre-commit");
}

#[test]
fn test_provider_get_runtime() {
    let provider = PreCommitProvider::new();
    assert!(provider.get_runtime("pre-commit").is_some());
    assert!(provider.get_runtime("git").is_none());
}

#[test]
fn test_create_provider() {
    let provider = create_provider();
    assert_eq!(provider.name(), "pre-commit");
}

// ============================================================================
// Runtime Tests
// ============================================================================

#[test]
fn test_runtime_name() {
    let runtime = PreCommitRuntime::new();
    assert_eq!(runtime.name(), "pre-commit");
}

#[test]
fn test_runtime_description() {
    let runtime = PreCommitRuntime::new();
    let desc = runtime.description();
    assert!(!desc.is_empty());
    assert!(desc.contains("pre-commit"));
}

#[test]
fn test_runtime_aliases() {
    let runtime = PreCommitRuntime::new();
    let aliases = runtime.aliases();
    assert!(aliases.is_empty());
}

#[test]
fn test_runtime_ecosystem() {
    let runtime = PreCommitRuntime::new();
    assert_eq!(runtime.ecosystem(), Ecosystem::Python);
}

#[test]
fn test_runtime_metadata() {
    let runtime = PreCommitRuntime::new();
    let meta = runtime.metadata();
    assert!(!meta.is_empty());
    assert!(meta.contains_key("homepage"));
    assert!(meta.contains_key("repository"));
    assert!(meta.contains_key("install_method"));
    assert_eq!(meta.get("install_method").unwrap(), "pip");
    assert_eq!(meta.get("pip_package").unwrap(), "pre-commit");
}

// ============================================================================
// PackageRuntime Tests
// ============================================================================

#[test]
fn test_install_method() {
    let runtime = PreCommitRuntime::new();
    let method = runtime.install_method();
    match method {
        InstallMethod::PipPackage { package_name, .. } => {
            assert_eq!(package_name, "pre-commit");
        }
        _ => panic!("Expected InstallMethod::PipPackage"),
    }
}

#[test]
fn test_required_runtime() {
    let runtime = PreCommitRuntime::new();
    assert_eq!(runtime.required_runtime(), "uv");
}

#[test]
fn test_required_runtime_version() {
    let runtime = PreCommitRuntime::new();
    assert!(runtime.required_runtime_version().is_none());
}

// ============================================================================
// Executable Path Tests
// ============================================================================

#[rstest]
#[case(Os::Windows, Arch::X86_64, "venv/Scripts/pre-commit.exe")]
#[case(Os::Windows, Arch::Aarch64, "venv/Scripts/pre-commit.exe")]
#[case(Os::MacOS, Arch::X86_64, "venv/bin/pre-commit")]
#[case(Os::MacOS, Arch::Aarch64, "venv/bin/pre-commit")]
#[case(Os::Linux, Arch::X86_64, "venv/bin/pre-commit")]
#[case(Os::Linux, Arch::Aarch64, "venv/bin/pre-commit")]
fn test_executable_relative_path(#[case] os: Os, #[case] arch: Arch, #[case] expected: &str) {
    let runtime = PreCommitRuntime::new();
    let platform = Platform::new(os, arch);
    let path = runtime.executable_relative_path("3.7.0", &platform);
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
    let runtime = PreCommitRuntime::new();
    let platform = Platform::new(os, arch);
    // pip packages should work on all platforms
    let path = runtime.executable_relative_path("3.7.0", &platform);
    assert!(!path.is_empty());
}
