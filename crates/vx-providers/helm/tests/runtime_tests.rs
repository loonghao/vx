//! Helm runtime tests

use rstest::rstest;
use vx_provider_helm::{HelmProvider, HelmRuntime};
use vx_runtime::{Arch, Ecosystem, Os, Platform, Provider, Runtime};

#[test]
fn test_helm_runtime_creation() {
    let runtime = HelmRuntime::new();
    assert_eq!(runtime.name(), "helm");
    assert!(!runtime.description().is_empty());
    assert_eq!(runtime.ecosystem(), Ecosystem::Unknown);
}

#[test]
fn test_helm_runtime_description() {
    let runtime = HelmRuntime::new();
    assert!(
        runtime.description().contains("Helm")
            || runtime.description().contains("Kubernetes")
            || runtime.description().contains("package")
    );
}

#[test]
fn test_helm_runtime_metadata() {
    let runtime = HelmRuntime::new();
    let metadata = runtime.metadata();

    assert!(metadata.contains_key("homepage"));
    assert!(metadata.contains_key("ecosystem"));
    assert!(metadata.contains_key("repository"));
    assert!(metadata.contains_key("license"));
    assert_eq!(
        metadata.get("homepage"),
        Some(&"https://helm.sh/".to_string())
    );
    assert_eq!(metadata.get("ecosystem"), Some(&"kubernetes".to_string()));
    assert_eq!(metadata.get("license"), Some(&"Apache-2.0".to_string()));
}

#[test]
fn test_helm_runtime_aliases() {
    let runtime = HelmRuntime::new();
    // Helm has no aliases
    assert!(runtime.aliases().is_empty());
}

#[test]
fn test_helm_provider_creation() {
    let provider = HelmProvider::new();
    assert_eq!(provider.name(), "helm");
    assert!(!provider.description().is_empty());
}

#[test]
fn test_helm_provider_runtimes() {
    let provider = HelmProvider::new();
    let runtimes = provider.runtimes();

    assert_eq!(runtimes.len(), 1);
    assert_eq!(runtimes[0].name(), "helm");
}

#[rstest]
#[case("helm", true)]
#[case("kubectl", false)]
#[case("terraform", false)]
fn test_helm_provider_supports(#[case] name: &str, #[case] expected: bool) {
    let provider = HelmProvider::new();
    assert_eq!(provider.supports(name), expected);
}

#[test]
fn test_helm_provider_get_runtime() {
    let provider = HelmProvider::new();

    let helm = provider.get_runtime("helm");
    assert!(helm.is_some());
    assert_eq!(helm.unwrap().name(), "helm");

    let unknown = provider.get_runtime("unknown");
    assert!(unknown.is_none());
}

/// Test that executable_relative_path returns correct path for Helm archives
/// Helm archives extract to {os}-{arch}/helm
#[rstest]
#[case(Os::Linux, Arch::X86_64, "linux-amd64/helm")]
#[case(Os::Linux, Arch::Aarch64, "linux-arm64/helm")]
#[case(Os::MacOS, Arch::X86_64, "darwin-amd64/helm")]
#[case(Os::MacOS, Arch::Aarch64, "darwin-arm64/helm")]
#[case(Os::Windows, Arch::X86_64, "windows-amd64/helm.exe")]
fn test_helm_executable_relative_path(#[case] os: Os, #[case] arch: Arch, #[case] expected: &str) {
    let runtime = HelmRuntime::new();
    let platform = Platform { os, arch };
    let path = runtime.executable_relative_path("3.13.0", &platform);
    assert_eq!(path, expected);
}

#[tokio::test]
async fn test_helm_download_url_format() {
    let runtime = HelmRuntime::new();
    let platform = Platform {
        os: Os::Linux,
        arch: Arch::X86_64,
    };

    let url = runtime.download_url("3.13.0", &platform).await.unwrap();
    assert!(url.is_some());
    let url = url.unwrap();
    assert!(url.contains("get.helm.sh"));
    assert!(url.contains("helm-v3.13.0"));
    assert!(url.contains("linux-amd64"));
    assert!(url.ends_with(".tar.gz"));
}

#[tokio::test]
async fn test_helm_download_url_macos() {
    let runtime = HelmRuntime::new();
    let platform = Platform {
        os: Os::MacOS,
        arch: Arch::Aarch64,
    };

    let url = runtime.download_url("3.13.0", &platform).await.unwrap();
    assert!(url.is_some());
    let url = url.unwrap();
    assert!(url.contains("darwin-arm64"));
    assert!(url.ends_with(".tar.gz"));
}

#[tokio::test]
async fn test_helm_download_url_windows() {
    let runtime = HelmRuntime::new();
    let platform = Platform {
        os: Os::Windows,
        arch: Arch::X86_64,
    };

    let url = runtime.download_url("3.13.0", &platform).await.unwrap();
    assert!(url.is_some());
    let url = url.unwrap();
    assert!(url.contains("windows-amd64"));
    assert!(url.ends_with(".zip"));
}
