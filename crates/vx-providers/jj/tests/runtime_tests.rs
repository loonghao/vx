//! Tests for jj runtime

use rstest::rstest;
use vx_provider_jj::JjUrlBuilder;
use vx_runtime::{Arch, Os, Platform};

#[rstest]
#[case::linux_x64(
    Os::Linux,
    Arch::X86_64,
    "0.38.0",
    "x86_64-unknown-linux-musl",
    "tar.gz"
)]
#[case::linux_arm64(
    Os::Linux,
    Arch::Aarch64,
    "0.38.0",
    "aarch64-unknown-linux-musl",
    "tar.gz"
)]
#[case::macos_x64(Os::MacOS, Arch::X86_64, "0.38.0", "x86_64-apple-darwin", "tar.gz")]
#[case::macos_arm64(Os::MacOS, Arch::Aarch64, "0.38.0", "aarch64-apple-darwin", "tar.gz")]
#[case::windows_x64(Os::Windows, Arch::X86_64, "0.38.0", "x86_64-pc-windows-msvc", "zip")]
#[case::windows_arm64(Os::Windows, Arch::Aarch64, "0.38.0", "aarch64-pc-windows-msvc", "zip")]
fn test_download_url(
    #[case] os: Os,
    #[case] arch: Arch,
    #[case] version: &str,
    #[case] expected_target: &str,
    #[case] expected_ext: &str,
) {
    let platform = Platform::new(os, arch);
    let url = JjUrlBuilder::download_url(version, &platform);

    // Version without 'v' prefix in storage, URL adds 'v' prefix
    let expected_url = format!(
        "https://github.com/jj-vcs/jj/releases/download/v{}/jj-v{}-{}.{}",
        version, version, expected_target, expected_ext
    );
    assert_eq!(url, Some(expected_url));
}

#[rstest]
#[case::windows(Os::Windows, "zip")]
#[case::linux(Os::Linux, "tar.gz")]
#[case::macos(Os::MacOS, "tar.gz")]
fn test_archive_extension(#[case] os: Os, #[case] expected: &str) {
    let platform = Platform::new(os, Arch::X86_64);
    assert_eq!(JjUrlBuilder::get_archive_extension(&platform), expected);
}

#[rstest]
#[case::windows(Os::Windows, "jj.exe")]
#[case::linux(Os::Linux, "jj")]
#[case::macos(Os::MacOS, "jj")]
fn test_executable_name(#[case] os: Os, #[case] expected: &str) {
    let platform = Platform::new(os, Arch::X86_64);
    assert_eq!(JjUrlBuilder::get_executable_name(&platform), expected);
}
