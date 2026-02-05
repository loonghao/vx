//! NuGet runtime tests

use vx_provider_nuget::{NugetProvider, NugetRuntime};
use vx_runtime::{Arch, Ecosystem, Os, Platform, Provider, Runtime};

#[test]
fn test_provider_name() {
    let provider = NugetProvider::new();
    assert_eq!(provider.name(), "nuget");
}

#[test]
fn test_provider_runtimes() {
    let provider = NugetProvider::new();
    let runtimes = provider.runtimes();
    assert_eq!(runtimes.len(), 1);
    assert_eq!(runtimes[0].name(), "nuget");
}

#[test]
fn test_runtime_name() {
    let runtime = NugetRuntime::new();
    assert_eq!(runtime.name(), "nuget");
}

#[test]
fn test_runtime_ecosystem() {
    let runtime = NugetRuntime::new();
    assert_eq!(runtime.ecosystem(), Ecosystem::Dotnet);
}

#[test]
fn test_runtime_aliases() {
    let runtime = NugetRuntime::new();
    assert!(runtime.aliases().contains(&"nuget-cli"));
}

#[test]
fn test_supported_platforms() {
    let runtime = NugetRuntime::new();
    let platforms = runtime.supported_platforms();

    // Should only support Windows (nuget.exe is Windows-only)
    assert!(platforms.iter().all(|p| matches!(p.os, Os::Windows)));
}

#[test]
fn test_platform_support() {
    let runtime = NugetRuntime::new();

    let windows = Platform::new(Os::Windows, Arch::X86_64);
    let linux = Platform::new(Os::Linux, Arch::X86_64);
    let macos = Platform::new(Os::MacOS, Arch::Aarch64);

    assert!(runtime.is_platform_supported(&windows));
    assert!(!runtime.is_platform_supported(&linux));
    assert!(!runtime.is_platform_supported(&macos));
}

#[tokio::test]
async fn test_download_url() {
    let runtime = NugetRuntime::new();
    let platform = Platform::new(Os::Windows, Arch::X86_64);

    let url = runtime.download_url("6.11.1", &platform).await.unwrap();
    assert!(url.is_some());
    let url = url.unwrap();
    assert!(url.contains("6.11.1"));
    assert!(url.contains("nuget.exe"));
}

#[test]
fn test_is_version_installable() {
    let runtime = NugetRuntime::new();
    // nuget can be installed via download
    assert!(runtime.is_version_installable("6.11.1"));
    assert!(runtime.is_version_installable("6.10.2"));
}

#[test]
fn test_store_name() {
    let runtime = NugetRuntime::new();
    assert_eq!(runtime.store_name(), "nuget");
}

#[test]
fn test_download_url_non_windows() {
    let runtime = NugetRuntime::new();
    let linux = Platform::new(Os::Linux, Arch::X86_64);
    let macos = Platform::new(Os::MacOS, Arch::Aarch64);

    // Non-Windows platforms should return None
    let url_linux = tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(async { runtime.download_url("6.11.1", &linux).await.unwrap() });
    assert!(url_linux.is_none());

    let url_macos = tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(async { runtime.download_url("6.11.1", &macos).await.unwrap() });
    assert!(url_macos.is_none());
}

#[test]
fn test_executable_relative_path() {
    let runtime = NugetRuntime::new();
    let windows = Platform::new(Os::Windows, Arch::X86_64);

    // BinaryHandler installs to bin/ subdirectory
    let path = runtime.executable_relative_path("6.11.1", &windows);
    assert_eq!(path, "bin/nuget.exe");
}
