//! Tests for Just runtime

use rstest::rstest;
use vx_provider_just::{JustProvider, JustRuntime, JustUrlBuilder};
use vx_runtime::{Arch, Ecosystem, Os, Platform, Provider, Runtime};

#[test]
fn test_runtime_name() {
    let runtime = JustRuntime::new();
    assert_eq!(runtime.name(), "just");
}

#[test]
fn test_runtime_description() {
    let runtime = JustRuntime::new();
    assert!(runtime.description().contains("Just"));
}

#[test]
fn test_runtime_ecosystem() {
    let runtime = JustRuntime::new();
    assert_eq!(runtime.ecosystem(), Ecosystem::System);
}

#[test]
fn test_runtime_metadata() {
    let runtime = JustRuntime::new();
    let meta = runtime.metadata();
    assert!(meta.contains_key("homepage"));
    assert!(meta.get("homepage").unwrap().contains("casey/just"));
}

#[test]
fn test_provider_name() {
    let provider = JustProvider::new();
    assert_eq!(provider.name(), "just");
}

#[test]
fn test_provider_supports() {
    let provider = JustProvider::new();
    assert!(provider.supports("just"));
    assert!(!provider.supports("make"));
}

#[test]
fn test_provider_runtimes() {
    let provider = JustProvider::new();
    let runtimes = provider.runtimes();
    assert_eq!(runtimes.len(), 1);
    assert_eq!(runtimes[0].name(), "just");
}

#[test]
fn test_provider_get_runtime() {
    let provider = JustProvider::new();
    assert!(provider.get_runtime("just").is_some());
    assert!(provider.get_runtime("make").is_none());
}

#[rstest]
#[case(Os::Linux, Arch::X86_64, "x86_64-unknown-linux-musl")]
#[case(Os::Linux, Arch::Aarch64, "aarch64-unknown-linux-musl")]
#[case(Os::MacOS, Arch::X86_64, "x86_64-apple-darwin")]
#[case(Os::MacOS, Arch::Aarch64, "aarch64-apple-darwin")]
#[case(Os::Windows, Arch::X86_64, "x86_64-pc-windows-msvc")]
#[case(Os::Windows, Arch::Aarch64, "aarch64-pc-windows-msvc")]
#[case(Os::Linux, Arch::Arm, "arm-unknown-linux-musleabihf")]
fn test_target_triple(#[case] os: Os, #[case] arch: Arch, #[case] expected: &str) {
    let platform = Platform::new(os, arch);
    let triple = JustUrlBuilder::get_target_triple(&platform);
    assert_eq!(triple, Some(expected.to_string()));
}

#[rstest]
#[case(Os::Windows, "zip")]
#[case(Os::Linux, "tar.gz")]
#[case(Os::MacOS, "tar.gz")]
fn test_archive_extension(#[case] os: Os, #[case] expected: &str) {
    let platform = Platform::new(os, Arch::X86_64);
    let ext = JustUrlBuilder::get_archive_extension(&platform);
    assert_eq!(ext, expected);
}

#[rstest]
#[case(Os::Windows, "just.exe")]
#[case(Os::Linux, "just")]
#[case(Os::MacOS, "just")]
fn test_executable_name(#[case] os: Os, #[case] expected: &str) {
    let platform = Platform::new(os, Arch::X86_64);
    let name = JustUrlBuilder::get_executable_name(&platform);
    assert_eq!(name, expected);
}

#[test]
fn test_download_url_format() {
    let platform = Platform::new(Os::Linux, Arch::X86_64);
    let url = JustUrlBuilder::download_url("1.45.0", &platform).unwrap();
    assert!(url.contains("github.com/casey/just"));
    assert!(url.contains("1.45.0"));
    assert!(url.contains("x86_64-unknown-linux-musl"));
    assert!(url.ends_with(".tar.gz"));
}

#[test]
fn test_executable_relative_path() {
    let runtime = JustRuntime::new();

    let linux_platform = Platform::new(Os::Linux, Arch::X86_64);
    assert_eq!(
        runtime.executable_relative_path("1.45.0", &linux_platform),
        "just"
    );

    let windows_platform = Platform::new(Os::Windows, Arch::X86_64);
    assert_eq!(
        runtime.executable_relative_path("1.45.0", &windows_platform),
        "just.exe"
    );
}
