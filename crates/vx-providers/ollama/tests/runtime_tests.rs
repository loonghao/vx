//! Tests for Ollama runtime

use rstest::rstest;
use vx_provider_ollama::{OllamaProvider, OllamaRuntime, OllamaUrlBuilder};
use vx_runtime::{Arch, Ecosystem, Os, Platform, Provider, Runtime};

#[test]
fn test_runtime_name() {
    let runtime = OllamaRuntime::new();
    assert_eq!(runtime.name(), "ollama");
}

#[test]
fn test_runtime_description() {
    let runtime = OllamaRuntime::new();
    assert!(runtime.description().contains("Ollama"));
    assert!(runtime.description().contains("LLM") || runtime.description().contains("language"));
}

#[test]
fn test_runtime_ecosystem() {
    let runtime = OllamaRuntime::new();
    assert_eq!(runtime.ecosystem(), Ecosystem::System);
}

#[test]
fn test_runtime_metadata() {
    let runtime = OllamaRuntime::new();
    let meta = runtime.metadata();
    assert!(meta.contains_key("homepage"));
    assert!(meta.contains_key("repository"));
    assert!(meta.contains_key("category"));
    assert_eq!(meta.get("category"), Some(&"ai".to_string()));
}

#[test]
fn test_provider_name() {
    let provider = OllamaProvider::new();
    assert_eq!(provider.name(), "ollama");
}

#[test]
fn test_provider_description() {
    let provider = OllamaProvider::new();
    assert!(provider.description().contains("Ollama"));
}

#[test]
fn test_provider_supports() {
    let provider = OllamaProvider::new();
    assert!(provider.supports("ollama"));
    assert!(!provider.supports("other"));
    assert!(!provider.supports("llama"));
}

#[test]
fn test_provider_runtimes() {
    let provider = OllamaProvider::new();
    let runtimes = provider.runtimes();
    assert_eq!(runtimes.len(), 1);
    assert_eq!(runtimes[0].name(), "ollama");
}

#[test]
fn test_provider_get_runtime() {
    let provider = OllamaProvider::new();
    assert!(provider.get_runtime("ollama").is_some());
    assert!(provider.get_runtime("other").is_none());
}

#[rstest]
#[case(Os::Linux, Arch::X86_64, Some("linux-amd64"))]
#[case(Os::Linux, Arch::Aarch64, Some("linux-arm64"))]
#[case(Os::MacOS, Arch::X86_64, Some("darwin"))]
#[case(Os::MacOS, Arch::Aarch64, Some("darwin"))]
#[case(Os::Windows, Arch::X86_64, Some("windows-amd64"))]
#[case(Os::Windows, Arch::Aarch64, Some("windows-arm64"))]
fn test_target_string(#[case] os: Os, #[case] arch: Arch, #[case] expected: Option<&str>) {
    let platform = Platform { os, arch };
    let target = OllamaUrlBuilder::get_target_string(&platform);
    assert_eq!(target, expected);
}

#[rstest]
#[case(Os::Windows, "zip")]
#[case(Os::Linux, "tgz")]
#[case(Os::MacOS, "tgz")]
fn test_archive_extension(#[case] os: Os, #[case] expected: &str) {
    let platform = Platform {
        os,
        arch: Arch::X86_64,
    };
    let ext = OllamaUrlBuilder::get_archive_extension(&platform);
    assert_eq!(ext, expected);
}

#[rstest]
#[case(Os::Windows, "ollama.exe")]
#[case(Os::Linux, "ollama")]
#[case(Os::MacOS, "ollama")]
fn test_executable_name(#[case] os: Os, #[case] expected: &str) {
    let platform = Platform {
        os,
        arch: Arch::X86_64,
    };
    let name = OllamaUrlBuilder::get_executable_name(&platform);
    assert_eq!(name, expected);
}

#[test]
fn test_download_url_format_linux() {
    let platform = Platform {
        os: Os::Linux,
        arch: Arch::X86_64,
    };
    let url = OllamaUrlBuilder::download_url("0.13.5", &platform).unwrap();
    assert!(url.contains("github.com/ollama/ollama"));
    assert!(url.contains("v0.13.5"));
    assert!(url.contains("linux-amd64"));
    assert!(url.ends_with(".tgz"));
}

#[test]
fn test_download_url_format_windows() {
    let platform = Platform {
        os: Os::Windows,
        arch: Arch::X86_64,
    };
    let url = OllamaUrlBuilder::download_url("0.13.5", &platform).unwrap();
    assert!(url.contains("github.com/ollama/ollama"));
    assert!(url.contains("v0.13.5"));
    assert!(url.contains("windows-amd64"));
    assert!(url.ends_with(".zip"));
}

#[test]
fn test_download_url_format_macos() {
    let platform = Platform {
        os: Os::MacOS,
        arch: Arch::Aarch64,
    };
    let url = OllamaUrlBuilder::download_url("0.13.5", &platform).unwrap();
    assert!(url.contains("github.com/ollama/ollama"));
    assert!(url.contains("v0.13.5"));
    assert!(url.contains("darwin"));
    assert!(url.ends_with(".tgz"));
}

#[test]
fn test_download_url_with_v_prefix() {
    let platform = Platform {
        os: Os::Linux,
        arch: Arch::X86_64,
    };
    // Should handle version with 'v' prefix without doubling it
    let url = OllamaUrlBuilder::download_url("v0.13.5", &platform).unwrap();
    assert!(url.contains("/v0.13.5/"));
    assert!(!url.contains("/vv0.13.5/"));
}

#[test]
fn test_executable_relative_path_linux() {
    let runtime = OllamaRuntime::new();
    let platform = Platform {
        os: Os::Linux,
        arch: Arch::X86_64,
    };
    assert_eq!(
        runtime.executable_relative_path("0.13.5", &platform),
        "bin/ollama"
    );
}

#[test]
fn test_executable_relative_path_macos() {
    let runtime = OllamaRuntime::new();
    let platform = Platform {
        os: Os::MacOS,
        arch: Arch::Aarch64,
    };
    assert_eq!(
        runtime.executable_relative_path("0.13.5", &platform),
        "bin/ollama"
    );
}

#[test]
fn test_executable_relative_path_windows() {
    let runtime = OllamaRuntime::new();
    let platform = Platform {
        os: Os::Windows,
        arch: Arch::X86_64,
    };
    assert_eq!(
        runtime.executable_relative_path("0.13.5", &platform),
        "ollama.exe"
    );
}
