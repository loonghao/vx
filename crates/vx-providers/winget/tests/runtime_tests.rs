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
    assert!(platforms.iter().all(|p| matches!(p.os, Os::Windows)));
}

#[test]
fn test_platform_support() {
    let runtime = WingetRuntime::new();

    let windows = Platform::new(Os::Windows, Arch::X86_64);
    let linux = Platform::new(Os::Linux, Arch::X86_64);
    let macos = Platform::new(Os::MacOS, Arch::Aarch64);

    assert!(runtime.is_platform_supported(&windows));
    assert!(!runtime.is_platform_supported(&linux));
    assert!(!runtime.is_platform_supported(&macos));
}

#[test]
fn test_is_version_installable() {
    let runtime = WingetRuntime::new();
    // winget can be installed from GitHub releases
    assert!(runtime.is_version_installable("1.12.460"));
    assert!(runtime.is_version_installable("1.9.25200"));
}

#[test]
fn test_download_url_windows() {
    let runtime = WingetRuntime::new();
    let windows = Platform::new(Os::Windows, Arch::X86_64);

    // Test download URL generation for Windows
    let url = tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(async { runtime.download_url("1.12.460", &windows).await.unwrap() });

    assert!(url.is_some());
    let url = url.unwrap();
    assert!(url.contains("github.com/microsoft/winget-cli/releases/download"));
    assert!(url.contains("1.12.460"));
    assert!(url.contains(".msixbundle"));
}

#[test]
fn test_download_url_non_windows() {
    let runtime = WingetRuntime::new();
    let linux = Platform::new(Os::Linux, Arch::X86_64);
    let macos = Platform::new(Os::MacOS, Arch::Aarch64);

    // Non-Windows platforms should return None
    let url_linux = tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(async { runtime.download_url("1.12.460", &linux).await.unwrap() });
    assert!(url_linux.is_none());

    let url_macos = tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(async { runtime.download_url("1.12.460", &macos).await.unwrap() });
    assert!(url_macos.is_none());
}

#[test]
fn test_store_name() {
    let runtime = WingetRuntime::new();
    assert_eq!(runtime.store_name(), "winget");
}
