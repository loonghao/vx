//! Go runtime tests

use rstest::rstest;
use vx_provider_go::{GoProvider, GoRuntime};
use vx_runtime::{Arch, Ecosystem, Os, Platform, Provider, Runtime};

#[test]
fn test_go_runtime_creation() {
    let runtime = GoRuntime::new();
    assert_eq!(runtime.name(), "go");
    assert!(!runtime.description().is_empty());
    assert!(runtime.aliases().contains(&"golang"));
    assert_eq!(runtime.ecosystem(), Ecosystem::Go);
}

#[test]
fn test_go_runtime_metadata() {
    let runtime = GoRuntime::new();
    let metadata = runtime.metadata();

    assert!(metadata.contains_key("homepage"));
    assert!(metadata.contains_key("ecosystem"));
    assert_eq!(metadata.get("ecosystem"), Some(&"go".to_string()));
}

#[test]
fn test_go_provider_creation() {
    let provider = GoProvider::new();
    assert_eq!(provider.name(), "go");
    assert!(!provider.description().is_empty());
}

#[test]
fn test_go_provider_runtimes() {
    let provider = GoProvider::new();
    let runtimes = provider.runtimes();

    assert_eq!(runtimes.len(), 1);
    assert_eq!(runtimes[0].name(), "go");
}

#[rstest]
#[case("go", true)]
#[case("golang", true)]
#[case("node", false)]
#[case("python", false)]
fn test_go_provider_supports(#[case] name: &str, #[case] expected: bool) {
    let provider = GoProvider::new();
    assert_eq!(provider.supports(name), expected);
}

#[test]
fn test_go_provider_get_runtime() {
    let provider = GoProvider::new();

    let go = provider.get_runtime("go");
    assert!(go.is_some());
    assert_eq!(go.unwrap().name(), "go");

    let golang = provider.get_runtime("golang");
    assert!(golang.is_some());
    assert_eq!(golang.unwrap().name(), "go");

    let unknown = provider.get_runtime("unknown");
    assert!(unknown.is_none());
}

/// Test that executable_relative_path returns correct path for Go archives
/// Go archives extract to a `go/` subdirectory, so path should be `go/bin/go`
#[rstest]
#[case(Os::Linux, Arch::X86_64, "go/bin/go")]
#[case(Os::Linux, Arch::Aarch64, "go/bin/go")]
#[case(Os::MacOS, Arch::X86_64, "go/bin/go")]
#[case(Os::MacOS, Arch::Aarch64, "go/bin/go")]
#[case(Os::Windows, Arch::X86_64, "go/bin/go.exe")]
#[case(Os::Windows, Arch::Aarch64, "go/bin/go.exe")]
fn test_go_executable_relative_path(#[case] os: Os, #[case] arch: Arch, #[case] expected: &str) {
    let runtime = GoRuntime::new();
    let platform = Platform::new(os, arch);
    let path = runtime.executable_relative_path("1.21.0", &platform);
    assert_eq!(path, expected);
}
