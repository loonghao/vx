#![cfg(target_os = "windows")]

//! Tests for MSVC Build Tools runtime

use rstest::rstest;
use std::path::PathBuf;
use tempfile::TempDir;

use vx_provider_msvc::{
    MsvcInstallConfig, MsvcInstallInfo, MsvcInstaller, MsvcRuntime, PlatformHelper,
    create_provider, star_metadata,
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
    let provider = create_provider();
    assert_eq!(provider.name(), "msvc");
}

#[test]
fn test_provider_description() {
    let provider = create_provider();
    assert!(!provider.description().is_empty());
}

#[test]
fn test_provider_runtimes() {
    let provider = create_provider();
    let runtimes = provider.runtimes();
    assert!(!runtimes.is_empty());
    let names: Vec<&str> = runtimes
        .iter()
        .map(|r: &std::sync::Arc<dyn Runtime>| r.name())
        .collect();
    assert!(names.contains(&"msvc"));
    assert!(names.contains(&"nmake"));
    assert!(names.contains(&"link"));
}

#[rstest]
#[case("msvc", true)]
#[case("cl", true)]
#[case("vs-build-tools", true)]
#[case("nmake", true)]
#[case("link", true)]
#[case("node", false)]
fn test_provider_supports(#[case] name: &str, #[case] expected: bool) {
    let provider = create_provider();
    assert_eq!(provider.supports(name), expected);
}

#[test]
fn test_provider_get_runtime() {
    let provider = create_provider();
    assert!(provider.get_runtime("msvc").is_some());
    assert!(provider.get_runtime("cl").is_some());
    assert!(provider.get_runtime("nmake").is_some());
    assert!(provider.get_runtime("unknown").is_none());
}

#[test]
fn test_star_metadata() {
    let meta = star_metadata();
    assert!(meta.name.is_some());
    assert!(!meta.runtimes.is_empty());
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

// ============================================
// Issue #573: MSVC env vars conflict with node-gyp
// ============================================
// These tests verify the fix for #573 where MSVC environment variables
// (LIB, INCLUDE, PATH) injected globally by prepare_environment()
// conflicted with node-gyp's Visual Studio discovery logic.
//
// The fix:
// 1. prepare_environment() now uses VX_MSVC_* prefix (not LIB/INCLUDE/PATH)
// 2. execution_environment() still uses full LIB/INCLUDE/PATH (only for direct MSVC invocation)
// 3. validate_paths() detects stale cached paths
// 4. VCINSTALLDIR/VCToolsInstallDir/GYP_MSVS_VERSION are set for node-gyp discovery

#[test]
fn test_issue_573_get_environment_does_not_use_vx_prefix() {
    // get_environment() is used by execution_environment() for direct MSVC tool invocation.
    // It SHOULD set LIB, INCLUDE, PATH (full env needed for cl.exe, link.exe, etc.)
    let temp_dir = TempDir::new().unwrap();
    let install_path = temp_dir.path().to_path_buf();
    let info = create_test_install_info(install_path);

    let env = info.get_environment();

    // Direct invocation env uses standard names (not VX_MSVC_* prefix)
    assert!(
        env.contains_key("INCLUDE"),
        "execution env should set INCLUDE"
    );
    assert!(env.contains_key("LIB"), "execution env should set LIB");
    assert!(env.contains_key("PATH"), "execution env should set PATH");

    // Should NOT contain VX_MSVC_* prefixed vars (those are for prepare_environment)
    assert!(
        !env.contains_key("VX_MSVC_INCLUDE"),
        "execution env should not use VX_MSVC_INCLUDE"
    );
    assert!(
        !env.contains_key("VX_MSVC_LIB"),
        "execution env should not use VX_MSVC_LIB"
    );
}

#[test]
fn test_issue_573_validate_paths_with_version_mismatch() {
    // Simulates the exact scenario from #573: msvc-info.json cached with version 14.44
    // but actual installation is 14.42, so paths referencing 14.44 don't exist.
    let temp_dir = TempDir::new().unwrap();
    let install_path = temp_dir.path().to_path_buf();

    // Create paths for "14.42" (actual installed version)
    let actual_version_dir = install_path
        .join("VC")
        .join("Tools")
        .join("MSVC")
        .join("14.42.34433");
    let actual_bin = actual_version_dir.join("bin").join("Hostx64").join("x64");
    std::fs::create_dir_all(&actual_bin).unwrap();
    std::fs::write(actual_bin.join("cl.exe"), "fake").unwrap();

    // But msvc-info.json references paths for "14.44" (stale cache)
    let stale_version_dir = install_path
        .join("VC")
        .join("Tools")
        .join("MSVC")
        .join("14.44.35207");
    let stale_bin = stale_version_dir.join("bin").join("Hostx64").join("x64");

    let info = MsvcInstallInfo {
        install_path,
        msvc_version: "14.44.35207".to_string(),
        sdk_version: None,
        cl_exe_path: stale_bin.join("cl.exe"), // doesn't exist!
        link_exe_path: Some(stale_bin.join("link.exe")),
        lib_exe_path: None,
        nmake_exe_path: None,
        include_paths: vec![stale_version_dir.join("include")],
        lib_paths: vec![stale_version_dir.join("lib").join("x64")],
        bin_paths: vec![stale_bin],
    };

    // validate_paths should detect that cl.exe doesn't exist
    assert!(
        !info.validate_paths(),
        "validate_paths should fail when cl.exe path is stale"
    );
}

#[test]
fn test_issue_573_get_environment_only_includes_existing_paths_in_include() {
    // Verifies that get_environment() filters out non-existent paths from INCLUDE.
    // This prevents stale MSVC version paths from being injected into node-gyp's
    // csc.exe (C# compiler) environment, which was the root cause of #573.
    let temp_dir = TempDir::new().unwrap();
    let install_path = temp_dir.path().to_path_buf();

    // Create one real MSVC include path and one real SDK include path
    let msvc_include = install_path
        .join("VC")
        .join("Tools")
        .join("MSVC")
        .join("14.42")
        .join("include");
    let sdk_include = install_path
        .join("SDK")
        .join("Include")
        .join("10.0.22621.0")
        .join("ucrt");
    std::fs::create_dir_all(&msvc_include).unwrap();
    std::fs::create_dir_all(&sdk_include).unwrap();

    // Simulate stale paths from a different MSVC version
    let stale_include = install_path
        .join("VC")
        .join("Tools")
        .join("MSVC")
        .join("14.44")
        .join("include");

    let info = MsvcInstallInfo {
        install_path: install_path.clone(),
        msvc_version: "14.42".to_string(),
        sdk_version: Some("10.0.22621.0".to_string()),
        cl_exe_path: install_path.join("cl.exe"),
        link_exe_path: None,
        lib_exe_path: None,
        nmake_exe_path: None,
        include_paths: vec![
            msvc_include.clone(),  // exists
            stale_include.clone(), // doesn't exist (stale from different version)
            sdk_include.clone(),   // exists
        ],
        lib_paths: vec![],
        bin_paths: vec![],
    };

    // validated_include_paths should only return 2 paths (existing ones)
    let valid = info.validated_include_paths();
    assert_eq!(valid.len(), 2, "should only return existing include paths");
    assert!(
        valid
            .iter()
            .any(|p| p.ends_with("14.42/include") || p.ends_with("14.42\\include"))
    );
    assert!(valid.iter().any(|p| p.to_string_lossy().contains("ucrt")));

    // The INCLUDE env var should not contain the stale path
    let env = info.get_environment();
    if let Some(include) = env.get("INCLUDE") {
        assert!(
            !include.contains("14.44"),
            "INCLUDE should not contain stale version 14.44 paths"
        );
        assert!(
            include.contains("14.42"),
            "INCLUDE should contain valid version 14.42 paths"
        );
    }
}

#[test]
fn test_issue_573_validate_paths_empty_include_lib_is_ok() {
    // When include_paths and lib_paths are empty, validation should still pass
    // as long as cl.exe exists. This handles fresh installs where SDK wasn't requested.
    let temp_dir = TempDir::new().unwrap();
    let install_path = temp_dir.path().to_path_buf();

    let cl_path = install_path.join("cl.exe");
    std::fs::write(&cl_path, "fake").unwrap();

    let info = MsvcInstallInfo {
        install_path,
        msvc_version: "14.42".to_string(),
        sdk_version: None,
        cl_exe_path: cl_path,
        link_exe_path: None,
        lib_exe_path: None,
        nmake_exe_path: None,
        include_paths: vec![],
        lib_paths: vec![],
        bin_paths: vec![],
    };

    assert!(
        info.validate_paths(),
        "validate_paths should pass when cl.exe exists and include/lib are empty"
    );
}

#[test]
fn test_issue_573_validate_paths_partial_include_paths() {
    // When at least one include path exists, validation should pass
    let temp_dir = TempDir::new().unwrap();
    let install_path = temp_dir.path().to_path_buf();

    let cl_path = install_path.join("cl.exe");
    std::fs::write(&cl_path, "fake").unwrap();

    let real_include = install_path.join("include_real");
    std::fs::create_dir_all(&real_include).unwrap();

    let info = MsvcInstallInfo {
        install_path: install_path.clone(),
        msvc_version: "14.42".to_string(),
        sdk_version: None,
        cl_exe_path: cl_path,
        link_exe_path: None,
        lib_exe_path: None,
        nmake_exe_path: None,
        include_paths: vec![
            real_include,
            install_path.join("include_stale"), // doesn't exist
        ],
        lib_paths: vec![],
        bin_paths: vec![],
    };

    assert!(
        info.validate_paths(),
        "validate_paths should pass when at least one include path exists"
    );
}

#[test]
fn test_issue_573_validate_paths_all_include_paths_stale() {
    // When ALL include paths are stale (none exist), validation should fail
    let temp_dir = TempDir::new().unwrap();
    let install_path = temp_dir.path().to_path_buf();

    let cl_path = install_path.join("cl.exe");
    std::fs::write(&cl_path, "fake").unwrap();

    let info = MsvcInstallInfo {
        install_path: install_path.clone(),
        msvc_version: "14.44".to_string(),
        sdk_version: None,
        cl_exe_path: cl_path,
        link_exe_path: None,
        lib_exe_path: None,
        nmake_exe_path: None,
        include_paths: vec![
            install_path.join("stale_include_1"),
            install_path.join("stale_include_2"),
        ],
        lib_paths: vec![],
        bin_paths: vec![],
    };

    assert!(
        !info.validate_paths(),
        "validate_paths should fail when all include paths are stale"
    );
}

#[test]
fn test_issue_573_normalize_version_strips_patch() {
    // MsvcInstaller::normalize_version should reduce "14.40.33807" to "14.40"
    // because msvc-kit expects major.minor format
    let installer = MsvcInstaller::new("14.40.33807");
    assert_eq!(
        installer.msvc_version,
        Some("14.40".to_string()),
        "normalize_version should strip patch from version"
    );
}

#[test]
fn test_issue_573_normalize_version_keeps_major_minor() {
    let installer = MsvcInstaller::new("14.42");
    assert_eq!(
        installer.msvc_version,
        Some("14.42".to_string()),
        "normalize_version should keep major.minor as-is"
    );
}

#[test]
fn test_issue_573_normalize_version_single_component() {
    let installer = MsvcInstaller::new("14");
    assert_eq!(
        installer.msvc_version,
        Some("14".to_string()),
        "normalize_version should keep single component as-is"
    );
}

#[test]
fn test_issue_573_installer_latest_no_version() {
    let installer = MsvcInstaller::latest();
    assert_eq!(
        installer.msvc_version, None,
        "latest() should not set msvc_version"
    );
}

// ============================================
// MSBuild Bridge Deployment Tests
// ============================================
// These tests verify that the MSBuild bridge is properly deployed
// to the MSVC installation directory after installation, enabling
// node-gyp and other build tools to discover MSBuild.exe.

#[test]
fn test_deploy_msbuild_bridge_creates_expected_path() {
    let temp_dir = TempDir::new().unwrap();
    let install_path = temp_dir.path().to_path_buf();

    // Use a unique bridge name to avoid global registry conflicts with other tests
    let bridge_name = "MSBuild_deploy_path_test";
    let data = Vec::from(b"MZ-fake-msbuild-bridge-for-test" as &[u8]);
    let fake_msbuild: &'static [u8] = Box::leak(data.into_boxed_slice());
    vx_bridge::register_embedded_bridge(bridge_name, fake_msbuild);

    // Call deploy_bridge to the expected path (same as MsvcRuntime::deploy_msbuild_bridge)
    let target = install_path
        .join("MSBuild")
        .join("Current")
        .join("Bin")
        .join("MSBuild.exe");

    let result = vx_bridge::deploy_bridge(bridge_name, &target);
    assert!(
        result.is_ok(),
        "MSBuild bridge deployment should succeed: {:?}",
        result.err()
    );

    // Verify the bridge was deployed to the expected location
    assert!(
        target.exists(),
        "MSBuild.exe should exist at MSBuild/Current/Bin/MSBuild.exe"
    );

    // Verify the content matches what was registered
    let content = std::fs::read(&target).unwrap();
    assert_eq!(content, b"MZ-fake-msbuild-bridge-for-test");
}

#[test]
fn test_msbuild_bridge_path_matches_node_gyp_expectation() {
    // node-gyp expects MSBuild.exe at:
    // {VCINSTALLDIR}/../MSBuild/Current/Bin/MSBuild.exe
    // Since VCINSTALLDIR points to {install_path}/VC/,
    // resolving VCINSTALLDIR/.. gives {install_path}/
    // So MSBuild.exe should be at {install_path}/MSBuild/Current/Bin/MSBuild.exe
    let temp_dir = TempDir::new().unwrap();
    let install_path = temp_dir.path().to_path_buf();

    // Create the VC directory structure
    let vc_dir = install_path.join("VC");
    std::fs::create_dir_all(&vc_dir).unwrap();

    // Use a unique bridge name to avoid global registry conflicts
    let bridge_name = "MSBuild_node_gyp_test";
    let data = Vec::from(b"msbuild-content" as &[u8]);
    let bridge_data: &'static [u8] = Box::leak(data.into_boxed_slice());
    vx_bridge::register_embedded_bridge(bridge_name, bridge_data);

    let target = install_path
        .join("MSBuild")
        .join("Current")
        .join("Bin")
        .join("MSBuild.exe");
    vx_bridge::deploy_bridge(bridge_name, &target).unwrap();

    // Verify the path relative to VCINSTALLDIR
    // VCINSTALLDIR = {install_path}/VC/ → parent = {install_path}
    let vc_install_dir = vc_dir.to_path_buf();
    let vs_root = vc_install_dir.parent().unwrap();
    let expected_msbuild = vs_root
        .join("MSBuild")
        .join("Current")
        .join("Bin")
        .join("MSBuild.exe");

    assert!(
        expected_msbuild.exists(),
        "MSBuild.exe should be discoverable via VCINSTALLDIR parent: {}",
        expected_msbuild.display()
    );
}

#[test]
fn test_msbuild_bridge_directory_structure() {
    // Verify that deployment creates the full directory tree
    let temp_dir = TempDir::new().unwrap();
    let install_path = temp_dir.path().to_path_buf();

    // Use a unique bridge name to avoid global registry conflicts
    let bridge_name = "MSBuild_dir_structure_test";
    let data = Vec::from(b"bridge-exe" as &[u8]);
    let bridge_data: &'static [u8] = Box::leak(data.into_boxed_slice());
    vx_bridge::register_embedded_bridge(bridge_name, bridge_data);

    let target = install_path
        .join("MSBuild")
        .join("Current")
        .join("Bin")
        .join("MSBuild.exe");
    vx_bridge::deploy_bridge(bridge_name, &target).unwrap();

    // Verify directory structure
    assert!(install_path.join("MSBuild").is_dir());
    assert!(install_path.join("MSBuild").join("Current").is_dir());
    assert!(
        install_path
            .join("MSBuild")
            .join("Current")
            .join("Bin")
            .is_dir()
    );
    assert!(target.is_file());
}

#[test]
fn test_get_environment_includes_msbuild_discoverable_path() {
    // After MSVC installation with bridge deployed, the environment
    // should make MSBuild.exe discoverable via VCINSTALLDIR
    let temp_dir = TempDir::new().unwrap();
    let install_path = temp_dir.path().to_path_buf();

    // Create VC directory structure
    let vc_dir = install_path.join("VC");
    let tools_dir = vc_dir.join("Tools").join("MSVC").join("14.42.34433");
    std::fs::create_dir_all(&tools_dir).unwrap();

    // Deploy MSBuild bridge (unique name to avoid global registry conflicts)
    let bridge_name = "MSBuild_env_test";
    let data = Vec::from(b"msbuild" as &[u8]);
    let bridge_data: &'static [u8] = Box::leak(data.into_boxed_slice());
    vx_bridge::register_embedded_bridge(bridge_name, bridge_data);
    let msbuild_path = install_path
        .join("MSBuild")
        .join("Current")
        .join("Bin")
        .join("MSBuild.exe");
    vx_bridge::deploy_bridge(bridge_name, &msbuild_path).unwrap();

    let info = MsvcInstallInfo {
        install_path: install_path.clone(),
        msvc_version: "14.42.34433".to_string(),
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

    // VCINSTALLDIR should be set
    let vcinstalldir = env.get("VCINSTALLDIR").unwrap();
    assert!(vcinstalldir.contains("VC"));

    // Verify that MSBuild.exe is discoverable relative to VCINSTALLDIR
    let vc_path = PathBuf::from(vcinstalldir.trim_end_matches('\\'));
    let vs_root = vc_path.parent().unwrap();
    let msbuild = vs_root
        .join("MSBuild")
        .join("Current")
        .join("Bin")
        .join("MSBuild.exe");
    assert!(
        msbuild.exists(),
        "MSBuild.exe should be discoverable via VCINSTALLDIR: {}",
        msbuild.display()
    );
}
