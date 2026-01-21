//! Tests for Azure CLI runtime

use rstest::rstest;
use vx_provider_azcli::{AzCliConfig, AzCliProvider, AzCliRuntime, AzCliUrlBuilder};
use vx_runtime::{Arch, Ecosystem, Os, Platform, Provider, Runtime};

#[test]
fn test_runtime_name() {
    let runtime = AzCliRuntime::new();
    assert_eq!(runtime.name(), "az");
}

#[test]
fn test_runtime_description() {
    let runtime = AzCliRuntime::new();
    assert!(runtime.description().contains("Azure"));
}

#[test]
fn test_runtime_ecosystem() {
    let runtime = AzCliRuntime::new();
    assert_eq!(runtime.ecosystem(), Ecosystem::Custom("cloud".to_string()));
}

#[test]
fn test_runtime_aliases() {
    let runtime = AzCliRuntime::new();
    assert!(runtime.aliases().contains(&"azcli"));
    assert!(runtime.aliases().contains(&"azure-cli"));
}

#[test]
fn test_runtime_metadata() {
    let runtime = AzCliRuntime::new();
    let meta = runtime.metadata();
    assert!(meta.contains_key("homepage"));
    assert!(meta.contains_key("repository"));
}

#[test]
fn test_provider_name() {
    let provider = AzCliProvider::new();
    assert_eq!(provider.name(), "azcli");
}

#[test]
fn test_provider_supports() {
    let provider = AzCliProvider::new();
    assert!(provider.supports("az"));
    assert!(!provider.supports("other"));
}

#[test]
fn test_provider_runtimes() {
    let provider = AzCliProvider::new();
    let runtimes = provider.runtimes();
    assert_eq!(runtimes.len(), 1);
    assert_eq!(runtimes[0].name(), "az");
}

#[test]
fn test_provider_get_runtime() {
    let provider = AzCliProvider::new();
    assert!(provider.get_runtime("az").is_some());
    assert!(provider.get_runtime("other").is_none());
}

#[test]
fn test_config_default() {
    let config = AzCliConfig::default();
    assert!(config.default_version.is_none());
    assert!(config.install_dir.is_none());
}

#[rstest]
#[case(Os::Linux, Arch::X86_64, "bin/az")]
#[case(Os::Linux, Arch::Aarch64, "bin/az")]
#[case(Os::MacOS, Arch::X86_64, "bin/az")]
#[case(Os::MacOS, Arch::Aarch64, "bin/az")]
#[case(Os::Windows, Arch::X86_64, "Microsoft SDKs/Azure/CLI2/wbin/az.cmd")]
fn test_executable_relative_path(#[case] os: Os, #[case] arch: Arch, #[case] expected: &str) {
    let runtime = AzCliRuntime::new();
    let platform = Platform::new(os, arch);
    assert_eq!(
        runtime.executable_relative_path("2.55.0", &platform),
        expected
    );
}

#[rstest]
#[case(Os::Linux, Arch::X86_64, true)]
#[case(Os::Linux, Arch::Aarch64, true)]
#[case(Os::MacOS, Arch::X86_64, true)]
#[case(Os::MacOS, Arch::Aarch64, true)]
#[case(Os::Windows, Arch::X86_64, true)]
#[case(Os::Windows, Arch::Aarch64, true)]
fn test_download_url_availability(#[case] os: Os, #[case] arch: Arch, #[case] available: bool) {
    let platform = Platform::new(os, arch);
    let url = AzCliUrlBuilder::download_url("2.55.0", &platform);
    assert_eq!(url.is_some(), available);
}

#[rstest]
#[case(Os::Linux, "tar.gz")]
#[case(Os::MacOS, "tar.gz")]
#[case(Os::Windows, "msi")]
fn test_archive_type(#[case] os: Os, #[case] expected: &str) {
    let platform = Platform::new(os, Arch::X86_64);
    assert_eq!(AzCliUrlBuilder::archive_type(&platform), expected);
}
