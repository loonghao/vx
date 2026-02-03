//! Windows Package Manager runtime tests

use vx_provider_winget::{WingetProvider, WingetRuntime};
use vx_runtime::{Arch, Ecosystem, Os, Platform, Provider, Runtime};

#[test]
fn test_provider_name() {
    let provider = WingetProvider::new();
    assert_eq!(provider.name(), "winget");
}

#[test]
fn test_provider_runtimes() {
    let provider = WingetProvider::new();
    let runtimes = provider.runtimes();
    assert_eq!(runtimes.len(), 1);
    assert_eq!(runtimes[0].name(), "winget");
}

#[test]
fn test_runtime_name() {
    let runtime = WingetRuntime::new();
    assert_eq!(runtime.name(), "winget");
}

#[test]
fn test_runtime_ecosystem() {
    let runtime = WingetRuntime::new();
    assert_eq!(runtime.ecosystem(), Ecosystem::System);
}

#[test]
fn test_runtime_aliases() {
    let runtime = WingetRuntime::new();
    assert!(runtime.aliases().contains(&"winget-cli"));
}

#[test]
fn test_supported_platforms() {
    let runtime = WingetRuntime::new();
    let platforms = runtime.supported_platforms();

    // Should only support Windows
    assert!(platforms
        .iter()
        .all(|p| matches!(p.os, Os::Windows)));
}

#[test]
fn test_platform_support() {
    let runtime = WingetRuntime::new();

    let windows = Platform::new(Os::Windows, Arch::X86_64);
    let linux = Platform::new(Os::Linux, Arch::X86_64);
    let macos = Platform::new(Os::MacOs, Arch::Aarch64);

    assert!(runtime.is_platform_supported(&windows));
    assert!(!runtime.is_platform_supported(&linux));
    assert!(!runtime.is_platform_supported(&macos));
}
