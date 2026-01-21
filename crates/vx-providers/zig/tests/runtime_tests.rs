//! Zig runtime tests

use rstest::rstest;
use vx_provider_zig::{ZigProvider, ZigRuntime};
use vx_runtime::{Arch, Ecosystem, Os, Platform, Provider, Runtime};

#[test]
fn test_zig_runtime_creation() {
    let runtime = ZigRuntime::new();
    assert_eq!(runtime.name(), "zig");
    assert!(!runtime.description().is_empty());
    assert_eq!(runtime.ecosystem(), Ecosystem::System);
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
/// Zig archives extract to a versioned directory like zig-x86_64-linux-0.11.0/
/// Note: Zig uses {arch}-{os} order, NOT {os}-{arch}
#[rstest]
#[case(Os::Linux, Arch::X86_64, "0.11.0", "zig-x86_64-linux-0.11.0/zig")]
#[case(Os::Linux, Arch::Aarch64, "0.11.0", "zig-aarch64-linux-0.11.0/zig")]
#[case(Os::MacOS, Arch::X86_64, "0.11.0", "zig-x86_64-macos-0.11.0/zig")]
#[case(Os::MacOS, Arch::Aarch64, "0.11.0", "zig-aarch64-macos-0.11.0/zig")]
#[case(
    Os::Windows,
    Arch::X86_64,
    "0.11.0",
    "zig-x86_64-windows-0.11.0/zig.exe"
)]
fn test_zig_executable_relative_path(
    #[case] os: Os,
    #[case] arch: Arch,
    #[case] version: &str,
    #[case] expected: &str,
) {
    let runtime = ZigRuntime::new();
    let platform = Platform::new(os, arch);
    let path = runtime.executable_relative_path(version, &platform);
    assert_eq!(path, expected);
}

#[tokio::test]
async fn test_zig_download_url_format() {
    let runtime = ZigRuntime::new();
    let platform = Platform::new(Os::Linux, Arch::X86_64);

    let url = runtime.download_url("0.11.0", &platform).await.unwrap();
    assert!(url.is_some());
    let url = url.unwrap();
    assert!(url.contains("ziglang.org/download"));
    assert!(url.contains("0.11.0"));
    // Note: Zig uses {arch}-{os} order: x86_64-linux
    assert!(url.contains("x86_64-linux"));
    assert!(url.ends_with(".tar.xz"));
}

#[tokio::test]
async fn test_zig_download_url_windows() {
    let runtime = ZigRuntime::new();
    let platform = Platform::new(Os::Windows, Arch::X86_64);

    let url = runtime.download_url("0.11.0", &platform).await.unwrap();
    assert!(url.is_some());
    let url = url.unwrap();
    // Note: Zig uses {arch}-{os} order: x86_64-windows
    assert!(url.contains("x86_64-windows"));
    assert!(url.ends_with(".zip"));
}

/// Test the exact download URL format matches Zig's official format
#[tokio::test]
async fn test_zig_download_url_exact_format() {
    let runtime = ZigRuntime::new();

    // Test Linux x86_64
    let platform = Platform::new(Os::Linux, Arch::X86_64);
    let url = runtime.download_url("0.15.2", &platform).await.unwrap();
    assert_eq!(
        url,
        Some("https://ziglang.org/download/0.15.2/zig-x86_64-linux-0.15.2.tar.xz".to_string())
    );

    // Test Windows x86_64
    let platform = Platform::new(Os::Windows, Arch::X86_64);
    let url = runtime.download_url("0.15.2", &platform).await.unwrap();
    assert_eq!(
        url,
        Some("https://ziglang.org/download/0.15.2/zig-x86_64-windows-0.15.2.zip".to_string())
    );

    // Test macOS aarch64
    let platform = Platform::new(Os::MacOS, Arch::Aarch64);
    let url = runtime.download_url("0.15.2", &platform).await.unwrap();
    assert_eq!(
        url,
        Some("https://ziglang.org/download/0.15.2/zig-aarch64-macos-0.15.2.tar.xz".to_string())
    );
}
