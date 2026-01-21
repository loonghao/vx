//! Tests for protoc runtime

use rstest::rstest;
use vx_provider_protoc::{ProtocProvider, ProtocRuntime, ProtocUrlBuilder};
use vx_runtime::{Arch, Ecosystem, Os, Platform, Provider, Runtime};

#[test]
fn test_runtime_name() {
    let runtime = ProtocRuntime::new();
    assert_eq!(runtime.name(), "protoc");
}

#[test]
fn test_runtime_description() {
    let runtime = ProtocRuntime::new();
    assert!(runtime.description().contains("Protocol Buffers"));
}

#[test]
fn test_runtime_ecosystem() {
    let runtime = ProtocRuntime::new();
    assert_eq!(runtime.ecosystem(), Ecosystem::System);
}

#[test]
fn test_runtime_metadata() {
    let runtime = ProtocRuntime::new();
    let meta = runtime.metadata();
    assert!(meta.contains_key("homepage"));
    assert!(meta.contains_key("repository"));
}

#[test]
fn test_provider_name() {
    let provider = ProtocProvider::new();
    assert_eq!(provider.name(), "protoc");
}

#[test]
fn test_provider_supports() {
    let provider = ProtocProvider::new();
    assert!(provider.supports("protoc"));
    assert!(!provider.supports("other"));
}

#[test]
fn test_provider_runtimes() {
    let provider = ProtocProvider::new();
    let runtimes = provider.runtimes();
    assert_eq!(runtimes.len(), 1);
    assert_eq!(runtimes[0].name(), "protoc");
}

#[test]
fn test_provider_get_runtime() {
    let provider = ProtocProvider::new();
    assert!(provider.get_runtime("protoc").is_some());
    assert!(provider.get_runtime("other").is_none());
}

#[rstest]
#[case(Os::Linux, Arch::X86_64, Some("linux-x86_64"))]
#[case(Os::Linux, Arch::Aarch64, Some("linux-aarch_64"))]
#[case(Os::Linux, Arch::X86, Some("linux-x86_32"))]
#[case(Os::MacOS, Arch::X86_64, Some("osx-universal_binary"))]
#[case(Os::MacOS, Arch::Aarch64, Some("osx-universal_binary"))]
#[case(Os::Windows, Arch::X86_64, Some("win64"))]
#[case(Os::Windows, Arch::X86, Some("win32"))]
fn test_platform_suffix(#[case] os: Os, #[case] arch: Arch, #[case] expected: Option<&str>) {
    let platform = Platform::new(os, arch);
    assert_eq!(ProtocUrlBuilder::get_platform_suffix(&platform), expected);
}

#[rstest]
#[case(Os::Windows, "protoc.exe")]
#[case(Os::Linux, "protoc")]
#[case(Os::MacOS, "protoc")]
fn test_executable_name(#[case] os: Os, #[case] expected: &str) {
    let platform = Platform::new(os, Arch::X86_64);
    assert_eq!(ProtocUrlBuilder::get_executable_name(&platform), expected);
}

#[test]
fn test_download_url_format() {
    let platform = Platform::new(Os::Linux, Arch::X86_64);
    let url = ProtocUrlBuilder::download_url("29.2", &platform).unwrap();
    assert!(url.contains("github.com/protocolbuffers/protobuf"));
    assert!(url.contains("v29.2"));
    assert!(url.contains("linux-x86_64"));
    assert!(url.ends_with(".zip"));
}

#[test]
fn test_download_url_with_v_prefix() {
    let platform = Platform::new(Os::Linux, Arch::X86_64);
    let url1 = ProtocUrlBuilder::download_url("29.2", &platform);
    let url2 = ProtocUrlBuilder::download_url("v29.2", &platform);
    // Both should produce the same URL
    assert_eq!(url1, url2);
}
