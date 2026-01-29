//! Platform redirection tests

use std::fs;
use vx_paths::PathManager;

#[test]
fn test_platform_dir_name() {
    let manager = PathManager::new().unwrap();
    let platform_name = manager.platform_dir_name();

    // Platform name should contain os and arch
    assert!(platform_name.contains('-'));

    // Common platforms
    #[cfg(target_os = "windows")]
    {
        assert!(platform_name.starts_with("windows"));
        #[cfg(target_arch = "x86_64")]
        assert_eq!(platform_name, "windows-x64");
    }

    #[cfg(target_os = "macos")]
    {
        assert!(platform_name.starts_with("darwin"));
        #[cfg(target_arch = "x86_64")]
        assert_eq!(platform_name, "darwin-x64");
    }

    #[cfg(target_os = "linux")]
    {
        assert!(platform_name.starts_with("linux"));
        #[cfg(target_arch = "x86_64")]
        assert_eq!(platform_name, "linux-x64");
    }
}

#[test]
fn test_platform_store_dir() {
    let manager = PathManager::new().unwrap();
    let platform_dir = manager.platform_store_dir("node", "20.0.0");

    // Should contain platform-specific subdirectory
    let platform_name = manager.platform_dir_name();
    assert!(platform_dir.ends_with(format!("node/20.0.0/{}", platform_name)));
}

#[test]
fn test_is_version_in_store_with_platform() {
    let manager = PathManager::new().unwrap();

    // Create a platform-specific directory
    let platform_dir = manager.platform_store_dir("test-tool", "1.0.0");
    fs::create_dir_all(&platform_dir).unwrap();

    // Should detect as installed
    assert!(manager.is_version_in_store("test-tool", "1.0.0"));

    // Clean up
    fs::remove_dir_all(&platform_dir).unwrap();

    // Should not detect as installed
    assert!(!manager.is_version_in_store("test-tool", "1.0.0"));
}

#[test]
fn test_list_store_versions_filters_by_platform() {
    let manager = PathManager::new().unwrap();

    // Clean up any existing test-tool directory first
    let runtime_dir = manager.runtime_store_dir("test-tool");
    if runtime_dir.exists() {
        fs::remove_dir_all(&runtime_dir).unwrap();
    }

    // Create base version directory
    let base_dir = manager.version_store_dir("test-tool", "1.0.0");
    fs::create_dir_all(&base_dir).unwrap();

    // Create wrong platform directory
    let wrong_platform = if manager.platform_dir_name().contains("windows") {
        "linux-x64"
    } else {
        "windows-x64"
    };
    let wrong_platform_dir = base_dir.join(wrong_platform);
    fs::create_dir_all(&wrong_platform_dir).unwrap();

    // Should NOT list the version (wrong platform)
    let versions = manager.list_store_versions("test-tool").unwrap();
    assert!(versions.is_empty(), "Expected no versions when only wrong platform exists, but got: {:?}", versions);

    // Create correct platform directory
    let correct_platform_dir = manager.platform_store_dir("test-tool", "1.0.0");
    fs::create_dir_all(&correct_platform_dir).unwrap();

    // Should list the version (correct platform)
    let versions = manager.list_store_versions("test-tool").unwrap();
    assert_eq!(versions, vec!["1.0.0"]);

    // Clean up
    fs::remove_dir_all(&runtime_dir).unwrap();
}
