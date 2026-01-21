//! Terraform runtime tests

use rstest::rstest;
use vx_provider_terraform::{TerraformProvider, TerraformRuntime};
use vx_runtime::{Arch, Ecosystem, Os, Platform, Provider, Runtime};

#[test]
fn test_terraform_runtime_creation() {
    let runtime = TerraformRuntime::new();
    assert_eq!(runtime.name(), "terraform");
    assert!(!runtime.description().is_empty());
    assert_eq!(runtime.ecosystem(), Ecosystem::Custom("devops".to_string()));
}

#[test]
fn test_terraform_runtime_description() {
    let runtime = TerraformRuntime::new();
    assert!(
        runtime.description().contains("Terraform")
            || runtime.description().contains("Infrastructure")
    );
}

#[test]
fn test_terraform_runtime_metadata() {
    let runtime = TerraformRuntime::new();
    let metadata = runtime.metadata();

    assert!(metadata.contains_key("homepage"));
    assert!(metadata.contains_key("ecosystem"));
    assert!(metadata.contains_key("repository"));
    assert!(metadata.contains_key("license"));
    assert_eq!(
        metadata.get("homepage"),
        Some(&"https://www.terraform.io/".to_string())
    );
    assert_eq!(metadata.get("ecosystem"), Some(&"devops".to_string()));
}

#[test]
fn test_terraform_runtime_aliases() {
    let runtime = TerraformRuntime::new();
    assert!(runtime.aliases().contains(&"tf"));
}

#[test]
fn test_terraform_provider_creation() {
    let provider = TerraformProvider::new();
    assert_eq!(provider.name(), "terraform");
    assert!(!provider.description().is_empty());
}

#[test]
fn test_terraform_provider_runtimes() {
    let provider = TerraformProvider::new();
    let runtimes = provider.runtimes();

    assert_eq!(runtimes.len(), 1);
    assert_eq!(runtimes[0].name(), "terraform");
}

#[rstest]
#[case("terraform", true)]
#[case("tf", true)]
#[case("kubectl", false)]
#[case("helm", false)]
fn test_terraform_provider_supports(#[case] name: &str, #[case] expected: bool) {
    let provider = TerraformProvider::new();
    assert_eq!(provider.supports(name), expected);
}

#[test]
fn test_terraform_provider_get_runtime() {
    let provider = TerraformProvider::new();

    let terraform = provider.get_runtime("terraform");
    assert!(terraform.is_some());
    assert_eq!(terraform.unwrap().name(), "terraform");

    let tf = provider.get_runtime("tf");
    assert!(tf.is_some());
    assert_eq!(tf.unwrap().name(), "terraform");

    let unknown = provider.get_runtime("unknown");
    assert!(unknown.is_none());
}

/// Test that executable_relative_path returns correct path for Terraform
/// Terraform archives extract directly to the binary (no subdirectory)
#[rstest]
#[case(Os::Linux, Arch::X86_64, "terraform")]
#[case(Os::Linux, Arch::Aarch64, "terraform")]
#[case(Os::MacOS, Arch::X86_64, "terraform")]
#[case(Os::MacOS, Arch::Aarch64, "terraform")]
#[case(Os::Windows, Arch::X86_64, "terraform.exe")]
fn test_terraform_executable_relative_path(
    #[case] os: Os,
    #[case] arch: Arch,
    #[case] expected: &str,
) {
    let runtime = TerraformRuntime::new();
    let platform = Platform::new(os, arch);
    let path = runtime.executable_relative_path("1.6.0", &platform);
    assert_eq!(path, expected);
}

#[tokio::test]
async fn test_terraform_download_url_format() {
    let runtime = TerraformRuntime::new();
    let platform = Platform::new(Os::Linux, Arch::X86_64);

    let url = runtime.download_url("1.6.0", &platform).await.unwrap();
    assert!(url.is_some());
    let url = url.unwrap();
    assert!(url.contains("releases.hashicorp.com/terraform"));
    assert!(url.contains("1.6.0"));
    assert!(url.contains("linux_amd64"));
    assert!(url.ends_with(".zip"));
}

#[tokio::test]
async fn test_terraform_download_url_macos() {
    let runtime = TerraformRuntime::new();
    let platform = Platform::new(Os::MacOS, Arch::Aarch64);

    let url = runtime.download_url("1.6.0", &platform).await.unwrap();
    assert!(url.is_some());
    let url = url.unwrap();
    assert!(url.contains("darwin_arm64"));
}

#[tokio::test]
async fn test_terraform_download_url_windows() {
    let runtime = TerraformRuntime::new();
    let platform = Platform::new(Os::Windows, Arch::X86_64);

    let url = runtime.download_url("1.6.0", &platform).await.unwrap();
    assert!(url.is_some());
    let url = url.unwrap();
    assert!(url.contains("windows_amd64"));
}
