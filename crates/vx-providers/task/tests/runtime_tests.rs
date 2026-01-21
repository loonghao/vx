//! Tests for Task runtime

use rstest::rstest;
use vx_provider_task::{TaskProvider, TaskRuntime, TaskUrlBuilder};
use vx_runtime::{Arch, Ecosystem, Os, Platform, Provider, Runtime};

#[test]
fn test_runtime_name() {
    let runtime = TaskRuntime::new();
    assert_eq!(runtime.name(), "task");
}

#[test]
fn test_runtime_description() {
    let runtime = TaskRuntime::new();
    assert!(runtime.description().contains("Task"));
}

#[test]
fn test_runtime_ecosystem() {
    let runtime = TaskRuntime::new();
    assert_eq!(runtime.ecosystem(), Ecosystem::System);
}

#[test]
fn test_runtime_metadata() {
    let runtime = TaskRuntime::new();
    let meta = runtime.metadata();
    assert!(meta.contains_key("homepage"));
    assert!(meta.contains_key("repository"));
}

#[test]
fn test_provider_name() {
    let provider = TaskProvider::new();
    assert_eq!(provider.name(), "task");
}

#[test]
fn test_provider_supports() {
    let provider = TaskProvider::new();
    assert!(provider.supports("task"));
    assert!(!provider.supports("other"));
}

#[test]
fn test_provider_runtimes() {
    let provider = TaskProvider::new();
    let runtimes = provider.runtimes();
    assert_eq!(runtimes.len(), 1);
    assert_eq!(runtimes[0].name(), "task");
}

#[test]
fn test_provider_get_runtime() {
    let provider = TaskProvider::new();
    assert!(provider.get_runtime("task").is_some());
    assert!(provider.get_runtime("other").is_none());
}

#[rstest]
#[case(Os::Linux, Arch::X86_64, Some(("linux", "amd64")))]
#[case(Os::Linux, Arch::Aarch64, Some(("linux", "arm64")))]
#[case(Os::MacOS, Arch::X86_64, Some(("darwin", "amd64")))]
#[case(Os::MacOS, Arch::Aarch64, Some(("darwin", "arm64")))]
#[case(Os::Windows, Arch::X86_64, Some(("windows", "amd64")))]
#[case(Os::Windows, Arch::Aarch64, Some(("windows", "arm64")))]
fn test_platform_parts(#[case] os: Os, #[case] arch: Arch, #[case] expected: Option<(&str, &str)>) {
    let platform = Platform::new(os, arch);
    assert_eq!(TaskUrlBuilder::get_platform_parts(&platform), expected);
}

#[rstest]
#[case(Os::Windows, "zip")]
#[case(Os::Linux, "tar.gz")]
#[case(Os::MacOS, "tar.gz")]
fn test_archive_extension(#[case] os: Os, #[case] expected: &str) {
    let platform = Platform::new(os, Arch::X86_64);
    assert_eq!(TaskUrlBuilder::get_archive_extension(&platform), expected);
}

#[rstest]
#[case(Os::Windows, "task.exe")]
#[case(Os::Linux, "task")]
#[case(Os::MacOS, "task")]
fn test_executable_name(#[case] os: Os, #[case] expected: &str) {
    let platform = Platform::new(os, Arch::X86_64);
    assert_eq!(TaskUrlBuilder::get_executable_name(&platform), expected);
}

#[test]
fn test_download_url_format() {
    let platform = Platform::new(Os::Linux, Arch::X86_64);
    let url = TaskUrlBuilder::download_url("3.40.1", &platform).unwrap();
    assert!(url.contains("github.com/go-task/task"));
    assert!(url.contains("v3.40.1"));
    assert!(url.contains("task_linux_amd64"));
    assert!(url.ends_with(".tar.gz"));
}

#[test]
fn test_download_url_with_v_prefix() {
    let platform = Platform::new(Os::Linux, Arch::X86_64);
    let url1 = TaskUrlBuilder::download_url("3.40.1", &platform);
    let url2 = TaskUrlBuilder::download_url("v3.40.1", &platform);
    // Both should produce the same URL
    assert_eq!(url1, url2);
}
