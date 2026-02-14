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
    let platform = Platform::new(os, arch);
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
    let platform = Platform::new(os, arch);
    assert_eq!(PlatformHelper::is_platform_supported(&platform), expected);
}

// ============================================
// Executable Path Tests
// ============================================

#[test]
fn test_executable_relative_path_x64() {
    let runtime = MsvcRuntime::new();
    let platform = Platform::new(Os::Windows, Arch::X86_64);
    let path = runtime.executable_relative_path("14.40.33807", &platform);
    assert_eq!(path, "VC/Tools/MSVC/14.40.33807/bin/Hostx64/x64/cl.exe");
}

#[test]
fn test_executable_relative_path_x86() {
    let runtime = MsvcRuntime::new();
    let platform = Platform::new(Os::Windows, Arch::X86);
    let path = runtime.executable_relative_path("14.40.33807", &platform);
    assert_eq!(path, "VC/Tools/MSVC/14.40.33807/bin/Hostx86/x86/cl.exe");
}

#[test]
fn test_executable_relative_path_arm64() {
    let runtime = MsvcRuntime::new();
    let platform = Platform::new(Os::Windows, Arch::Aarch64);
    let path = runtime.executable_relative_path("14.40.33807", &platform);
    assert_eq!(path, "VC/Tools/MSVC/14.40.33807/bin/Hostarm64/arm64/cl.exe");
}

// ============================================
// MsvcInstallInfo Tests
// ============================================

fn create_test_install_info(install_path: PathBuf) -> MsvcInstallInfo {
    // Create actual directories on disk so path validation passes
    let include_dir = install_path.join("include");
    let ucrt_dir = install_path.join("sdk").join("include").join("ucrt");
    let lib_x64_dir = install_path.join("lib").join("x64");
    let sdk_lib_dir = install_path.join("sdk").join("lib").join("x64");
    let bin_dir = install_path.join("bin").join("Hostx64").join("x64");
    std::fs::create_dir_all(&include_dir).unwrap();
    std::fs::create_dir_all(&ucrt_dir).unwrap();
    std::fs::create_dir_all(&lib_x64_dir).unwrap();
    std::fs::create_dir_all(&sdk_lib_dir).unwrap();
    std::fs::create_dir_all(&bin_dir).unwrap();

    // Create fake executables
    std::fs::write(bin_dir.join("cl.exe"), "fake").unwrap();
    std::fs::write(bin_dir.join("link.exe"), "fake").unwrap();
    std::fs::write(bin_dir.join("lib.exe"), "fake").unwrap();
    std::fs::write(bin_dir.join("nmake.exe"), "fake").unwrap();

    MsvcInstallInfo {
        install_path: install_path.clone(),
        msvc_version: "14.40.33807".to_string(),
        sdk_version: Some("10.0.22621.0".to_string()),
        cl_exe_path: bin_dir.join("cl.exe"),
        link_exe_path: Some(bin_dir.join("link.exe")),
        lib_exe_path: Some(bin_dir.join("lib.exe")),
        nmake_exe_path: Some(bin_dir.join("nmake.exe")),
        include_paths: vec![include_dir, ucrt_dir],
        lib_paths: vec![lib_x64_dir, sdk_lib_dir],
        bin_paths: vec![bin_dir],
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

// ============================================
// Path Validation Tests (Issue #573)
// ============================================

#[test]
fn test_validate_paths_valid_installation() {
    let temp_dir = TempDir::new().unwrap();
    let install_path = temp_dir.path().to_path_buf();

    // Create real paths on disk
    let include_dir = install_path.join("include");
    let lib_dir = install_path.join("lib").join("x64");
    let bin_dir = install_path.join("bin").join("Hostx64").join("x64");
    std::fs::create_dir_all(&include_dir).unwrap();
    std::fs::create_dir_all(&lib_dir).unwrap();
    std::fs::create_dir_all(&bin_dir).unwrap();

    // Create cl.exe
    let cl_path = bin_dir.join("cl.exe");
    std::fs::write(&cl_path, "fake").unwrap();

    let info = MsvcInstallInfo {
        install_path,
        msvc_version: "14.40.33807".to_string(),
        sdk_version: None,
        cl_exe_path: cl_path,
        link_exe_path: None,
        lib_exe_path: None,
        nmake_exe_path: None,
        include_paths: vec![include_dir],
        lib_paths: vec![lib_dir],
        bin_paths: vec![bin_dir],
    };

    assert!(info.validate_paths());
}

#[test]
fn test_validate_paths_stale_cache() {
    let temp_dir = TempDir::new().unwrap();
    let install_path = temp_dir.path().to_path_buf();

    // Don't create any paths — simulates stale cache with non-existent paths
    let info = MsvcInstallInfo {
        install_path: install_path.clone(),
        msvc_version: "14.44.35207".to_string(),
        sdk_version: None,
        cl_exe_path: install_path.join("bin/Hostx64/x64/cl.exe"),
        link_exe_path: None,
        lib_exe_path: None,
        nmake_exe_path: None,
        include_paths: vec![install_path.join("include")],
        lib_paths: vec![install_path.join("lib/x64")],
        bin_paths: vec![install_path.join("bin/Hostx64/x64")],
    };

    // cl.exe doesn't exist, so validation should fail
    assert!(!info.validate_paths());
}

#[test]
fn test_get_environment_filters_nonexistent_paths() {
    let temp_dir = TempDir::new().unwrap();
    let install_path = temp_dir.path().to_path_buf();

    // Create only some paths — simulates partial stale cache
    let include_real = install_path.join("include_real");
    let lib_real = install_path.join("lib_real");
    let bin_real = install_path.join("bin_real");
    std::fs::create_dir_all(&include_real).unwrap();
    std::fs::create_dir_all(&lib_real).unwrap();
    std::fs::create_dir_all(&bin_real).unwrap();

    let info = MsvcInstallInfo {
        install_path: install_path.clone(),
        msvc_version: "14.40.33807".to_string(),
        sdk_version: None,
        cl_exe_path: install_path.join("cl.exe"),
        link_exe_path: None,
        lib_exe_path: None,
        nmake_exe_path: None,
        include_paths: vec![
            include_real.clone(),
            install_path.join("include_nonexistent"), // doesn't exist
        ],
        lib_paths: vec![
            lib_real.clone(),
            install_path.join("lib_nonexistent"), // doesn't exist
        ],
        bin_paths: vec![
            bin_real.clone(),
            install_path.join("bin_nonexistent"), // doesn't exist
        ],
    };

    let env = info.get_environment();

    // INCLUDE should only contain the real path
    let include = env.get("INCLUDE").unwrap();
    assert!(include.contains("include_real"));
    assert!(!include.contains("include_nonexistent"));

    // LIB should only contain the real path
    let lib = env.get("LIB").unwrap();
    assert!(lib.contains("lib_real"));
    assert!(!lib.contains("lib_nonexistent"));

    // PATH should only contain the real bin path
    let path = env.get("PATH").unwrap();
    assert!(path.contains("bin_real"));
    assert!(!path.contains("bin_nonexistent"));
}

#[test]
fn test_get_environment_sets_vcinstalldir() {
    let temp_dir = TempDir::new().unwrap();
    let install_path = temp_dir.path().to_path_buf();

    // Create VC directory structure (simulates vx-managed MSVC layout)
    let vc_dir = install_path.join("VC");
    let tools_dir = vc_dir.join("Tools").join("MSVC").join("14.40.33807");
    std::fs::create_dir_all(&tools_dir).unwrap();

    let info = MsvcInstallInfo {
        install_path,
        msvc_version: "14.40.33807".to_string(),
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

    // VCINSTALLDIR should be set with trailing backslash
    let vcinstalldir = env.get("VCINSTALLDIR").unwrap();
    assert!(vcinstalldir.contains("VC"));
    assert!(vcinstalldir.ends_with('\\'));

    // VCToolsInstallDir should point to the exact version directory
    let vctoolsdir = env.get("VCToolsInstallDir").unwrap();
    assert!(vctoolsdir.contains("14.40.33807"));
    assert!(vctoolsdir.ends_with('\\'));
}

#[test]
fn test_get_environment_no_vcinstalldir_without_vc_dir() {
    let temp_dir = TempDir::new().unwrap();
    let install_path = temp_dir.path().to_path_buf();

    // Don't create VC directory
    let info = MsvcInstallInfo {
        install_path,
        msvc_version: "14.40.33807".to_string(),
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

    // Should not set VCINSTALLDIR when VC dir doesn't exist
    assert!(!env.contains_key("VCINSTALLDIR"));
    assert!(!env.contains_key("VCToolsInstallDir"));
}

#[test]
fn test_validated_paths_methods() {
    let temp_dir = TempDir::new().unwrap();
    let install_path = temp_dir.path().to_path_buf();

    // Create only some paths
    let real_include = install_path.join("real_include");
    let real_lib = install_path.join("real_lib");
    let real_bin = install_path.join("real_bin");
    std::fs::create_dir_all(&real_include).unwrap();
    std::fs::create_dir_all(&real_lib).unwrap();
    std::fs::create_dir_all(&real_bin).unwrap();

    let info = MsvcInstallInfo {
        install_path: install_path.clone(),
        msvc_version: "14.40".to_string(),
        sdk_version: None,
        cl_exe_path: PathBuf::from("cl.exe"),
        link_exe_path: None,
        lib_exe_path: None,
        nmake_exe_path: None,
        include_paths: vec![real_include.clone(), install_path.join("fake_include")],
        lib_paths: vec![real_lib.clone(), install_path.join("fake_lib")],
        bin_paths: vec![real_bin.clone(), install_path.join("fake_bin")],
    };

    // Only real paths should be returned
    assert_eq!(info.validated_include_paths().len(), 1);
    assert_eq!(info.validated_lib_paths().len(), 1);
    assert_eq!(info.validated_bin_paths().len(), 1);
}
