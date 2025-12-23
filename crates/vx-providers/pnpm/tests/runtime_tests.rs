//! PNPM runtime tests

use rstest::rstest;
use vx_provider_pnpm::{PnpmProvider, PnpmRuntime};
use vx_runtime::{Arch, Ecosystem, Os, Platform, Provider, Runtime};

#[rstest]
fn test_pnpm_runtime_name() {
    let runtime = PnpmRuntime::new();
    assert_eq!(runtime.name(), "pnpm");
}

#[rstest]
fn test_pnpm_runtime_ecosystem() {
    let runtime = PnpmRuntime::new();
    assert_eq!(runtime.ecosystem(), Ecosystem::NodeJs);
}

#[rstest]
fn test_pnpm_runtime_description() {
    let runtime = PnpmRuntime::new();
    assert_eq!(
        runtime.description(),
        "Fast, disk space efficient package manager"
    );
}

#[rstest]
fn test_pnpm_provider_name() {
    let provider = PnpmProvider::new();
    assert_eq!(provider.name(), "pnpm");
}

#[rstest]
fn test_pnpm_provider_runtimes() {
    let provider = PnpmProvider::new();
    let runtimes = provider.runtimes();
    assert_eq!(runtimes.len(), 1);
    assert_eq!(runtimes[0].name(), "pnpm");
}

#[rstest]
fn test_pnpm_provider_supports() {
    let provider = PnpmProvider::new();
    assert!(provider.supports("pnpm"));
    assert!(!provider.supports("npm"));
}

#[rstest]
fn test_pnpm_provider_get_runtime() {
    let provider = PnpmProvider::new();

    let pnpm = provider.get_runtime("pnpm");
    assert!(pnpm.is_some());
    assert_eq!(pnpm.unwrap().name(), "pnpm");

    let unknown = provider.get_runtime("unknown");
    assert!(unknown.is_none());
}

/// Test that executable_relative_path returns standard name (after post_extract rename)
/// The downloaded file is renamed from platform-specific name to standard name
#[rstest]
#[case(Os::Linux, Arch::X86_64, "pnpm")]
#[case(Os::Linux, Arch::Aarch64, "pnpm")]
#[case(Os::MacOS, Arch::X86_64, "pnpm")]
#[case(Os::MacOS, Arch::Aarch64, "pnpm")]
#[case(Os::Windows, Arch::X86_64, "pnpm.exe")]
#[case(Os::Windows, Arch::Aarch64, "pnpm.exe")]
fn test_pnpm_executable_relative_path(#[case] os: Os, #[case] arch: Arch, #[case] expected: &str) {
    let runtime = PnpmRuntime::new();
    let platform = Platform { os, arch };
    let path = runtime.executable_relative_path("9.0.0", &platform);
    assert_eq!(path, expected);
}

/// Test download URL format - still uses platform-specific naming
#[rstest]
#[case(Os::Linux, Arch::X86_64, "pnpm-linux-x64")]
#[case(Os::Windows, Arch::X86_64, "pnpm-win-x64.exe")]
#[case(Os::MacOS, Arch::Aarch64, "pnpm-macos-arm64")]
#[tokio::test]
async fn test_pnpm_download_url(#[case] os: Os, #[case] arch: Arch, #[case] expected_suffix: &str) {
    let runtime = PnpmRuntime::new();
    let platform = Platform { os, arch };
    let url = runtime.download_url("9.0.0", &platform).await.unwrap();
    assert!(url.is_some());
    let url = url.unwrap();
    assert!(url.contains("github.com/pnpm/pnpm/releases"));
    assert!(url.contains("v9.0.0"));
    assert!(url.ends_with(expected_suffix));
}
