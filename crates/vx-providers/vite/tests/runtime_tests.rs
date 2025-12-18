//! Tests for Vite runtime

use rstest::rstest;
use vx_provider_vite::{ViteProvider, ViteRuntime, ViteUrlBuilder};
use vx_runtime::{Arch, Ecosystem, Os, Platform, Provider, Runtime};

#[test]
fn test_runtime_name() {
    let runtime = ViteRuntime::new();
    assert_eq!(runtime.name(), "vite");
}

#[test]
fn test_runtime_description() {
    let runtime = ViteRuntime::new();
    assert!(runtime.description().contains("Vite"));
}

#[test]
fn test_runtime_ecosystem() {
    let runtime = ViteRuntime::new();
    assert_eq!(runtime.ecosystem(), Ecosystem::NodeJs);
}

#[test]
fn test_runtime_metadata() {
    let runtime = ViteRuntime::new();
    let meta = runtime.metadata();
    assert!(meta.contains_key("homepage"));
    assert!(meta.get("homepage").unwrap().contains("vitejs"));
}

#[test]
fn test_provider_name() {
    let provider = ViteProvider::new();
    assert_eq!(provider.name(), "vite");
}

#[test]
fn test_provider_supports() {
    let provider = ViteProvider::new();
    assert!(provider.supports("vite"));
    assert!(!provider.supports("webpack"));
}

#[test]
fn test_provider_runtimes() {
    let provider = ViteProvider::new();
    let runtimes = provider.runtimes();
    assert_eq!(runtimes.len(), 1);
    assert_eq!(runtimes[0].name(), "vite");
}

#[test]
fn test_provider_get_runtime() {
    let provider = ViteProvider::new();
    assert!(provider.get_runtime("vite").is_some());
    assert!(provider.get_runtime("webpack").is_none());
}

#[rstest]
#[case(Os::Linux, Arch::X86_64, "x86_64-unknown-linux-musl")]
#[case(Os::Linux, Arch::Aarch64, "aarch64-unknown-linux-musl")]
#[case(Os::MacOS, Arch::X86_64, "x86_64-apple-darwin")]
#[case(Os::MacOS, Arch::Aarch64, "aarch64-apple-darwin")]
#[case(Os::Windows, Arch::X86_64, "x86_64-pc-windows-msvc")]
#[case(Os::Windows, Arch::Aarch64, "aarch64-pc-windows-msvc")]
fn test_target_triple(#[case] os: Os, #[case] arch: Arch, #[case] expected: &str) {
    let platform = Platform { os, arch };
    let triple = ViteUrlBuilder::get_target_triple(&platform);
    assert_eq!(triple, Some(expected.to_string()));
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
    let ext = ViteUrlBuilder::get_archive_extension(&platform);
    assert_eq!(ext, expected);
}

#[rstest]
#[case(Os::Windows, "vite.exe")]
#[case(Os::Linux, "vite")]
#[case(Os::MacOS, "vite")]
fn test_executable_name(#[case] os: Os, #[case] expected: &str) {
    let platform = Platform {
        os,
        arch: Arch::X86_64,
    };
    let name = ViteUrlBuilder::get_executable_name(&platform);
    assert_eq!(name, expected);
}

#[test]
fn test_download_url_linux_x64() {
    let platform = Platform {
        os: Os::Linux,
        arch: Arch::X86_64,
    };
    let url = ViteUrlBuilder::download_url("6.0.0", &platform);
    assert!(url.is_some());
    let url = url.unwrap();
    assert!(url.contains("vite-standalone"));
    assert!(url.contains("v6.0.0"));
    assert!(url.contains("x86_64-unknown-linux-musl"));
    assert!(url.ends_with(".tar.gz"));
}

#[test]
fn test_download_url_windows_x64() {
    let platform = Platform {
        os: Os::Windows,
        arch: Arch::X86_64,
    };
    let url = ViteUrlBuilder::download_url("6.0.0", &platform);
    assert!(url.is_some());
    let url = url.unwrap();
    assert!(url.contains("vite-standalone"));
    assert!(url.ends_with(".zip"));
}

#[test]
fn test_download_url_macos_arm64() {
    let platform = Platform {
        os: Os::MacOS,
        arch: Arch::Aarch64,
    };
    let url = ViteUrlBuilder::download_url("6.0.0", &platform);
    assert!(url.is_some());
    let url = url.unwrap();
    assert!(url.contains("aarch64-apple-darwin"));
    assert!(url.ends_with(".tar.gz"));
}
