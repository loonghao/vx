//! Tests for Spack runtime

use rstest::rstest;
use vx_provider_spack::{SpackProvider, SpackRuntime, SpackUrlBuilder};
use vx_runtime::{Arch, Ecosystem, Os, Platform, Provider, Runtime};

#[test]
fn test_runtime_name() {
    let runtime = SpackRuntime::new();
    assert_eq!(runtime.name(), "spack");
}

#[test]
fn test_runtime_description() {
    let runtime = SpackRuntime::new();
    assert!(runtime.description().contains("Spack"));
    assert!(
        runtime.description().contains("HPC") || runtime.description().contains("package manager")
    );
}

#[test]
fn test_runtime_ecosystem() {
    let runtime = SpackRuntime::new();
    assert_eq!(runtime.ecosystem(), Ecosystem::System);
}

#[test]
fn test_runtime_metadata() {
    let runtime = SpackRuntime::new();
    let meta = runtime.metadata();
    assert!(meta.contains_key("homepage"));
    assert!(meta.contains_key("repository"));
    assert!(meta.contains_key("documentation"));
    assert!(meta.contains_key("category"));
    assert_eq!(meta.get("category"), Some(&"hpc".to_string()));
    assert_eq!(meta.get("homepage"), Some(&"https://spack.io".to_string()));
}

#[test]
fn test_provider_name() {
    let provider = SpackProvider::new();
    assert_eq!(provider.name(), "spack");
}

#[test]
fn test_provider_description() {
    let provider = SpackProvider::new();
    assert!(provider.description().contains("Spack"));
}

#[test]
fn test_provider_supports() {
    let provider = SpackProvider::new();
    assert!(provider.supports("spack"));
    assert!(!provider.supports("other"));
    assert!(!provider.supports("conda"));
}

#[test]
fn test_provider_runtimes() {
    let provider = SpackProvider::new();
    let runtimes = provider.runtimes();
    assert_eq!(runtimes.len(), 1);
    assert_eq!(runtimes[0].name(), "spack");
}

#[test]
fn test_provider_get_runtime() {
    let provider = SpackProvider::new();
    assert!(provider.get_runtime("spack").is_some());
    assert!(provider.get_runtime("other").is_none());
}

#[test]
fn test_download_url_format() {
    let url = SpackUrlBuilder::download_url("1.1.0").unwrap();
    assert!(url.contains("github.com/spack/spack"));
    assert!(url.contains("v1.1.0"));
    assert!(url.contains("spack-1.1.0"));
    assert!(url.ends_with(".tar.gz"));
}

#[test]
fn test_download_url_with_v_prefix() {
    let url = SpackUrlBuilder::download_url("v1.1.0").unwrap();
    // Should not double the 'v' prefix
    assert!(url.contains("/v1.1.0/"));
    assert!(!url.contains("/vv1.1.0/"));
    assert!(url.contains("spack-1.1.0.tar.gz"));
}

#[rstest]
#[case(Os::Linux, "spack")]
#[case(Os::MacOS, "spack")]
#[case(Os::Windows, "spack")]
fn test_executable_name(#[case] os: Os, #[case] expected: &str) {
    let platform = Platform::new(os, Arch::X86_64);
    let name = SpackUrlBuilder::get_executable_name(&platform);
    assert_eq!(name, expected);
}

#[rstest]
#[case(Os::Linux, Arch::X86_64)]
#[case(Os::Linux, Arch::Aarch64)]
#[case(Os::MacOS, Arch::X86_64)]
#[case(Os::MacOS, Arch::Aarch64)]
#[case(Os::Windows, Arch::X86_64)]
fn test_executable_relative_path(#[case] os: Os, #[case] arch: Arch) {
    let runtime = SpackRuntime::new();
    let platform = Platform::new(os, arch);
    let path = runtime.executable_relative_path("1.1.0", &platform);
    assert_eq!(path, "spack-1.1.0/bin/spack");
}

#[test]
fn test_archive_extension() {
    assert_eq!(SpackUrlBuilder::get_archive_extension(), "tar.gz");
}
