//! Tests for Google Cloud CLI runtime

use rstest::rstest;
use vx_provider_gcloud::{GcloudConfig, GcloudProvider, GcloudRuntime, GcloudUrlBuilder};
use vx_runtime::{Arch, Ecosystem, Os, Platform, Provider, Runtime};

#[test]
fn test_runtime_name() {
    let runtime = GcloudRuntime::new();
    assert_eq!(runtime.name(), "gcloud");
}

#[test]
fn test_runtime_description() {
    let runtime = GcloudRuntime::new();
    assert!(runtime.description().contains("Google Cloud"));
}

#[test]
fn test_runtime_ecosystem() {
    let runtime = GcloudRuntime::new();
    assert_eq!(runtime.ecosystem(), Ecosystem::Custom("cloud".to_string()));
}

#[test]
fn test_runtime_aliases() {
    let runtime = GcloudRuntime::new();
    assert!(runtime.aliases().contains(&"google-cloud-sdk"));
    assert!(runtime.aliases().contains(&"google-cloud-cli"));
}

#[test]
fn test_runtime_metadata() {
    let runtime = GcloudRuntime::new();
    let meta = runtime.metadata();
    assert!(meta.contains_key("homepage"));
    assert!(meta.contains_key("repository"));
    assert_eq!(
        meta.get("homepage"),
        Some(&"https://cloud.google.com/sdk/".to_string())
    );
}

#[test]
fn test_provider_name() {
    let provider = GcloudProvider::new();
    assert_eq!(provider.name(), "gcloud");
}

#[test]
fn test_provider_supports() {
    let provider = GcloudProvider::new();
    assert!(provider.supports("gcloud"));
    assert!(!provider.supports("other"));
}

#[test]
fn test_provider_runtimes() {
    let provider = GcloudProvider::new();
    let runtimes = provider.runtimes();
    // Provider includes gcloud, gsutil, and bq (all bundled in Google Cloud SDK)
    assert_eq!(runtimes.len(), 3);
    let names: Vec<&str> = runtimes.iter().map(|r| r.name()).collect();
    assert!(names.contains(&"gcloud"));
    assert!(names.contains(&"gsutil"));
    assert!(names.contains(&"bq"));
}

#[test]
fn test_provider_get_runtime() {
    let provider = GcloudProvider::new();
    assert!(provider.get_runtime("gcloud").is_some());
    assert!(provider.get_runtime("other").is_none());
}

#[test]
fn test_config_default() {
    let config = GcloudConfig::default();
    assert!(config.default_version.is_none());
    assert!(config.install_dir.is_none());
}

#[rstest]
#[case(Os::Linux, Arch::X86_64, "google-cloud-sdk/bin/gcloud")]
#[case(Os::Linux, Arch::Aarch64, "google-cloud-sdk/bin/gcloud")]
#[case(Os::MacOS, Arch::X86_64, "google-cloud-sdk/bin/gcloud")]
#[case(Os::MacOS, Arch::Aarch64, "google-cloud-sdk/bin/gcloud")]
#[case(Os::Windows, Arch::X86_64, "google-cloud-sdk/bin/gcloud.cmd")]
fn test_executable_relative_path(#[case] os: Os, #[case] arch: Arch, #[case] expected: &str) {
    let runtime = GcloudRuntime::new();
    let platform = Platform::new(os, arch);
    assert_eq!(
        runtime.executable_relative_path("500.0.0", &platform),
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
    let url = GcloudUrlBuilder::download_url("500.0.0", &platform);
    assert_eq!(url.is_some(), available);
}

#[rstest]
#[case(Os::Linux, "tar.gz")]
#[case(Os::MacOS, "tar.gz")]
#[case(Os::Windows, "zip")]
fn test_archive_type(#[case] os: Os, #[case] expected: &str) {
    let platform = Platform::new(os, Arch::X86_64);
    assert_eq!(GcloudUrlBuilder::archive_type(&platform), expected);
}

#[test]
fn test_download_url_format() {
    let platform = Platform::new(Os::Linux, Arch::X86_64);
    let url = GcloudUrlBuilder::download_url("500.0.0", &platform).unwrap();
    assert!(url.contains("dl.google.com"));
    assert!(url.contains("500.0.0"));
    assert!(url.contains("linux"));
    assert!(url.ends_with(".tar.gz"));
}
