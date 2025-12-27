//! Tests for release-please runtime

use vx_provider_release_please::{ReleasePleaseProvider, ReleasePleaseRuntime};
use vx_runtime::{Ecosystem, Os, Platform, Provider, Runtime};

#[test]
fn test_runtime_name() {
    let runtime = ReleasePleaseRuntime::new();
    assert_eq!(runtime.name(), "release-please");
}

#[test]
fn test_runtime_description() {
    let runtime = ReleasePleaseRuntime::new();
    assert!(runtime.description().contains("release-please"));
}

#[test]
fn test_runtime_ecosystem() {
    let runtime = ReleasePleaseRuntime::new();
    assert_eq!(runtime.ecosystem(), Ecosystem::NodeJs);
}

#[test]
fn test_runtime_metadata() {
    let runtime = ReleasePleaseRuntime::new();
    let meta = runtime.metadata();
    assert!(meta.contains_key("homepage"));
    assert!(meta.contains_key("install_method"));
    assert_eq!(meta.get("install_method").unwrap(), "npm");
}

#[test]
fn test_provider_name() {
    let provider = ReleasePleaseProvider::new();
    assert_eq!(provider.name(), "release-please");
}

#[test]
fn test_provider_supports() {
    let provider = ReleasePleaseProvider::new();
    assert!(provider.supports("release-please"));
    assert!(!provider.supports("other"));
}

#[test]
fn test_provider_runtimes() {
    let provider = ReleasePleaseProvider::new();
    let runtimes = provider.runtimes();
    assert_eq!(runtimes.len(), 1);
    assert_eq!(runtimes[0].name(), "release-please");
}

#[test]
fn test_provider_get_runtime() {
    let provider = ReleasePleaseProvider::new();
    assert!(provider.get_runtime("release-please").is_some());
    assert!(provider.get_runtime("other").is_none());
}

#[test]
fn test_executable_relative_path_linux() {
    let runtime = ReleasePleaseRuntime::new();
    let platform = Platform {
        os: Os::Linux,
        arch: vx_runtime::Arch::X86_64,
    };
    assert_eq!(
        runtime.executable_relative_path("17.0.0", &platform),
        "bin/release-please"
    );
}

#[test]
fn test_executable_relative_path_windows() {
    let runtime = ReleasePleaseRuntime::new();
    let platform = Platform {
        os: Os::Windows,
        arch: vx_runtime::Arch::X86_64,
    };
    assert_eq!(
        runtime.executable_relative_path("17.0.0", &platform),
        "bin/release-please.cmd"
    );
}

#[test]
fn test_executable_relative_path_macos() {
    let runtime = ReleasePleaseRuntime::new();
    let platform = Platform {
        os: Os::MacOS,
        arch: vx_runtime::Arch::Aarch64,
    };
    assert_eq!(
        runtime.executable_relative_path("17.0.0", &platform),
        "bin/release-please"
    );
}
