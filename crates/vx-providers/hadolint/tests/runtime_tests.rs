//! Tests for Hadolint runtime

use rstest::rstest;
use vx_provider_hadolint::{HadolintProvider, HadolintRuntime, HadolintUrlBuilder};
use vx_runtime::{Arch, Ecosystem, Os, Platform, Provider, Runtime};

#[test]
fn test_runtime_name() {
    let runtime = HadolintRuntime::new();
    assert_eq!(runtime.name(), "hadolint");
}

#[test]
fn test_runtime_description() {
    let runtime = HadolintRuntime::new();
    assert!(runtime.description().contains("Hadolint"));
}

#[test]
fn test_runtime_ecosystem() {
    let runtime = HadolintRuntime::new();
    assert_eq!(
        runtime.ecosystem(),
        Ecosystem::Custom("devtools".to_string())
    );
}

#[test]
fn test_runtime_metadata() {
    let runtime = HadolintRuntime::new();
    let meta = runtime.metadata();
    assert!(meta.contains_key("homepage"));
    assert!(meta.contains_key("repository"));
    assert!(meta.contains_key("license"));
    assert_eq!(meta.get("license"), Some(&"GPL-3.0".to_string()));
    assert_eq!(meta.get("category"), Some(&"linter".to_string()));
}

#[test]
fn test_runtime_aliases() {
    let runtime = HadolintRuntime::new();
    assert!(runtime.aliases().is_empty());
}

#[test]
fn test_provider_name() {
    let provider = HadolintProvider::new();
    assert_eq!(provider.name(), "hadolint");
}

#[test]
fn test_provider_description() {
    let provider = HadolintProvider::new();
    assert!(!provider.description().is_empty());
}

#[test]
fn test_provider_supports() {
    let provider = HadolintProvider::new();
    assert!(provider.supports("hadolint"));
    assert!(!provider.supports("other"));
}

#[test]
fn test_provider_runtimes() {
    let provider = HadolintProvider::new();
    let runtimes = provider.runtimes();
    assert_eq!(runtimes.len(), 1);
    assert_eq!(runtimes[0].name(), "hadolint");
}

#[test]
fn test_provider_get_runtime() {
    let provider = HadolintProvider::new();
    assert!(provider.get_runtime("hadolint").is_some());
    assert!(provider.get_runtime("other").is_none());
}

/// Test executable_relative_path for all supported platforms
#[rstest]
#[case(Os::Linux, Arch::X86_64, "bin/hadolint")]
#[case(Os::Linux, Arch::Aarch64, "bin/hadolint")]
#[case(Os::MacOS, Arch::X86_64, "bin/hadolint")]
#[case(Os::MacOS, Arch::Aarch64, "bin/hadolint")]
#[case(Os::Windows, Arch::X86_64, "bin/hadolint.exe")]
fn test_executable_relative_path(#[case] os: Os, #[case] arch: Arch, #[case] expected: &str) {
    let runtime = HadolintRuntime::new();
    let platform = Platform::new(os, arch);
    let path = runtime.executable_relative_path("2.14.0", &platform);
    assert_eq!(path, expected);
}

/// Test download URL format for all supported platforms
#[tokio::test]
async fn test_download_url_linux_x64() {
    let runtime = HadolintRuntime::new();
    let platform = Platform::new(Os::Linux, Arch::X86_64);
    let url = runtime.download_url("2.14.0", &platform).await.unwrap();
    assert!(url.is_some());
    let url = url.unwrap();
    assert!(url.contains("hadolint/hadolint"));
    assert!(url.contains("v2.14.0"));
    assert!(url.contains("hadolint-linux-x86_64"));
    assert!(!url.ends_with(".exe"));
}

#[tokio::test]
async fn test_download_url_linux_arm64() {
    let runtime = HadolintRuntime::new();
    let platform = Platform::new(Os::Linux, Arch::Aarch64);
    let url = runtime.download_url("2.14.0", &platform).await.unwrap();
    assert!(url.is_some());
    assert!(url.unwrap().contains("hadolint-linux-arm64"));
}

#[tokio::test]
async fn test_download_url_macos_x64() {
    let runtime = HadolintRuntime::new();
    let platform = Platform::new(Os::MacOS, Arch::X86_64);
    let url = runtime.download_url("2.14.0", &platform).await.unwrap();
    assert!(url.is_some());
    assert!(url.unwrap().contains("hadolint-macos-x86_64"));
}

#[tokio::test]
async fn test_download_url_macos_arm64() {
    let runtime = HadolintRuntime::new();
    let platform = Platform::new(Os::MacOS, Arch::Aarch64);
    let url = runtime.download_url("2.14.0", &platform).await.unwrap();
    assert!(url.is_some());
    assert!(url.unwrap().contains("hadolint-macos-arm64"));
}

#[tokio::test]
async fn test_download_url_windows_x64() {
    let runtime = HadolintRuntime::new();
    let platform = Platform::new(Os::Windows, Arch::X86_64);
    let url = runtime.download_url("2.14.0", &platform).await.unwrap();
    assert!(url.is_some());
    let url = url.unwrap();
    assert!(url.contains("hadolint-windows-x86_64.exe"));
}

#[tokio::test]
async fn test_download_url_windows_arm64_not_supported() {
    let runtime = HadolintRuntime::new();
    let platform = Platform::new(Os::Windows, Arch::Aarch64);
    let url = runtime.download_url("2.14.0", &platform).await.unwrap();
    assert!(url.is_none());
}

/// Test asset name generation
#[rstest]
#[case(Os::Linux, Arch::X86_64, "hadolint-linux-x86_64")]
#[case(Os::Linux, Arch::Aarch64, "hadolint-linux-arm64")]
#[case(Os::MacOS, Arch::X86_64, "hadolint-macos-x86_64")]
#[case(Os::MacOS, Arch::Aarch64, "hadolint-macos-arm64")]
#[case(Os::Windows, Arch::X86_64, "hadolint-windows-x86_64.exe")]
fn test_asset_name(#[case] os: Os, #[case] arch: Arch, #[case] expected: &str) {
    let platform = Platform::new(os, arch);
    let asset = HadolintUrlBuilder::get_asset_name(&platform);
    assert_eq!(asset, Some(expected.to_string()));
}

/// Test executable name for different platforms
#[rstest]
#[case(Os::Windows, "hadolint.exe")]
#[case(Os::Linux, "hadolint")]
#[case(Os::MacOS, "hadolint")]
fn test_executable_name(#[case] os: Os, #[case] expected: &str) {
    let platform = Platform::new(os, Arch::X86_64);
    let name = HadolintUrlBuilder::get_executable_name(&platform);
    assert_eq!(name, expected);
}

/// Test executable_extensions returns default
#[test]
fn test_executable_extensions() {
    let runtime = HadolintRuntime::new();
    assert_eq!(runtime.executable_extensions(), &[".exe"]);
}

/// Test executable_name method returns correct base name
#[test]
fn test_executable_name_method() {
    let runtime = HadolintRuntime::new();
    assert_eq!(runtime.executable_name(), runtime.name());
}
