//! Tests for .NET SDK runtime

use rstest::rstest;
use vx_provider_dotnet::{DotnetProvider, DotnetRuntime, DotnetUrlBuilder};
use vx_runtime::{Arch, Ecosystem, Os, Platform, Provider, Runtime};

#[test]
fn test_runtime_name() {
    let runtime = DotnetRuntime::new();
    assert_eq!(runtime.name(), "dotnet");
}

#[test]
fn test_runtime_description() {
    let runtime = DotnetRuntime::new();
    assert!(runtime.description().contains(".NET"));
}

#[test]
fn test_runtime_ecosystem() {
    let runtime = DotnetRuntime::new();
    assert_eq!(runtime.ecosystem(), Ecosystem::System);
}

#[test]
fn test_runtime_aliases() {
    let runtime = DotnetRuntime::new();
    assert!(runtime.aliases().contains(&"dotnet-sdk"));
}

#[test]
fn test_runtime_metadata() {
    let runtime = DotnetRuntime::new();
    let meta = runtime.metadata();
    assert!(meta.contains_key("homepage"));
    assert!(meta.contains_key("documentation"));
    assert!(meta.contains_key("repository"));
}

#[test]
fn test_provider_name() {
    let provider = DotnetProvider::new();
    assert_eq!(provider.name(), "dotnet");
}

#[test]
fn test_provider_supports() {
    let provider = DotnetProvider::new();
    assert!(provider.supports("dotnet"));
    assert!(provider.supports("dotnet-sdk"));
    assert!(!provider.supports("other"));
}

#[test]
fn test_provider_runtimes() {
    let provider = DotnetProvider::new();
    let runtimes = provider.runtimes();
    assert_eq!(runtimes.len(), 1);
    assert_eq!(runtimes[0].name(), "dotnet");
}

#[test]
fn test_provider_get_runtime() {
    let provider = DotnetProvider::new();
    assert!(provider.get_runtime("dotnet").is_some());
    assert!(provider.get_runtime("dotnet-sdk").is_some());
    assert!(provider.get_runtime("other").is_none());
}

#[rstest]
#[case(Os::Linux, Arch::X86_64, "linux-x64")]
#[case(Os::Linux, Arch::Aarch64, "linux-arm64")]
#[case(Os::Linux, Arch::Arm, "linux-arm")]
#[case(Os::MacOS, Arch::X86_64, "osx-x64")]
#[case(Os::MacOS, Arch::Aarch64, "osx-arm64")]
#[case(Os::Windows, Arch::X86_64, "win-x64")]
#[case(Os::Windows, Arch::X86, "win-x86")]
#[case(Os::Windows, Arch::Aarch64, "win-arm64")]
fn test_runtime_identifier(#[case] os: Os, #[case] arch: Arch, #[case] expected: &str) {
    let platform = Platform::new(os, arch);
    let rid = DotnetUrlBuilder::get_runtime_identifier(&platform);
    assert_eq!(rid, Some(expected.to_string()));
}

#[rstest]
#[case(Os::Windows, "zip")]
#[case(Os::Linux, "tar.gz")]
#[case(Os::MacOS, "tar.gz")]
fn test_archive_extension(#[case] os: Os, #[case] expected: &str) {
    let platform = Platform::new(os, Arch::X86_64);
    let ext = DotnetUrlBuilder::get_archive_extension(&platform);
    assert_eq!(ext, expected);
}

#[rstest]
#[case(Os::Windows, "dotnet.exe")]
#[case(Os::Linux, "dotnet")]
#[case(Os::MacOS, "dotnet")]
fn test_executable_name(#[case] os: Os, #[case] expected: &str) {
    let platform = Platform::new(os, Arch::X86_64);
    let name = DotnetUrlBuilder::get_executable_name(&platform);
    assert_eq!(name, expected);
}

#[test]
fn test_download_url_format() {
    let platform = Platform::new(Os::Linux, Arch::X86_64);
    let url = DotnetUrlBuilder::download_url("9.0.310", &platform).unwrap();
    assert!(url.contains("builds.dotnet.microsoft.com"));
    assert!(url.contains("9.0.310"));
    assert!(url.contains("linux-x64"));
    assert!(url.ends_with(".tar.gz"));
}

#[test]
fn test_download_url_windows() {
    let platform = Platform::new(Os::Windows, Arch::X86_64);
    let url = DotnetUrlBuilder::download_url("9.0.310", &platform).unwrap();
    assert!(url.contains("win-x64"));
    assert!(url.ends_with(".zip"));
}

#[test]
fn test_download_url_macos_arm64() {
    let platform = Platform::new(Os::MacOS, Arch::Aarch64);
    let url = DotnetUrlBuilder::download_url("9.0.310", &platform).unwrap();
    assert!(url.contains("osx-arm64"));
    assert!(url.ends_with(".tar.gz"));
}

/// Test executable_relative_path uses the correct path
#[test]
fn test_executable_relative_path() {
    let runtime = DotnetRuntime::new();

    let linux_platform = Platform::new(Os::Linux, Arch::X86_64);
    assert_eq!(
        runtime.executable_relative_path("9.0.310", &linux_platform),
        "dotnet"
    );

    let windows_platform = Platform::new(Os::Windows, Arch::X86_64);
    assert_eq!(
        runtime.executable_relative_path("9.0.310", &windows_platform),
        "dotnet.exe"
    );
}

/// Test executable_extensions returns correct extensions
#[test]
fn test_executable_extensions() {
    let runtime = DotnetRuntime::new();
    // Default is [".exe"]
    assert_eq!(runtime.executable_extensions(), &[".exe"]);
}

/// Test executable_name returns correct base name
#[test]
fn test_executable_name_method() {
    let runtime = DotnetRuntime::new();
    // Default is same as name()
    assert_eq!(runtime.executable_name(), runtime.name());
}
