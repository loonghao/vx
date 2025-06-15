//! Platform detection and utility tests

use rstest::*;
use vx_core::platform::{Architecture, OperatingSystem, Platform};

/// Test current platform detection
#[rstest]
fn test_current_platform() {
    let platform = Platform::current();

    // Should return a valid platform
    match platform.os {
        OperatingSystem::Windows => {
            #[cfg(target_os = "windows")]
            assert!(true);
            #[cfg(not(target_os = "windows"))]
            panic!("Platform detection mismatch: detected Windows on non-Windows");
        }
        OperatingSystem::MacOS => {
            #[cfg(target_os = "macos")]
            assert!(true);
            #[cfg(not(target_os = "macos"))]
            panic!("Platform detection mismatch: detected macOS on non-macOS");
        }
        OperatingSystem::Linux => {
            #[cfg(target_os = "linux")]
            assert!(true);
            #[cfg(not(target_os = "linux"))]
            panic!("Platform detection mismatch: detected Linux on non-Linux");
        }
        _ => {
            // Other OS types are acceptable
        }
    }

    // Should also detect a valid architecture
    assert!(!matches!(platform.arch, Architecture::Other(_)));
}

/// Test platform string representations
#[rstest]
#[case(OperatingSystem::Windows, "windows")]
#[case(OperatingSystem::MacOS, "macos")]
#[case(OperatingSystem::Linux, "linux")]
fn test_platform_strings(#[case] os: OperatingSystem, #[case] expected: &str) {
    assert_eq!(os.to_string().to_lowercase(), expected);
}

/// Test platform extensions
#[rstest]
fn test_extensions() {
    let platform = Platform::current();

    let archive_ext = platform.archive_extension();
    let exe_ext = platform.executable_extension();

    // Archive extension should not be empty
    assert!(!archive_ext.is_empty());

    // Executable extension depends on platform
    match platform.os {
        OperatingSystem::Windows => assert_eq!(exe_ext, "exe"),
        _ => assert_eq!(exe_ext, ""),
    }
}

/// Test platform-specific string generation
#[rstest]
fn test_platform_specific_strings() {
    let platform = Platform::current();

    // Test Node.js platform strings
    if let Some((os_str, arch_str)) = platform.node_platform_string() {
        assert!(!os_str.is_empty());
        assert!(!arch_str.is_empty());
    }

    // Test Go platform strings
    if let Some((os_str, arch_str)) = platform.go_platform_string() {
        assert!(!os_str.is_empty());
        assert!(!arch_str.is_empty());
    }
}
