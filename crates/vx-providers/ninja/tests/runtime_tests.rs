//! Tests for Ninja runtime

use rstest::rstest;
use vx_provider_ninja::{NinjaProvider, NinjaRuntime, NinjaUrlBuilder};
use vx_runtime::{Arch, Ecosystem, Os, Platform, Provider, Runtime};

#[test]
fn test_runtime_name() {
    let runtime = NinjaRuntime::new();
    assert_eq!(runtime.name(), "ninja");
}

#[test]
fn test_runtime_description() {
    let runtime = NinjaRuntime::new();
    assert!(runtime.description().contains("Ninja"));
}

#[test]
fn test_runtime_ecosystem() {
    let runtime = NinjaRuntime::new();
    assert_eq!(runtime.ecosystem(), Ecosystem::System);
}

#[test]
fn test_runtime_metadata() {
    let runtime = NinjaRuntime::new();
    let meta = runtime.metadata();
    assert!(meta.contains_key("homepage"));
    assert!(meta.contains_key("repository"));
}

#[test]
fn test_provider_name() {
    let provider = NinjaProvider::new();
    assert_eq!(provider.name(), "ninja");
}

#[test]
fn test_provider_supports() {
    let provider = NinjaProvider::new();
    assert!(provider.supports("ninja"));
    assert!(!provider.supports("other"));
}

#[test]
fn test_provider_runtimes() {
    let provider = NinjaProvider::new();
    let runtimes = provider.runtimes();
    assert_eq!(runtimes.len(), 1);
    assert_eq!(runtimes[0].name(), "ninja");
}

#[test]
fn test_provider_get_runtime() {
    let provider = NinjaProvider::new();
    assert!(provider.get_runtime("ninja").is_some());
    assert!(provider.get_runtime("other").is_none());
}

#[rstest]
#[case(Os::Linux, Arch::X86_64, Some("linux"))]
#[case(Os::Linux, Arch::Aarch64, Some("linux-aarch64"))]
#[case(Os::MacOS, Arch::X86_64, Some("mac"))]
#[case(Os::MacOS, Arch::Aarch64, Some("mac"))]
#[case(Os::Windows, Arch::X86_64, Some("win"))]
fn test_platform_name(#[case] os: Os, #[case] arch: Arch, #[case] expected: Option<&str>) {
    let platform = Platform::new(os, arch);
    assert_eq!(NinjaUrlBuilder::get_platform_name(&platform), expected);
}

#[rstest]
#[case(Os::Windows, "ninja.exe")]
#[case(Os::Linux, "ninja")]
#[case(Os::MacOS, "ninja")]
fn test_executable_name(#[case] os: Os, #[case] expected: &str) {
    let platform = Platform::new(os, Arch::X86_64);
    assert_eq!(NinjaUrlBuilder::get_executable_name(&platform), expected);
}

#[test]
fn test_download_url_format() {
    let platform = Platform::new(Os::Linux, Arch::X86_64);
    let url = NinjaUrlBuilder::download_url("1.12.1", &platform).unwrap();
    assert!(url.contains("github.com/ninja-build/ninja"));
    assert!(url.contains("v1.12.1"));
    assert!(url.contains("ninja-linux"));
    assert!(url.ends_with(".zip"));
}

#[test]
fn test_download_url_with_v_prefix() {
    let platform = Platform::new(Os::Linux, Arch::X86_64);
    let url1 = NinjaUrlBuilder::download_url("1.12.1", &platform);
    let url2 = NinjaUrlBuilder::download_url("v1.12.1", &platform);
    // Both should produce the same URL
    assert_eq!(url1, url2);
}
