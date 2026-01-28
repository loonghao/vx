//! Tests for Just runtime

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
    assert!(runtime.description().contains("handy"));
}

#[test]
fn test_runtime_ecosystem() {
    let runtime = JustRuntime::new();
    assert!(matches!(runtime.ecosystem(), Ecosystem::Unknown));
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

#[test]
fn test_target_triple_linux_x64() {
    let platform = Platform::new(Os::Linux, Arch::X86_64);
    assert_eq!(
        JustUrlBuilder::get_target_triple(&platform),
        Some("x86_64-unknown-linux-musl".to_string())
    );
}

#[test]
fn test_target_triple_linux_arm64() {
    let platform = Platform::new(Os::Linux, Arch::Aarch64);
    assert_eq!(
        JustUrlBuilder::get_target_triple(&platform),
        Some("aarch64-unknown-linux-musl".to_string())
    );
}

#[test]
fn test_target_triple_macos_x64() {
    let platform = Platform::new(Os::MacOS, Arch::X86_64);
    assert_eq!(
        JustUrlBuilder::get_target_triple(&platform),
        Some("x86_64-apple-darwin".to_string())
    );
}

#[test]
fn test_target_triple_macos_arm64() {
    let platform = Platform::new(Os::MacOS, Arch::Aarch64);
    assert_eq!(
        JustUrlBuilder::get_target_triple(&platform),
        Some("aarch64-apple-darwin".to_string())
    );
}

#[test]
fn test_target_triple_windows_x64() {
    let platform = Platform::new(Os::Windows, Arch::X86_64);
    assert_eq!(
        JustUrlBuilder::get_target_triple(&platform),
        Some("x86_64-pc-windows-msvc".to_string())
    );
}

#[test]
fn test_target_triple_windows_arm64() {
    let platform = Platform::new(Os::Windows, Arch::Aarch64);
    assert_eq!(
        JustUrlBuilder::get_target_triple(&platform),
        Some("aarch64-pc-windows-msvc".to_string())
    );
}

#[test]
fn test_archive_extension_windows() {
    let platform = Platform::new(Os::Windows, Arch::X86_64);
    assert_eq!(JustUrlBuilder::get_archive_extension(&platform), "zip");
}

#[test]
fn test_archive_extension_linux() {
    let platform = Platform::new(Os::Linux, Arch::X86_64);
    assert_eq!(JustUrlBuilder::get_archive_extension(&platform), "tar.gz");
}

#[test]
fn test_archive_extension_macos() {
    let platform = Platform::new(Os::MacOS, Arch::X86_64);
    assert_eq!(JustUrlBuilder::get_archive_extension(&platform), "tar.gz");
}

#[test]
fn test_executable_name_windows() {
    let platform = Platform::new(Os::Windows, Arch::X86_64);
    assert_eq!(JustUrlBuilder::get_executable_name(&platform), "just.exe");
}

#[test]
fn test_executable_name_linux() {
    let platform = Platform::new(Os::Linux, Arch::X86_64);
    assert_eq!(JustUrlBuilder::get_executable_name(&platform), "just");
}

#[test]
fn test_executable_name_macos() {
    let platform = Platform::new(Os::MacOS, Arch::X86_64);
    assert_eq!(JustUrlBuilder::get_executable_name(&platform), "just");
}

#[test]
fn test_download_url_linux() {
    let platform = Platform::new(Os::Linux, Arch::X86_64);
    let url = JustUrlBuilder::download_url("1.45.0", &platform).unwrap();
    assert!(url.contains("github.com/casey/just"));
    assert!(url.contains("1.45.0"));
}

#[test]
fn test_download_url_windows() {
    let platform = Platform::new(Os::Windows, Arch::X86_64);
    let url = JustUrlBuilder::download_url("1.45.0", &platform).unwrap();
    assert!(url.contains("github.com/casey/just"));
    assert!(url.contains("1.45.0"));
}

#[test]
fn test_download_url_macos() {
    let platform = Platform::new(Os::MacOS, Arch::Aarch64);
    let url = JustUrlBuilder::download_url("1.45.0", &platform).unwrap();
    assert!(url.contains("github.com/casey/just"));
    assert!(url.contains("1.45.0"));
}
