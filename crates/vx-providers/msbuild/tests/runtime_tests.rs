//! Tests for MSBuild runtime

use rstest::rstest;
use vx_provider_msbuild::{MsbuildProvider, MsbuildRuntime};
use vx_runtime::{Ecosystem, Provider, Runtime};

#[test]
fn test_runtime_name() {
    let runtime = MsbuildRuntime::new();
    assert_eq!(runtime.name(), "msbuild");
}

#[test]
fn test_runtime_description() {
    let runtime = MsbuildRuntime::new();
    assert!(runtime.description().contains("Build Engine"));
}

#[test]
fn test_runtime_ecosystem() {
    let runtime = MsbuildRuntime::new();
    assert_eq!(runtime.ecosystem(), Ecosystem::System);
}

#[test]
fn test_runtime_aliases() {
    let runtime = MsbuildRuntime::new();
    assert!(runtime.aliases().contains(&"msbuild.exe"));
}

#[test]
fn test_runtime_metadata() {
    let runtime = MsbuildRuntime::new();
    let meta = runtime.metadata();
    assert!(meta.contains_key("homepage"));
    assert!(meta.contains_key("bundled_with"));
    assert_eq!(meta.get("bundled_with"), Some(&"dotnet".to_string()));
}

#[test]
fn test_provider_name() {
    let provider = MsbuildProvider::new();
    assert_eq!(provider.name(), "msbuild");
}

#[test]
fn test_provider_description() {
    let provider = MsbuildProvider::new();
    assert!(provider.description().contains("Build Engine"));
}

#[test]
fn test_provider_supports() {
    let provider = MsbuildProvider::new();
    assert!(provider.supports("msbuild"));
    assert!(provider.supports("msbuild.exe"));
    assert!(!provider.supports("dotnet"));
    assert!(!provider.supports("other"));
}

#[test]
fn test_provider_runtimes() {
    let provider = MsbuildProvider::new();
    let runtimes = provider.runtimes();
    assert_eq!(runtimes.len(), 1);
    assert_eq!(runtimes[0].name(), "msbuild");
}

#[test]
fn test_provider_get_runtime() {
    let provider = MsbuildProvider::new();
    assert!(provider.get_runtime("msbuild").is_some());
    assert!(provider.get_runtime("msbuild.exe").is_some());
    assert!(provider.get_runtime("other").is_none());
}

/// RFC 0028: MSBuild is not directly installable
#[test]
fn test_is_version_installable() {
    let runtime = MsbuildRuntime::new();
    // All versions should return false - MSBuild is bundled
    assert!(!runtime.is_version_installable("17.0.0"));
    assert!(!runtime.is_version_installable("16.0.0"));
    assert!(!runtime.is_version_installable("latest"));
}

/// RFC 0028: download_url should return None for bundled tools
#[rstest]
#[tokio::test]
async fn test_download_url_returns_none() {
    use vx_runtime::{Arch, Os, Platform};

    let runtime = MsbuildRuntime::new();
    let platforms = [
        Platform::new(Os::Windows, Arch::X86_64),
        Platform::new(Os::Linux, Arch::X86_64),
        Platform::new(Os::MacOS, Arch::Aarch64),
    ];

    for platform in &platforms {
        let url = runtime.download_url("17.0.0", platform).await.unwrap();
        assert!(url.is_none(), "MSBuild should not have direct download URL");
    }
}
