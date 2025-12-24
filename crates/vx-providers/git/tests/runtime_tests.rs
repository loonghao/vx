//! Unit tests for Git runtime.

use rstest::rstest;
use vx_provider_git::{GitProvider, GitRuntime, GitUrlBuilder};
use vx_runtime::{Arch, Ecosystem, Os, Platform, Provider, Runtime};

#[test]
fn test_git_runtime_creation() {
    let runtime = GitRuntime::new();
    assert_eq!(runtime.name(), "git");
    assert_eq!(
        runtime.description(),
        "Git - Distributed version control system"
    );
    assert_eq!(runtime.ecosystem(), Ecosystem::System);
}

#[test]
fn test_git_provider_creation() {
    let provider = GitProvider::new();
    assert_eq!(provider.name(), "git");
    assert_eq!(
        provider.description(),
        "Git version control system support for vx"
    );

    let runtimes = provider.runtimes();
    assert_eq!(runtimes.len(), 1);
    assert_eq!(runtimes[0].name(), "git");
}

#[test]
fn test_git_provider_supports() {
    let provider = GitProvider::new();
    assert!(provider.supports("git"));
    assert!(!provider.supports("node"));
}

#[test]
fn test_git_provider_get_runtime() {
    let provider = GitProvider::new();

    let git_runtime = provider.get_runtime("git");
    assert!(git_runtime.is_some());
    assert_eq!(git_runtime.unwrap().name(), "git");

    let unknown_runtime = provider.get_runtime("unknown");
    assert!(unknown_runtime.is_none());
}

#[rstest]
#[case(Os::Windows, Arch::X86_64, "cmd/git.exe")]
#[case(Os::Windows, Arch::X86, "cmd/git.exe")]
#[case(Os::Linux, Arch::X86_64, "bin/git")]
#[case(Os::MacOS, Arch::X86_64, "bin/git")]
#[case(Os::MacOS, Arch::Aarch64, "bin/git")]
fn test_executable_relative_path(#[case] os: Os, #[case] arch: Arch, #[case] expected: &str) {
    let runtime = GitRuntime::new();
    let platform = Platform { os, arch };
    assert_eq!(
        runtime.executable_relative_path("2.43.0", &platform),
        expected
    );
}

#[rstest]
#[case(Os::Windows, Arch::X86_64, Some("MinGit-2.43.0-64-bit.zip".to_string()))]
#[case(Os::Windows, Arch::X86, Some("MinGit-2.43.0-32-bit.zip".to_string()))]
#[case(Os::Linux, Arch::X86_64, None)]
#[case(Os::MacOS, Arch::X86_64, None)]
fn test_get_filename(#[case] os: Os, #[case] arch: Arch, #[case] expected: Option<String>) {
    let platform = Platform { os, arch };
    assert_eq!(GitUrlBuilder::get_filename("2.43.0", &platform), expected);
}

#[rstest]
#[case(Os::Windows, Arch::X86_64, "x86_64-pc-windows-msvc")]
#[case(Os::MacOS, Arch::Aarch64, "aarch64-apple-darwin")]
#[case(Os::Linux, Arch::X86_64, "x86_64-unknown-linux-gnu")]
fn test_get_target_triple(#[case] os: Os, #[case] arch: Arch, #[case] expected: &str) {
    let platform = Platform { os, arch };
    assert_eq!(GitUrlBuilder::get_target_triple(&platform), expected);
}

#[rstest]
#[case(Os::Windows, "zip")]
#[case(Os::Linux, "tar.gz")]
#[case(Os::MacOS, "tar.gz")]
fn test_get_archive_extension(#[case] os: Os, #[case] expected: &str) {
    let platform = Platform {
        os,
        arch: Arch::X86_64,
    };
    assert_eq!(GitUrlBuilder::get_archive_extension(&platform), expected);
}

#[rstest]
#[case(Os::Windows, "git.exe")]
#[case(Os::Linux, "git")]
#[case(Os::MacOS, "git")]
fn test_get_executable_name(#[case] os: Os, #[case] expected: &str) {
    let platform = Platform {
        os,
        arch: Arch::X86_64,
    };
    assert_eq!(GitUrlBuilder::get_executable_name(&platform), expected);
}

#[test]
fn test_download_url_windows() {
    let platform = Platform {
        os: Os::Windows,
        arch: Arch::X86_64,
    };
    let url = GitUrlBuilder::download_url("2.43.0.windows.1", &platform);
    assert!(url.is_some());
    let url = url.unwrap();
    assert!(url.contains("git-for-windows"));
    assert!(url.contains("MinGit-2.43.0-64-bit.zip"));
}

#[test]
fn test_download_url_linux() {
    let platform = Platform {
        os: Os::Linux,
        arch: Arch::X86_64,
    };
    let url = GitUrlBuilder::download_url("2.43.0", &platform);
    // Linux should return None as Git should be installed via system package manager
    assert!(url.is_none());
}

#[test]
fn test_extract_base_version() {
    // Test that Windows version strings are properly parsed
    let platform = Platform {
        os: Os::Windows,
        arch: Arch::X86_64,
    };

    // Full Windows version
    let url = GitUrlBuilder::download_url("2.43.0.windows.1", &platform);
    assert!(url.is_some());
    assert!(url.unwrap().contains("MinGit-2.43.0-64-bit.zip"));

    // Plain version
    let url = GitUrlBuilder::download_url("2.43.0", &platform);
    assert!(url.is_some());
    assert!(url.unwrap().contains("MinGit-2.43.0-64-bit.zip"));
}

#[test]
fn test_git_runtime_metadata() {
    let runtime = GitRuntime::new();
    let metadata = runtime.metadata();

    assert!(metadata.contains_key("homepage"));
    assert!(metadata.contains_key("repository"));
    assert!(metadata.contains_key("documentation"));
    assert_eq!(metadata.get("homepage").unwrap(), "https://git-scm.com/");
}
