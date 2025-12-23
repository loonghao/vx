//! kubectl runtime tests

use rstest::rstest;
use vx_provider_kubectl::{KubectlProvider, KubectlRuntime};
use vx_runtime::{Arch, Ecosystem, Os, Platform, Provider, Runtime};

#[test]
fn test_kubectl_runtime_creation() {
    let runtime = KubectlRuntime::new();
    assert_eq!(runtime.name(), "kubectl");
    assert!(!runtime.description().is_empty());
    assert_eq!(
        runtime.ecosystem(),
        Ecosystem::Custom("kubernetes".to_string())
    );
}

#[test]
fn test_kubectl_runtime_description() {
    let runtime = KubectlRuntime::new();
    assert!(
        runtime.description().contains("kubectl") || runtime.description().contains("Kubernetes")
    );
}

#[test]
fn test_kubectl_runtime_metadata() {
    let runtime = KubectlRuntime::new();
    let metadata = runtime.metadata();

    assert!(metadata.contains_key("homepage"));
    assert!(metadata.contains_key("ecosystem"));
    assert!(metadata.contains_key("repository"));
    assert!(metadata.contains_key("license"));
    assert_eq!(metadata.get("ecosystem"), Some(&"kubernetes".to_string()));
    assert_eq!(metadata.get("license"), Some(&"Apache-2.0".to_string()));
}

#[test]
fn test_kubectl_runtime_aliases() {
    let runtime = KubectlRuntime::new();
    let aliases = runtime.aliases();
    assert!(aliases.contains(&"kube"));
    assert!(aliases.contains(&"k8s"));
}

#[test]
fn test_kubectl_provider_creation() {
    let provider = KubectlProvider::new();
    assert_eq!(provider.name(), "kubectl");
    assert!(!provider.description().is_empty());
}

#[test]
fn test_kubectl_provider_runtimes() {
    let provider = KubectlProvider::new();
    let runtimes = provider.runtimes();

    assert_eq!(runtimes.len(), 1);
    assert_eq!(runtimes[0].name(), "kubectl");
}

#[rstest]
#[case("kubectl", true)]
#[case("kube", true)]
#[case("k8s", true)]
#[case("helm", false)]
#[case("terraform", false)]
fn test_kubectl_provider_supports(#[case] name: &str, #[case] expected: bool) {
    let provider = KubectlProvider::new();
    assert_eq!(provider.supports(name), expected);
}

#[test]
fn test_kubectl_provider_get_runtime() {
    let provider = KubectlProvider::new();

    let kubectl = provider.get_runtime("kubectl");
    assert!(kubectl.is_some());
    assert_eq!(kubectl.unwrap().name(), "kubectl");

    let kube = provider.get_runtime("kube");
    assert!(kube.is_some());
    assert_eq!(kube.unwrap().name(), "kubectl");

    let k8s = provider.get_runtime("k8s");
    assert!(k8s.is_some());
    assert_eq!(k8s.unwrap().name(), "kubectl");

    let unknown = provider.get_runtime("unknown");
    assert!(unknown.is_none());
}

/// Test that executable_relative_path returns correct path for kubectl
/// kubectl is a single binary download (no archive structure)
#[rstest]
#[case(Os::Linux, Arch::X86_64, "kubectl")]
#[case(Os::Linux, Arch::Aarch64, "kubectl")]
#[case(Os::MacOS, Arch::X86_64, "kubectl")]
#[case(Os::MacOS, Arch::Aarch64, "kubectl")]
#[case(Os::Windows, Arch::X86_64, "kubectl.exe")]
fn test_kubectl_executable_relative_path(
    #[case] os: Os,
    #[case] arch: Arch,
    #[case] expected: &str,
) {
    let runtime = KubectlRuntime::new();
    let platform = Platform { os, arch };
    let path = runtime.executable_relative_path("1.28.0", &platform);
    assert_eq!(path, expected);
}

#[tokio::test]
async fn test_kubectl_download_url_format() {
    let runtime = KubectlRuntime::new();
    let platform = Platform {
        os: Os::Linux,
        arch: Arch::X86_64,
    };

    let url = runtime.download_url("1.28.0", &platform).await.unwrap();
    assert!(url.is_some());
    let url = url.unwrap();
    assert!(url.contains("dl.k8s.io/release"));
    assert!(url.contains("v1.28.0"));
    assert!(url.contains("linux/amd64"));
    assert!(url.ends_with("kubectl"));
}

#[tokio::test]
async fn test_kubectl_download_url_macos() {
    let runtime = KubectlRuntime::new();
    let platform = Platform {
        os: Os::MacOS,
        arch: Arch::Aarch64,
    };

    let url = runtime.download_url("1.28.0", &platform).await.unwrap();
    assert!(url.is_some());
    let url = url.unwrap();
    assert!(url.contains("darwin/arm64"));
}

#[tokio::test]
async fn test_kubectl_download_url_windows() {
    let runtime = KubectlRuntime::new();
    let platform = Platform {
        os: Os::Windows,
        arch: Arch::X86_64,
    };

    let url = runtime.download_url("1.28.0", &platform).await.unwrap();
    assert!(url.is_some());
    let url = url.unwrap();
    assert!(url.contains("windows/amd64"));
    assert!(url.ends_with("kubectl.exe"));
}
