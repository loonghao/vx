//! Deno runtime tests

use rstest::rstest;
use vx_provider_deno::{DenoProvider, DenoRuntime};
use vx_runtime::{Arch, Ecosystem, Os, Platform, Provider, Runtime};

#[test]
fn test_deno_runtime_creation() {
    let runtime = DenoRuntime::new();
    assert_eq!(runtime.name(), "deno");
    assert!(!runtime.description().is_empty());
    assert_eq!(runtime.ecosystem(), Ecosystem::NodeJs);
}

#[test]
fn test_deno_runtime_description() {
    let runtime = DenoRuntime::new();
    assert!(
        runtime.description().contains("JavaScript")
            || runtime.description().contains("TypeScript")
    );
}

#[test]
fn test_deno_runtime_metadata() {
    let runtime = DenoRuntime::new();
    let metadata = runtime.metadata();

    assert!(metadata.contains_key("homepage"));
    assert!(metadata.contains_key("ecosystem"));
    assert!(metadata.contains_key("repository"));
    assert!(metadata.contains_key("license"));
    assert_eq!(
        metadata.get("homepage"),
        Some(&"https://deno.land/".to_string())
    );
}

#[test]
fn test_deno_runtime_aliases() {
    let runtime = DenoRuntime::new();
    // Deno has no aliases
    assert!(runtime.aliases().is_empty());
}

#[test]
fn test_deno_provider_creation() {
    let provider = DenoProvider::new();
    assert_eq!(provider.name(), "deno");
    assert!(!provider.description().is_empty());
}

#[test]
fn test_deno_provider_runtimes() {
    let provider = DenoProvider::new();
    let runtimes = provider.runtimes();

    assert_eq!(runtimes.len(), 1);
    assert_eq!(runtimes[0].name(), "deno");
}

#[rstest]
#[case("deno", true)]
#[case("node", false)]
#[case("bun", false)]
fn test_deno_provider_supports(#[case] name: &str, #[case] expected: bool) {
    let provider = DenoProvider::new();
    assert_eq!(provider.supports(name), expected);
}

#[test]
fn test_deno_provider_get_runtime() {
    let provider = DenoProvider::new();

    let deno = provider.get_runtime("deno");
    assert!(deno.is_some());
    assert_eq!(deno.unwrap().name(), "deno");

    let unknown = provider.get_runtime("unknown");
    assert!(unknown.is_none());
}

/// Test that executable_relative_path returns correct path for Deno
/// Deno archives extract directly to the binary (no subdirectory)
#[rstest]
#[case(Os::Linux, Arch::X86_64, "deno")]
#[case(Os::Linux, Arch::Aarch64, "deno")]
#[case(Os::MacOS, Arch::X86_64, "deno")]
#[case(Os::MacOS, Arch::Aarch64, "deno")]
#[case(Os::Windows, Arch::X86_64, "deno.exe")]
fn test_deno_executable_relative_path(#[case] os: Os, #[case] arch: Arch, #[case] expected: &str) {
    let runtime = DenoRuntime::new();
    let platform = Platform { os, arch };
    let path = runtime.executable_relative_path("1.40.0", &platform);
    assert_eq!(path, expected);
}

/// Test download URL generation
#[rstest]
#[case(Os::Linux, Arch::X86_64, "deno-x86_64-unknown-linux-gnu.zip")]
#[case(Os::Linux, Arch::Aarch64, "deno-aarch64-unknown-linux-gnu.zip")]
#[case(Os::MacOS, Arch::X86_64, "deno-x86_64-apple-darwin.zip")]
#[case(Os::MacOS, Arch::Aarch64, "deno-aarch64-apple-darwin.zip")]
#[case(Os::Windows, Arch::X86_64, "deno-x86_64-pc-windows-msvc.zip")]
fn test_deno_download_url_contains_platform(
    #[case] os: Os,
    #[case] arch: Arch,
    #[case] expected_filename: &str,
) {
    // Test that the config generates correct filenames
    let platform = Platform { os, arch };
    let filename = vx_provider_deno::config::DenoUrlBuilder::get_filename(&platform);
    assert_eq!(filename, expected_filename);
}

#[tokio::test]
async fn test_deno_download_url_format() {
    let runtime = DenoRuntime::new();
    let platform = Platform {
        os: Os::Linux,
        arch: Arch::X86_64,
    };

    let url = runtime.download_url("1.40.0", &platform).await.unwrap();
    assert!(url.is_some());
    let url = url.unwrap();
    assert!(url.contains("github.com/denoland/deno/releases"));
    assert!(url.contains("v1.40.0"));
    assert!(url.contains("deno-x86_64-unknown-linux-gnu.zip"));
}
