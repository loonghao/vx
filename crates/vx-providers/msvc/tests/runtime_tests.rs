#![cfg(target_os = "windows")]

//! Tests for MSVC Build Tools runtime

use rstest::rstest;
use std::path::PathBuf;
use tempfile::TempDir;

use vx_provider_msvc::{
    MsvcInstallConfig, MsvcInstallInfo, MsvcProvider, MsvcRuntime, PlatformHelper,
};
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

// ============================================
// MsvcInstallInfo Tests
// ============================================

fn create_test_install_info(install_path: PathBuf) -> MsvcInstallInfo {
    MsvcInstallInfo {
        install_path: install_path.clone(),
        msvc_version: "14.40.33807".to_string(),
        sdk_version: Some("10.0.22621.0".to_string()),
        cl_exe_path: install_path.join("bin/Hostx64/x64/cl.exe"),
        link_exe_path: Some(install_path.join("bin/Hostx64/x64/link.exe")),
        lib_exe_path: Some(install_path.join("bin/Hostx64/x64/lib.exe")),
        nmake_exe_path: Some(install_path.join("bin/Hostx64/x64/nmake.exe")),
        include_paths: vec![
            install_path.join("include"),
            install_path.join("sdk/include/ucrt"),
        ],
        lib_paths: vec![
            install_path.join("lib/x64"),
            install_path.join("sdk/lib/x64"),
        ],
        bin_paths: vec![install_path.join("bin/Hostx64/x64")],
    }
}

#[test]
fn test_install_info_get_environment() {
    let temp_dir = TempDir::new().unwrap();
    let install_path = temp_dir.path().to_path_buf();
    let info = create_test_install_info(install_path.clone());

    let env = info.get_environment();

    // Check INCLUDE
    assert!(env.contains_key("INCLUDE"));
    let include = env.get("INCLUDE").unwrap();
    assert!(include.contains("include"));
    assert!(include.contains("ucrt"));

    // Check LIB
    assert!(env.contains_key("LIB"));
    let lib = env.get("LIB").unwrap();
    assert!(lib.contains("lib"));
    assert!(lib.contains("x64"));

    // Check PATH
    assert!(env.contains_key("PATH"));
    let path = env.get("PATH").unwrap();
    assert!(path.contains("bin"));
    assert!(path.contains("Hostx64"));
}

#[test]
fn test_install_info_save_and_load() {
    let temp_dir = TempDir::new().unwrap();
    let install_path = temp_dir.path().to_path_buf();
    let info = create_test_install_info(install_path.clone());

    // Save
    info.save().expect("Failed to save install info");

    // Verify file exists
    let info_file = install_path.join("msvc-info.json");
    assert!(info_file.exists(), "msvc-info.json should exist");

    // Load
    let loaded = MsvcInstallInfo::load(&install_path)
        .expect("Failed to load install info")
        .expect("Install info should exist");

    // Verify loaded data matches
    assert_eq!(loaded.msvc_version, info.msvc_version);
    assert_eq!(loaded.sdk_version, info.sdk_version);
    assert_eq!(loaded.include_paths.len(), info.include_paths.len());
    assert_eq!(loaded.lib_paths.len(), info.lib_paths.len());
    assert_eq!(loaded.bin_paths.len(), info.bin_paths.len());
}

#[test]
fn test_install_info_load_nonexistent() {
    let temp_dir = TempDir::new().unwrap();
    let install_path = temp_dir.path().to_path_buf();

    // Load from empty directory
    let result = MsvcInstallInfo::load(&install_path).expect("Should not error");
    assert!(result.is_none(), "Should return None for nonexistent file");
}

#[test]
fn test_install_info_get_tool_path() {
    let temp_dir = TempDir::new().unwrap();
    let install_path = temp_dir.path().to_path_buf();
    let info = create_test_install_info(install_path.clone());

    // Test known tools
    assert!(info.get_tool_path("cl").is_some());
    assert!(info.get_tool_path("cl.exe").is_some());
    assert!(info.get_tool_path("link").is_some());
    assert!(info.get_tool_path("lib").is_some());
    assert!(info.get_tool_path("nmake").is_some());

    // Test unknown tool
    assert!(info.get_tool_path("unknown").is_none());
}

#[test]
fn test_install_info_environment_empty_paths() {
    let temp_dir = TempDir::new().unwrap();
    let install_path = temp_dir.path().to_path_buf();

    let info = MsvcInstallInfo {
        install_path,
        msvc_version: "14.40".to_string(),
        sdk_version: None,
        cl_exe_path: PathBuf::from("cl.exe"),
        link_exe_path: None,
        lib_exe_path: None,
        nmake_exe_path: None,
        include_paths: vec![],
        lib_paths: vec![],
        bin_paths: vec![],
    };

    let env = info.get_environment();

    // Empty paths should not add environment variables
    assert!(!env.contains_key("INCLUDE"));
    assert!(!env.contains_key("LIB"));
    assert!(!env.contains_key("PATH"));
}
