//! Zig runtime tests

use rstest::rstest;
use vx_provider_zig::{ZigProvider, ZigRuntime};
use vx_runtime::{Arch, Ecosystem, Os, Platform, Provider, Runtime};

#[test]
fn test_zig_runtime_creation() {
    let runtime = ZigRuntime::new();
    assert_eq!(runtime.name(), "zig");
    assert!(!runtime.description().is_empty());
    assert_eq!(runtime.ecosystem(), Ecosystem::Unknown);
}

#[test]
fn test_zig_runtime_description() {
    let runtime = ZigRuntime::new();
    assert!(runtime.description().contains("Zig") || runtime.description().contains("programming"));
}

#[test]
fn test_zig_runtime_metadata() {
    let runtime = ZigRuntime::new();
    let metadata = runtime.metadata();

    assert!(metadata.contains_key("homepage"));
    assert!(metadata.contains_key("ecosystem"));
    assert!(metadata.contains_key("repository"));
    assert!(metadata.contains_key("license"));
    assert_eq!(
        metadata.get("homepage"),
        Some(&"https://ziglang.org/".to_string())
    );
}

#[test]
fn test_zig_runtime_aliases() {
    let runtime = ZigRuntime::new();
    assert!(runtime.aliases().contains(&"ziglang"));
}

#[test]
fn test_zig_provider_creation() {
    let provider = ZigProvider::new();
    assert_eq!(provider.name(), "zig");
    assert!(!provider.description().is_empty());
}

#[test]
fn test_zig_provider_runtimes() {
    let provider = ZigProvider::new();
    let runtimes = provider.runtimes();

    assert_eq!(runtimes.len(), 1);
    assert_eq!(runtimes[0].name(), "zig");
}

#[rstest]
#[case("zig", true)]
#[case("ziglang", true)]
#[case("rust", false)]
#[case("go", false)]
fn test_zig_provider_supports(#[case] name: &str, #[case] expected: bool) {
    let provider = ZigProvider::new();
    assert_eq!(provider.supports(name), expected);
}

#[test]
fn test_zig_provider_get_runtime() {
    let provider = ZigProvider::new();

    let zig = provider.get_runtime("zig");
    assert!(zig.is_some());
    assert_eq!(zig.unwrap().name(), "zig");

    let ziglang = provider.get_runtime("ziglang");
    assert!(ziglang.is_some());
    assert_eq!(ziglang.unwrap().name(), "zig");

    let unknown = provider.get_runtime("unknown");
    assert!(unknown.is_none());
}

/// Test that executable_relative_path returns correct path for Zig archives
/// Zig archives extract to a versioned directory like zig-linux-x86_64-0.11.0/
#[rstest]
#[case(Os::Linux, Arch::X86_64, "0.11.0", "zig-linux-x86_64-0.11.0/zig")]
#[case(Os::Linux, Arch::Aarch64, "0.11.0", "zig-linux-aarch64-0.11.0/zig")]
#[case(Os::MacOS, Arch::X86_64, "0.11.0", "zig-macos-x86_64-0.11.0/zig")]
#[case(Os::MacOS, Arch::Aarch64, "0.11.0", "zig-macos-aarch64-0.11.0/zig")]
#[case(
    Os::Windows,
    Arch::X86_64,
    "0.11.0",
    "zig-windows-x86_64-0.11.0/zig.exe"
)]
fn test_zig_executable_relative_path(
    #[case] os: Os,
    #[case] arch: Arch,
    #[case] version: &str,
    #[case] expected: &str,
) {
    let runtime = ZigRuntime::new();
    let platform = Platform { os, arch };
    let path = runtime.executable_relative_path(version, &platform);
    assert_eq!(path, expected);
}

#[tokio::test]
async fn test_zig_download_url_format() {
    let runtime = ZigRuntime::new();
    let platform = Platform {
        os: Os::Linux,
        arch: Arch::X86_64,
    };

    let url = runtime.download_url("0.11.0", &platform).await.unwrap();
    assert!(url.is_some());
    let url = url.unwrap();
    assert!(url.contains("ziglang.org/download"));
    assert!(url.contains("0.11.0"));
    assert!(url.contains("linux-x86_64"));
    assert!(url.ends_with(".tar.xz"));
}

#[tokio::test]
async fn test_zig_download_url_windows() {
    let runtime = ZigRuntime::new();
    let platform = Platform {
        os: Os::Windows,
        arch: Arch::X86_64,
    };

    let url = runtime.download_url("0.11.0", &platform).await.unwrap();
    assert!(url.is_some());
    let url = url.unwrap();
    assert!(url.contains("windows-x86_64"));
    assert!(url.ends_with(".zip"));
}
