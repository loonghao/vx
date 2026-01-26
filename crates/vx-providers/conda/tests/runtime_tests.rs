//! Unit tests for the Conda provider
//!
//! Tests are placed in a separate tests/ directory following project conventions.

use rstest::rstest;
use vx_provider_conda::{create_provider, CondaProvider, CondaRuntime, CondaUrlBuilder, MambaRuntime, MicromambaRuntime};
use vx_runtime::{Arch, Ecosystem, Os, Platform, Provider, Runtime};

// ============================================================================
// Provider Tests
// ============================================================================

#[test]
fn test_provider_name() {
    let provider = CondaProvider::new();
    assert_eq!(provider.name(), "conda");
}

#[test]
fn test_provider_description() {
    let provider = CondaProvider::new();
    assert!(!provider.description().is_empty());
    assert!(provider.description().contains("Conda") || provider.description().contains("conda"));
}

#[test]
fn test_provider_supports() {
    let provider = CondaProvider::new();
    assert!(provider.supports("micromamba"));
    assert!(provider.supports("conda"));
    assert!(provider.supports("mamba"));
    assert!(provider.supports("umamba")); // alias for micromamba
    assert!(provider.supports("miniforge")); // alias for conda
    assert!(!provider.supports("python"));
    assert!(!provider.supports("pip"));
    assert!(!provider.supports("spack"));
}

#[test]
fn test_provider_runtimes() {
    let provider = CondaProvider::new();
    let runtimes = provider.runtimes();
    assert_eq!(runtimes.len(), 3);
    // Micromamba should be first (recommended)
    assert_eq!(runtimes[0].name(), "micromamba");
    assert_eq!(runtimes[1].name(), "conda");
    assert_eq!(runtimes[2].name(), "mamba");
}

#[test]
fn test_provider_get_runtime() {
    let provider = CondaProvider::new();
    assert!(provider.get_runtime("micromamba").is_some());
    assert!(provider.get_runtime("conda").is_some());
    assert!(provider.get_runtime("mamba").is_some());
    assert!(provider.get_runtime("umamba").is_some()); // alias
    assert!(provider.get_runtime("miniforge").is_some()); // alias
    assert!(provider.get_runtime("python").is_none());
}

#[test]
fn test_create_provider() {
    let provider = create_provider();
    assert_eq!(provider.name(), "conda");
}

// ============================================================================
// Micromamba Runtime Tests
// ============================================================================

#[test]
fn test_micromamba_runtime_name() {
    let runtime = MicromambaRuntime::new();
    assert_eq!(runtime.name(), "micromamba");
}

#[test]
fn test_micromamba_runtime_description() {
    let runtime = MicromambaRuntime::new();
    let desc = runtime.description();
    assert!(!desc.is_empty());
    assert!(desc.contains("mamba") || desc.contains("conda") || desc.contains("package"));
}

#[test]
fn test_micromamba_runtime_aliases() {
    let runtime = MicromambaRuntime::new();
    let aliases = runtime.aliases();
    assert!(!aliases.is_empty());
    assert!(aliases.contains(&"umamba"));
}

#[test]
fn test_micromamba_runtime_ecosystem() {
    let runtime = MicromambaRuntime::new();
    assert_eq!(runtime.ecosystem(), Ecosystem::Python);
}

#[test]
fn test_micromamba_runtime_metadata() {
    let runtime = MicromambaRuntime::new();
    let meta = runtime.metadata();
    assert!(!meta.is_empty());
    assert!(meta.contains_key("homepage"));
    assert!(meta.contains_key("repository"));
    assert!(meta.contains_key("license"));
    assert!(meta.get("repository").unwrap().contains("mamba-org"));
}

// ============================================================================
// Conda Runtime Tests
// ============================================================================

#[test]
fn test_conda_runtime_name() {
    let runtime = CondaRuntime::new();
    assert_eq!(runtime.name(), "conda");
}

#[test]
fn test_conda_runtime_description() {
    let runtime = CondaRuntime::new();
    let desc = runtime.description();
    assert!(!desc.is_empty());
    assert!(desc.contains("Package") || desc.contains("environment") || desc.contains("Miniforge"));
}

#[test]
fn test_conda_runtime_aliases() {
    let runtime = CondaRuntime::new();
    let aliases = runtime.aliases();
    assert!(!aliases.is_empty());
    assert!(aliases.contains(&"miniforge"));
    assert!(aliases.contains(&"miniconda"));
}

#[test]
fn test_conda_runtime_ecosystem() {
    let runtime = CondaRuntime::new();
    assert_eq!(runtime.ecosystem(), Ecosystem::Python);
}

#[test]
fn test_conda_runtime_metadata() {
    let runtime = CondaRuntime::new();
    let meta = runtime.metadata();
    assert!(!meta.is_empty());
    assert!(meta.contains_key("homepage"));
    assert!(meta.contains_key("repository"));
    assert!(meta.get("homepage").unwrap().contains("conda"));
    assert!(meta.get("repository").unwrap().contains("miniforge"));
}

// ============================================================================
// Mamba Runtime Tests
// ============================================================================

#[test]
fn test_mamba_runtime_name() {
    let runtime = MambaRuntime::new();
    assert_eq!(runtime.name(), "mamba");
}

#[test]
fn test_mamba_runtime_description() {
    let runtime = MambaRuntime::new();
    let desc = runtime.description();
    assert!(!desc.is_empty());
    assert!(desc.contains("package") || desc.contains("Fast") || desc.contains("Miniforge"));
}

#[test]
fn test_mamba_runtime_ecosystem() {
    let runtime = MambaRuntime::new();
    assert_eq!(runtime.ecosystem(), Ecosystem::Python);
}

#[test]
fn test_mamba_runtime_metadata() {
    let runtime = MambaRuntime::new();
    let meta = runtime.metadata();
    assert!(!meta.is_empty());
    assert!(meta.contains_key("homepage"));
    assert!(meta.get("homepage").unwrap().contains("mamba"));
    assert!(meta.contains_key("bundled_with"));
    assert_eq!(meta.get("bundled_with").unwrap(), "conda");
}

// ============================================================================
// Executable Path Tests
// ============================================================================

#[rstest]
#[case(Os::Windows, Arch::X86_64, "Library/bin/micromamba.exe")]
#[case(Os::Windows, Arch::Aarch64, "Library/bin/micromamba.exe")]
#[case(Os::MacOS, Arch::X86_64, "bin/micromamba")]
#[case(Os::MacOS, Arch::Aarch64, "bin/micromamba")]
#[case(Os::Linux, Arch::X86_64, "bin/micromamba")]
#[case(Os::Linux, Arch::Aarch64, "bin/micromamba")]
fn test_micromamba_executable_relative_path(#[case] os: Os, #[case] arch: Arch, #[case] expected: &str) {
    let runtime = MicromambaRuntime::new();
    let platform = Platform::new(os, arch);
    let path = runtime.executable_relative_path("2.5.0-1", &platform);
    assert_eq!(path, expected);
}

#[rstest]
#[case(Os::Windows, Arch::X86_64, "Scripts\\conda.exe")]
#[case(Os::MacOS, Arch::X86_64, "bin/conda")]
#[case(Os::MacOS, Arch::Aarch64, "bin/conda")]
#[case(Os::Linux, Arch::X86_64, "bin/conda")]
#[case(Os::Linux, Arch::Aarch64, "bin/conda")]
fn test_conda_executable_relative_path(#[case] os: Os, #[case] arch: Arch, #[case] expected: &str) {
    let runtime = CondaRuntime::new();
    let platform = Platform::new(os, arch);
    let path = runtime.executable_relative_path("24.3.0-0", &platform);
    assert_eq!(path, expected);
}

#[rstest]
#[case(Os::Windows, Arch::X86_64, "Scripts\\mamba.exe")]
#[case(Os::MacOS, Arch::X86_64, "bin/mamba")]
#[case(Os::Linux, Arch::X86_64, "bin/mamba")]
fn test_mamba_executable_relative_path(#[case] os: Os, #[case] arch: Arch, #[case] expected: &str) {
    let runtime = MambaRuntime::new();
    let platform = Platform::new(os, arch);
    let path = runtime.executable_relative_path("24.3.0-0", &platform);
    assert_eq!(path, expected);
}

// ============================================================================
// URL Builder Tests
// ============================================================================

#[rstest]
#[case(Os::Linux, Arch::X86_64, "linux-64")]
#[case(Os::Linux, Arch::Aarch64, "linux-aarch64")]
#[case(Os::MacOS, Arch::X86_64, "osx-64")]
#[case(Os::MacOS, Arch::Aarch64, "osx-arm64")]
#[case(Os::Windows, Arch::X86_64, "win-64")]
fn test_micromamba_download_url_platform(#[case] os: Os, #[case] arch: Arch, #[case] platform_str: &str) {
    let platform = Platform::new(os, arch);
    let url = CondaUrlBuilder::micromamba_download_url("2.5.0-1", &platform);
    assert!(url.is_some());
    let url = url.unwrap();
    assert!(url.contains("mamba-org/micromamba-releases"));
    assert!(url.contains("2.5.0-1"));
    assert!(url.contains(platform_str));
    assert!(url.ends_with(".tar.bz2"));
}

#[rstest]
#[case(Os::Linux, Arch::X86_64, "Linux-x86_64.sh")]
#[case(Os::Linux, Arch::Aarch64, "Linux-aarch64.sh")]
#[case(Os::MacOS, Arch::X86_64, "MacOSX-x86_64.sh")]
#[case(Os::MacOS, Arch::Aarch64, "MacOSX-arm64.sh")]
#[case(Os::Windows, Arch::X86_64, "Windows-x86_64.exe")]
fn test_conda_download_url_platform(#[case] os: Os, #[case] arch: Arch, #[case] expected_suffix: &str) {
    let platform = Platform::new(os, arch);
    let url = CondaUrlBuilder::conda_download_url("24.3.0-0", &platform);
    assert!(url.is_some());
    let url = url.unwrap();
    assert!(url.contains("conda-forge/miniforge"));
    assert!(url.contains("24.3.0-0"));
    assert!(url.contains(expected_suffix));
}

#[test]
fn test_micromamba_download_url_unsupported_platform() {
    let platform = Platform::new(Os::FreeBSD, Arch::X86_64);
    let url = CondaUrlBuilder::micromamba_download_url("2.5.0-1", &platform);
    assert!(url.is_none());
}

#[test]
fn test_conda_download_url_unsupported_platform() {
    let platform = Platform::new(Os::FreeBSD, Arch::X86_64);
    let url = CondaUrlBuilder::conda_download_url("24.3.0-0", &platform);
    assert!(url.is_none());
}

// ============================================================================
// Platform Support Tests
// ============================================================================

#[rstest]
#[case(Os::Windows, Arch::X86_64)]
#[case(Os::MacOS, Arch::X86_64)]
#[case(Os::MacOS, Arch::Aarch64)]
#[case(Os::Linux, Arch::X86_64)]
#[case(Os::Linux, Arch::Aarch64)]
fn test_micromamba_platform_support(#[case] os: Os, #[case] arch: Arch) {
    let runtime = MicromambaRuntime::new();
    let platform = Platform::new(os, arch);
    let path = runtime.executable_relative_path("2.5.0-1", &platform);
    assert!(!path.is_empty());
}

#[rstest]
#[case(Os::Windows, Arch::X86_64)]
#[case(Os::MacOS, Arch::X86_64)]
#[case(Os::MacOS, Arch::Aarch64)]
#[case(Os::Linux, Arch::X86_64)]
#[case(Os::Linux, Arch::Aarch64)]
fn test_conda_platform_support(#[case] os: Os, #[case] arch: Arch) {
    let runtime = CondaRuntime::new();
    let platform = Platform::new(os, arch);
    let path = runtime.executable_relative_path("24.3.0-0", &platform);
    assert!(!path.is_empty());
}
