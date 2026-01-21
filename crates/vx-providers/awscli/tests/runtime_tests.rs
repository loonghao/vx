//! Tests for AWS CLI runtime

use rstest::rstest;
use vx_provider_awscli::{AwsCliConfig, AwsCliProvider, AwsCliRuntime, AwsCliUrlBuilder};
use vx_runtime::{Arch, Ecosystem, Os, Platform, Provider, Runtime};

#[test]
fn test_runtime_name() {
    let runtime = AwsCliRuntime::new();
    assert_eq!(runtime.name(), "aws");
}

#[test]
fn test_runtime_description() {
    let runtime = AwsCliRuntime::new();
    assert!(runtime.description().contains("AWS"));
}

#[test]
fn test_runtime_ecosystem() {
    let runtime = AwsCliRuntime::new();
    assert_eq!(runtime.ecosystem(), Ecosystem::Custom("cloud".to_string()));
}

#[test]
fn test_runtime_aliases() {
    let runtime = AwsCliRuntime::new();
    assert!(runtime.aliases().contains(&"awscli"));
    assert!(runtime.aliases().contains(&"aws-cli"));
}

#[test]
fn test_runtime_metadata() {
    let runtime = AwsCliRuntime::new();
    let meta = runtime.metadata();
    assert!(meta.contains_key("homepage"));
    assert!(meta.contains_key("repository"));
    assert_eq!(
        meta.get("homepage"),
        Some(&"https://aws.amazon.com/cli/".to_string())
    );
}

#[test]
fn test_provider_name() {
    let provider = AwsCliProvider::new();
    assert_eq!(provider.name(), "awscli");
}

#[test]
fn test_provider_supports() {
    let provider = AwsCliProvider::new();
    assert!(provider.supports("aws"));
    assert!(!provider.supports("other"));
}

#[test]
fn test_provider_runtimes() {
    let provider = AwsCliProvider::new();
    let runtimes = provider.runtimes();
    assert_eq!(runtimes.len(), 1);
    assert_eq!(runtimes[0].name(), "aws");
}

#[test]
fn test_provider_get_runtime() {
    let provider = AwsCliProvider::new();
    assert!(provider.get_runtime("aws").is_some());
    assert!(provider.get_runtime("other").is_none());
}

#[test]
fn test_config_default() {
    let config = AwsCliConfig::default();
    assert!(config.default_version.is_none());
    assert!(config.install_dir.is_none());
}

#[rstest]
#[case(Os::Linux, Arch::X86_64, "aws/dist/aws")]
#[case(Os::Linux, Arch::Aarch64, "aws/dist/aws")]
#[case(Os::MacOS, Arch::X86_64, "aws-cli/aws")]
#[case(Os::MacOS, Arch::Aarch64, "aws-cli/aws")]
#[case(Os::Windows, Arch::X86_64, "Amazon/AWSCLIV2/aws.exe")]
fn test_executable_relative_path(#[case] os: Os, #[case] arch: Arch, #[case] expected: &str) {
    let runtime = AwsCliRuntime::new();
    let platform = Platform::new(os, arch);
    assert_eq!(
        runtime.executable_relative_path("2.15.0", &platform),
        expected
    );
}

#[rstest]
#[case(Os::Linux, Arch::X86_64, true)]
#[case(Os::Linux, Arch::Aarch64, true)]
#[case(Os::MacOS, Arch::X86_64, true)]
#[case(Os::MacOS, Arch::Aarch64, true)]
#[case(Os::Windows, Arch::X86_64, true)]
#[case(Os::Windows, Arch::Aarch64, false)]
fn test_download_url_availability(#[case] os: Os, #[case] arch: Arch, #[case] available: bool) {
    let platform = Platform::new(os, arch);
    let url = AwsCliUrlBuilder::download_url("2.15.0", &platform);
    assert_eq!(url.is_some(), available);
}

#[rstest]
#[case(Os::Linux, "zip")]
#[case(Os::MacOS, "pkg")]
#[case(Os::Windows, "msi")]
fn test_archive_type(#[case] os: Os, #[case] expected: &str) {
    let platform = Platform::new(os, Arch::X86_64);
    assert_eq!(AwsCliUrlBuilder::archive_type(&platform), expected);
}
