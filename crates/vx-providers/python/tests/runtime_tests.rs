//! Python runtime tests

use rstest::rstest;
use vx_provider_python::{PythonProvider, PythonRuntime};
use vx_runtime::{Arch, Ecosystem, Os, Platform, Provider, Runtime};

#[test]
fn test_provider_name() {
    let provider = PythonProvider::new();
    assert_eq!(provider.name(), "python");
}

#[test]
fn test_provider_description() {
    let provider = PythonProvider::new();
    assert!(provider
        .description()
        .contains("Python programming language"));
}

#[test]
fn test_provider_runtimes() {
    let provider = PythonProvider::new();
    let runtimes = provider.runtimes();
    // Provider includes python and pip (bundled with python)
    assert_eq!(runtimes.len(), 2);
    let names: Vec<&str> = runtimes.iter().map(|r| r.name()).collect();
    assert!(names.contains(&"python"));
    assert!(names.contains(&"pip"));
}

#[test]
fn test_runtime_name() {
    let runtime = PythonRuntime::new();
    assert_eq!(runtime.name(), "python");
}

#[test]
fn test_runtime_description() {
    let runtime = PythonRuntime::new();
    let desc = runtime.description();
    assert!(desc.contains("Python"));
    // Description should mention 3.7 - 3.13 range
    assert!(desc.contains("3.7") || desc.contains("3.13"));
}

#[test]
fn test_runtime_aliases() {
    let runtime = PythonRuntime::new();
    let aliases = runtime.aliases();
    assert!(aliases.contains(&"python3"));
    assert!(aliases.contains(&"py"));
}

#[test]
fn test_runtime_ecosystem() {
    let runtime = PythonRuntime::new();
    assert_eq!(runtime.ecosystem(), Ecosystem::Python);
}

#[test]
fn test_runtime_metadata() {
    let runtime = PythonRuntime::new();
    let meta = runtime.metadata();
    assert!(meta.contains_key("homepage"));
    assert!(meta.contains_key("repository"));
    assert!(meta.contains_key("license"));
    assert!(meta.contains_key("supported_versions"));
    assert!(meta
        .get("source")
        .unwrap()
        .contains("python-build-standalone"));
    // Check supported versions includes 3.7 to 3.13
    let supported = meta.get("supported_versions").unwrap();
    assert!(supported.contains("3.7"));
    assert!(supported.contains("3.9"));
    assert!(supported.contains("3.12"));
    assert!(supported.contains("3.13"));
}

#[rstest]
#[case(Os::Windows, Arch::X86_64, "python/python.exe")]
#[case(Os::Linux, Arch::X86_64, "python/bin/python3")]
#[case(Os::MacOS, Arch::Aarch64, "python/bin/python3")]
fn test_executable_relative_path(#[case] os: Os, #[case] arch: Arch, #[case] expected_path: &str) {
    let runtime = PythonRuntime::new();
    let platform = Platform { os, arch };
    let path = runtime.executable_relative_path("3.12.8", &platform);
    assert_eq!(path, expected_path);
}
