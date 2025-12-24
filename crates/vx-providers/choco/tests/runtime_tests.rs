//! Chocolatey runtime tests

use rstest::rstest;
use vx_provider_choco::{ChocoProvider, ChocoRuntime};
use vx_runtime::{Ecosystem, Provider, Runtime};

#[rstest]
fn test_choco_runtime_name() {
    let runtime = ChocoRuntime::new();
    assert_eq!(runtime.name(), "choco");
}

#[rstest]
fn test_choco_runtime_ecosystem() {
    let runtime = ChocoRuntime::new();
    assert_eq!(runtime.ecosystem(), Ecosystem::System);
}

#[rstest]
fn test_choco_runtime_description() {
    let runtime = ChocoRuntime::new();
    assert!(runtime.description().contains("Chocolatey"));
    assert!(runtime.description().contains("Windows"));
}

#[rstest]
fn test_choco_runtime_aliases() {
    let runtime = ChocoRuntime::new();
    let aliases = runtime.aliases();
    assert!(aliases.contains(&"chocolatey"));
}

#[rstest]
fn test_choco_runtime_metadata() {
    let runtime = ChocoRuntime::new();
    let metadata = runtime.metadata();
    assert!(metadata.contains_key("homepage"));
    assert!(metadata.get("homepage").unwrap().contains("chocolatey.org"));
}

#[rstest]
fn test_choco_provider_name() {
    let provider = ChocoProvider::new();
    assert_eq!(provider.name(), "choco");
}

#[rstest]
fn test_choco_provider_description() {
    let provider = ChocoProvider::new();
    assert!(provider.description().contains("Chocolatey"));
}

#[rstest]
fn test_choco_provider_runtimes() {
    let provider = ChocoProvider::new();
    let runtimes = provider.runtimes();
    assert_eq!(runtimes.len(), 1);

    let names: Vec<&str> = runtimes.iter().map(|r| r.name()).collect();
    assert!(names.contains(&"choco"));
}

#[rstest]
fn test_choco_provider_supports() {
    let provider = ChocoProvider::new();
    assert!(provider.supports("choco"));
    assert!(provider.supports("chocolatey")); // alias
    assert!(!provider.supports("brew"));
    assert!(!provider.supports("npm"));
}

#[rstest]
fn test_choco_provider_get_runtime() {
    let provider = ChocoProvider::new();

    let choco = provider.get_runtime("choco");
    assert!(choco.is_some());
    assert_eq!(choco.unwrap().name(), "choco");

    // Test alias
    let chocolatey = provider.get_runtime("chocolatey");
    assert!(chocolatey.is_some());
    assert_eq!(chocolatey.unwrap().name(), "choco");

    let unknown = provider.get_runtime("unknown");
    assert!(unknown.is_none());
}

#[rstest]
fn test_choco_executable_path() {
    use vx_runtime::{Arch, Os, Platform};

    let runtime = ChocoRuntime::new();
    let platform = Platform {
        os: Os::Windows,
        arch: Arch::X86_64,
    };

    let path = runtime.executable_relative_path("2.4.3", &platform);
    assert!(path.contains("choco.exe"));
    assert!(path.contains("tools"));
}
