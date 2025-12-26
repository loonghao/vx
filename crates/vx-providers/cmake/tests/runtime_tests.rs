//! Tests for CMake runtime

use rstest::rstest;
use vx_provider_cmake::{CMakeProvider, CMakeRuntime, CMakeUrlBuilder};
use vx_runtime::{Arch, Ecosystem, Os, Platform, Provider, Runtime};

#[test]
fn test_runtime_name() {
    let runtime = CMakeRuntime::new();
    assert_eq!(runtime.name(), "cmake");
}

#[test]
fn test_runtime_description() {
    let runtime = CMakeRuntime::new();
    assert!(runtime.description().contains("CMake"));
}

#[test]
fn test_runtime_ecosystem() {
    let runtime = CMakeRuntime::new();
    assert_eq!(runtime.ecosystem(), Ecosystem::System);
}

#[test]
fn test_runtime_metadata() {
    let runtime = CMakeRuntime::new();
    let meta = runtime.metadata();
    assert!(meta.contains_key("homepage"));
    assert!(meta.contains_key("repository"));
}

#[test]
fn test_provider_name() {
    let provider = CMakeProvider::new();
    assert_eq!(provider.name(), "cmake");
}

#[test]
fn test_provider_supports() {
    let provider = CMakeProvider::new();
    assert!(provider.supports("cmake"));
    assert!(!provider.supports("other"));
}

#[test]
fn test_provider_runtimes() {
    let provider = CMakeProvider::new();
    let runtimes = provider.runtimes();
    assert_eq!(runtimes.len(), 1);
    assert_eq!(runtimes[0].name(), "cmake");
}

#[test]
fn test_provider_get_runtime() {
    let provider = CMakeProvider::new();
    assert!(provider.get_runtime("cmake").is_some());
    assert!(provider.get_runtime("other").is_none());
}

#[rstest]
#[case(Os::Linux, Arch::X86_64, Some("linux-x86_64"))]
#[case(Os::Linux, Arch::Aarch64, Some("linux-aarch64"))]
#[case(Os::MacOS, Arch::X86_64, Some("macos-universal"))]
#[case(Os::MacOS, Arch::Aarch64, Some("macos-universal"))]
#[case(Os::Windows, Arch::X86_64, Some("windows-x86_64"))]
#[case(Os::Windows, Arch::Aarch64, Some("windows-arm64"))]
fn test_platform_suffix(#[case] os: Os, #[case] arch: Arch, #[case] expected: Option<&str>) {
    let platform = Platform { os, arch };
    assert_eq!(CMakeUrlBuilder::get_platform_suffix(&platform), expected);
}

#[rstest]
#[case(Os::Windows, "zip")]
#[case(Os::Linux, "tar.gz")]
#[case(Os::MacOS, "tar.gz")]
fn test_archive_extension(#[case] os: Os, #[case] expected: &str) {
    let platform = Platform {
        os,
        arch: Arch::X86_64,
    };
    assert_eq!(CMakeUrlBuilder::get_archive_extension(&platform), expected);
}

#[rstest]
#[case(Os::Windows, "cmake.exe")]
#[case(Os::Linux, "cmake")]
#[case(Os::MacOS, "cmake")]
fn test_executable_name(#[case] os: Os, #[case] expected: &str) {
    let platform = Platform {
        os,
        arch: Arch::X86_64,
    };
    assert_eq!(CMakeUrlBuilder::get_executable_name(&platform), expected);
}

#[test]
fn test_download_url_format() {
    let platform = Platform {
        os: Os::Linux,
        arch: Arch::X86_64,
    };
    let url = CMakeUrlBuilder::download_url("3.31.3", &platform).unwrap();
    assert!(url.contains("github.com/Kitware/CMake"));
    assert!(url.contains("v3.31.3"));
    assert!(url.contains("linux-x86_64"));
    assert!(url.ends_with(".tar.gz"));
}

#[test]
fn test_archive_dir_name() {
    let platform = Platform {
        os: Os::Linux,
        arch: Arch::X86_64,
    };
    let dir = CMakeUrlBuilder::get_archive_dir_name("3.31.3", &platform);
    assert_eq!(dir, Some("cmake-3.31.3-linux-x86_64".to_string()));
}
