//! Unit tests for rcedit runtime
//!
//! Tests for the rcedit provider implementation.

use rstest::rstest;
use vx_runtime::{Arch, Os, Platform};

// Import the config module for URL builder tests
use vx_provider_rcedit::RceditUrlBuilder;

#[rstest]
#[case(
    "2.0.0",
    Arch::X86_64,
    "https://github.com/electron/rcedit/releases/download/v2.0.0/rcedit-x64.exe"
)]
#[case(
    "1.1.1",
    Arch::X86_64,
    "https://github.com/electron/rcedit/releases/download/v1.1.1/rcedit-x64.exe"
)]
#[case(
    "2.0.0",
    Arch::Aarch64,
    "https://github.com/electron/rcedit/releases/download/v2.0.0/rcedit-arm64.exe"
)]
fn test_download_url_windows(#[case] version: &str, #[case] arch: Arch, #[case] expected: &str) {
    let platform = Platform {
        os: Os::Windows,
        arch,
    };
    let url = RceditUrlBuilder::download_url(version, &platform);
    assert_eq!(url, Some(expected.to_string()));
}

#[rstest]
#[case(Os::Linux, Arch::X86_64)]
#[case(Os::MacOS, Arch::X86_64)]
#[case(Os::MacOS, Arch::Aarch64)]
fn test_download_url_unsupported_platforms(#[case] os: Os, #[case] arch: Arch) {
    let platform = Platform { os, arch };
    let url = RceditUrlBuilder::download_url("2.0.0", &platform);
    assert_eq!(url, None);
}

#[rstest]
#[case(Arch::X86_64, "rcedit-x64.exe")]
#[case(Arch::Aarch64, "rcedit-arm64.exe")]
#[case(Arch::X86, "rcedit-x86.exe")]
fn test_executable_name(#[case] arch: Arch, #[case] expected: &str) {
    let platform = Platform {
        os: Os::Windows,
        arch,
    };
    assert_eq!(RceditUrlBuilder::get_executable_name(&platform), expected);
}

#[rstest]
#[case(Os::Windows, Arch::X86_64, true)]
#[case(Os::Windows, Arch::Aarch64, true)]
#[case(Os::Windows, Arch::X86, true)]
#[case(Os::Linux, Arch::X86_64, false)]
#[case(Os::MacOS, Arch::Aarch64, false)]
fn test_platform_support(#[case] os: Os, #[case] arch: Arch, #[case] expected: bool) {
    let platform = Platform { os, arch };
    assert_eq!(RceditUrlBuilder::is_platform_supported(&platform), expected);
}

#[rstest]
#[case(Arch::X86_64, Some("x64"))]
#[case(Arch::Aarch64, Some("arm64"))]
#[case(Arch::X86, Some("x86"))]
fn test_arch_suffix(#[case] arch: Arch, #[case] expected: Option<&str>) {
    let platform = Platform {
        os: Os::Windows,
        arch,
    };
    assert_eq!(RceditUrlBuilder::get_arch_suffix(&platform), expected);
}
