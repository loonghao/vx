//! Tests for MSVC Build Tools runtime

use rstest::rstest;
use vx_provider_msvc::{MsvcInstallConfig, MsvcProvider, MsvcRuntime, PlatformHelper};
use vx_runtime::{Arch, Ecosystem, Os, Platform, Provider, Runtime};

// ============================================
// Runtime Tests
// ============================================

#[test]
fn test_runtime_name() {
    let runtime = MsvcRuntime::new();
    assert_eq!(runtime.name(), "msvc");
}

#[test]
fn test_runtime_description() {
    let runtime = MsvcRuntime::new();
    assert!(runtime.description().contains("MSVC"));
    assert!(runtime.description().contains("Microsoft"));
}

#[test]
fn test_runtime_ecosystem() {
    let runtime = MsvcRuntime::new();
    assert_eq!(runtime.ecosystem(), Ecosystem::System);
}

#[test]
fn test_runtime_aliases() {
    let runtime = MsvcRuntime::new();
    let aliases = runtime.aliases();
    assert!(aliases.contains(&"cl"));
    assert!(aliases.contains(&"nmake"));
}

#[test]
fn test_runtime_metadata() {
    let runtime = MsvcRuntime::new();
    let meta = runtime.metadata();
    assert!(meta.contains_key("homepage"));
    assert!(meta.contains_key("documentation"));
    assert!(meta.contains_key("category"));
    assert!(meta.contains_key("vendor"));
    assert_eq!(meta.get("category"), Some(&"build-tools".to_string()));
    assert_eq!(meta.get("vendor"), Some(&"Microsoft".to_string()));
}

#[test]
fn test_runtime_supported_platforms() {
    let runtime = MsvcRuntime::new();
    let platforms = runtime.supported_platforms();

    // Should only support Windows
    for platform in &platforms {
        assert_eq!(platform.os, Os::Windows);
    }
}

// ============================================
// Provider Tests
// ============================================

#[test]
fn test_provider_name() {
    let provider = MsvcProvider::new();
    assert_eq!(provider.name(), "msvc");
}

#[test]
fn test_provider_description() {
    let provider = MsvcProvider::new();
    assert!(provider.description().contains("MSVC"));
    assert!(provider.description().contains("Microsoft"));
}

#[test]
fn test_provider_supports() {
    let provider = MsvcProvider::new();
    assert!(provider.supports("msvc"));
    assert!(provider.supports("cl"));
    assert!(provider.supports("nmake"));
    assert!(!provider.supports("gcc"));
    assert!(!provider.supports("clang"));
}

#[test]
fn test_provider_runtimes() {
    let provider = MsvcProvider::new();
    let runtimes = provider.runtimes();
    assert_eq!(runtimes.len(), 1);
    assert_eq!(runtimes[0].name(), "msvc");
}

#[test]
fn test_provider_get_runtime() {
    let provider = MsvcProvider::new();
    assert!(provider.get_runtime("msvc").is_some());
    assert!(provider.get_runtime("cl").is_some());
    assert!(provider.get_runtime("nmake").is_some());
    assert!(provider.get_runtime("gcc").is_none());
}

// ============================================
// Install Config Tests
// ============================================

#[test]
fn test_default_install_config() {
    let config = MsvcInstallConfig::default();
    assert_eq!(config.msvc_version, "14.42");
    assert_eq!(config.host_arch, "x64");
    assert_eq!(config.target_arch, "x64");
    assert!(config.sdk_version.is_none());
}

#[test]
fn test_install_config_builder() {
    let config = MsvcInstallConfig::new("14.40")
        .with_sdk_version("10.0.22621.0")
        .with_host_arch("x64")
        .with_target_arch("arm64");

    assert_eq!(config.msvc_version, "14.40");
    assert_eq!(config.sdk_version, Some("10.0.22621.0".to_string()));
    assert_eq!(config.target_arch, "arm64");
}

// ============================================
// Platform Helper Tests
// ============================================

#[rstest]
#[case(Os::Windows, Arch::X86_64, Some("x64"))]
#[case(Os::Windows, Arch::X86, Some("x86"))]
#[case(Os::Windows, Arch::Aarch64, Some("arm64"))]
#[case(Os::Linux, Arch::X86_64, None)]
#[case(Os::MacOS, Arch::Aarch64, None)]
fn test_arch_string(#[case] os: Os, #[case] arch: Arch, #[case] expected: Option<&str>) {
    let platform = Platform { os, arch };
    assert_eq!(PlatformHelper::get_arch_string(&platform), expected);
}

#[rstest]
#[case(Os::Windows, Arch::X86_64, true)]
#[case(Os::Windows, Arch::X86, true)]
#[case(Os::Windows, Arch::Aarch64, true)]
#[case(Os::Linux, Arch::X86_64, false)]
#[case(Os::MacOS, Arch::X86_64, false)]
#[case(Os::MacOS, Arch::Aarch64, false)]
fn test_platform_support(#[case] os: Os, #[case] arch: Arch, #[case] expected: bool) {
    let platform = Platform { os, arch };
    assert_eq!(PlatformHelper::is_platform_supported(&platform), expected);
}

// ============================================
// Executable Path Tests
// ============================================

#[test]
fn test_executable_relative_path_x64() {
    let runtime = MsvcRuntime::new();
    let platform = Platform {
        os: Os::Windows,
        arch: Arch::X86_64,
    };
    let path = runtime.executable_relative_path("14.40.33807", &platform);
    assert_eq!(path, "VC/Tools/MSVC/14.40.33807/bin/Hostx64/x64/cl.exe");
}

#[test]
fn test_executable_relative_path_x86() {
    let runtime = MsvcRuntime::new();
    let platform = Platform {
        os: Os::Windows,
        arch: Arch::X86,
    };
    let path = runtime.executable_relative_path("14.40.33807", &platform);
    assert_eq!(path, "VC/Tools/MSVC/14.40.33807/bin/Hostx86/x86/cl.exe");
}

#[test]
fn test_executable_relative_path_arm64() {
    let runtime = MsvcRuntime::new();
    let platform = Platform {
        os: Os::Windows,
        arch: Arch::Aarch64,
    };
    let path = runtime.executable_relative_path("14.40.33807", &platform);
    assert_eq!(path, "VC/Tools/MSVC/14.40.33807/bin/Hostarm64/arm64/cl.exe");
}
