//! Tests for Docker runtime

use rstest::rstest;
use vx_provider_docker::{DockerConfig, DockerProvider, DockerRuntime};
use vx_runtime::{Arch, Ecosystem, Os, Platform, Provider, Runtime};

#[test]
fn test_runtime_name() {
    let runtime = DockerRuntime::new();
    assert_eq!(runtime.name(), "docker");
}

#[test]
fn test_runtime_description() {
    let runtime = DockerRuntime::new();
    assert!(runtime.description().contains("Docker"));
}

#[test]
fn test_runtime_ecosystem() {
    let runtime = DockerRuntime::new();
    assert_eq!(
        runtime.ecosystem(),
        Ecosystem::Custom("container".to_string())
    );
}

#[test]
fn test_runtime_aliases() {
    let runtime = DockerRuntime::new();
    assert!(runtime.aliases().contains(&"docker-cli"));
}

#[test]
fn test_runtime_metadata() {
    let runtime = DockerRuntime::new();
    let meta = runtime.metadata();
    assert!(meta.contains_key("homepage"));
    assert!(meta.contains_key("repository"));
    assert_eq!(
        meta.get("homepage"),
        Some(&"https://www.docker.com/".to_string())
    );
}

#[test]
fn test_provider_name() {
    let provider = DockerProvider::new();
    assert_eq!(provider.name(), "docker");
}

#[test]
fn test_provider_supports() {
    let provider = DockerProvider::new();
    assert!(provider.supports("docker"));
    assert!(!provider.supports("other"));
}

#[test]
fn test_provider_runtimes() {
    let provider = DockerProvider::new();
    let runtimes = provider.runtimes();
    assert_eq!(runtimes.len(), 1);
    assert_eq!(runtimes[0].name(), "docker");
}

#[test]
fn test_provider_get_runtime() {
    let provider = DockerProvider::new();
    assert!(provider.get_runtime("docker").is_some());
    assert!(provider.get_runtime("other").is_none());
}

#[test]
fn test_config_default() {
    let config = DockerConfig::default();
    assert!(config.default_version.is_none());
    assert!(config.install_dir.is_none());
}

#[rstest]
#[case(Os::Linux, Arch::X86_64, "docker/docker")]
#[case(Os::Linux, Arch::Aarch64, "docker/docker")]
#[case(Os::MacOS, Arch::X86_64, "docker/docker")]
#[case(Os::MacOS, Arch::Aarch64, "docker/docker")]
#[case(Os::Windows, Arch::X86_64, "docker/docker.exe")]
fn test_executable_relative_path(#[case] os: Os, #[case] arch: Arch, #[case] expected: &str) {
    let runtime = DockerRuntime::new();
    let platform = Platform::new(os, arch);
    assert_eq!(
        runtime.executable_relative_path("27.0.0", &platform),
        expected
    );
}
