//! Tests for Actrun runtime

use vx_provider_actrun::{ActrunProvider, ActrunRuntime, ActrunUrlBuilder};
use vx_runtime::{Arch, Ecosystem, Os, Platform, Provider, Runtime};

#[test]
fn test_runtime_name() {
    let runtime = ActrunRuntime::new();
    assert_eq!(runtime.name(), "actrun");
}

#[test]
fn test_runtime_description() {
    let runtime = ActrunRuntime::new();
    assert!(runtime.description().contains("Actionforge"));
}

#[test]
fn test_runtime_ecosystem() {
    let runtime = ActrunRuntime::new();
    assert!(matches!(runtime.ecosystem(), Ecosystem::Unknown));
}

#[test]
fn test_runtime_metadata() {
    let runtime = ActrunRuntime::new();
    let meta = runtime.metadata();
    assert!(meta.contains_key("homepage"));
    assert!(meta
        .get("homepage")
        .unwrap()
        .contains("actionforge/actrun-cli"));
}

#[test]
fn test_provider_name() {
    let provider = ActrunProvider::new();
    assert_eq!(provider.name(), "actrun");
}

#[test]
fn test_provider_supports() {
    let provider = ActrunProvider::new();
    assert!(provider.supports("actrun"));
    assert!(!provider.supports("other"));
}

#[test]
fn test_provider_runtimes() {
    let provider = ActrunProvider::new();
    let runtimes = provider.runtimes();
    assert_eq!(runtimes.len(), 1);
    assert_eq!(runtimes[0].name(), "actrun");
}

#[test]
fn test_provider_get_runtime() {
    let provider = ActrunProvider::new();
    assert!(provider.get_runtime("actrun").is_some());
    assert!(provider.get_runtime("other").is_none());
}

#[test]
fn test_arch_string_x64() {
    let platform = Platform::new(Os::Linux, Arch::X86_64);
    assert_eq!(ActrunUrlBuilder::get_arch_string(&platform), Some("x64"));
}

#[test]
fn test_arch_string_arm64() {
    let platform = Platform::new(Os::Linux, Arch::Aarch64);
    assert_eq!(ActrunUrlBuilder::get_arch_string(&platform), Some("arm64"));
}

#[test]
fn test_os_string_linux() {
    let platform = Platform::new(Os::Linux, Arch::X86_64);
    assert_eq!(ActrunUrlBuilder::get_os_string(&platform), Some("linux"));
}

#[test]
fn test_os_string_windows() {
    let platform = Platform::new(Os::Windows, Arch::X86_64);
    assert_eq!(
        ActrunUrlBuilder::get_os_string(&platform),
        Some("windows")
    );
}

#[test]
fn test_os_string_macos() {
    let platform = Platform::new(Os::MacOS, Arch::Aarch64);
    assert_eq!(
        ActrunUrlBuilder::get_os_string(&platform),
        Some("macos"),
    );
}

#[test]
fn test_archive_extension_windows() {
    let platform = Platform::new(Os::Windows, Arch::X86_64);
    assert_eq!(ActrunUrlBuilder::get_archive_extension(&platform), "zip");
}

#[test]
fn test_archive_extension_linux() {
    let platform = Platform::new(Os::Linux, Arch::X86_64);
    assert_eq!(
        ActrunUrlBuilder::get_archive_extension(&platform),
        "tar.gz"
    );
}

#[test]
fn test_archive_extension_macos() {
    let platform = Platform::new(Os::MacOS, Arch::Aarch64);
    assert_eq!(ActrunUrlBuilder::get_archive_extension(&platform), "pkg");
}

#[test]
fn test_executable_name_windows() {
    let platform = Platform::new(Os::Windows, Arch::X86_64);
    assert_eq!(
        ActrunUrlBuilder::get_executable_name(&platform),
        "actrun.exe"
    );
}

#[test]
fn test_executable_name_linux() {
    let platform = Platform::new(Os::Linux, Arch::X86_64);
    assert_eq!(ActrunUrlBuilder::get_executable_name(&platform), "actrun");
}

#[test]
fn test_download_url_linux_x64() {
    let platform = Platform::new(Os::Linux, Arch::X86_64);
    let url = ActrunUrlBuilder::download_url("0.14.6", &platform).unwrap();
    assert!(url.contains("github.com/actionforge/actrun-cli"));
    assert!(url.contains("0.14.6"));
    assert!(url.ends_with(".tar.gz"));
}

#[test]
fn test_download_url_linux_arm64() {
    let platform = Platform::new(Os::Linux, Arch::Aarch64);
    let url = ActrunUrlBuilder::download_url("0.14.6", &platform).unwrap();
    assert!(url.contains("arm64-linux"));
    assert!(url.ends_with(".tar.gz"));
}

#[test]
fn test_download_url_windows_x64() {
    let platform = Platform::new(Os::Windows, Arch::X86_64);
    let url = ActrunUrlBuilder::download_url("0.14.6", &platform).unwrap();
    assert!(url.contains("x64-windows"));
    assert!(url.ends_with(".zip"));
}

#[test]
fn test_download_url_windows_arm64() {
    let platform = Platform::new(Os::Windows, Arch::Aarch64);
    let url = ActrunUrlBuilder::download_url("0.14.6", &platform).unwrap();
    assert!(url.contains("arm64-windows"));
    assert!(url.ends_with(".zip"));
}

#[test]
fn test_download_url_macos_arm64() {
    let platform = Platform::new(Os::MacOS, Arch::Aarch64);
    let url = ActrunUrlBuilder::download_url("0.14.6", &platform).unwrap();
    assert!(url.contains("arm64-macos"));
    assert!(url.ends_with(".pkg"));
}

#[test]
fn test_download_url_macos_x64() {
    let platform = Platform::new(Os::MacOS, Arch::X86_64);
    let url = ActrunUrlBuilder::download_url("0.14.6", &platform).unwrap();
    assert!(url.contains("x64-macos"));
    assert!(url.ends_with(".pkg"));
}
