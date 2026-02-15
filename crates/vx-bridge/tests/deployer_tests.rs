//! Tests for the bridge deployer module.

use tempfile::TempDir;

// ============================================
// DeployError Display Tests
// ============================================

#[test]
fn test_deploy_error_not_found_display() {
    let temp_dir = TempDir::new().unwrap();
    let target_path = temp_dir.path().join("NotHere.exe");

    let result = vx_bridge::deploy_bridge("NoSuchBridge_9999", &target_path);
    assert!(result.is_err());

    let err_msg = format!("{}", result.unwrap_err());
    assert!(
        err_msg.contains("not found"),
        "error should mention 'not found': {}",
        err_msg
    );
    assert!(
        err_msg.contains("Searched"),
        "error should list searched locations: {}",
        err_msg
    );
}

// ============================================
// Platform executable name Tests
// ============================================

#[test]
fn test_deploy_bridge_platform_exe_name() {
    // On Windows, bridge names should get .exe appended
    // On Unix, they should stay as-is
    // We test this indirectly through the error message
    let temp_dir = TempDir::new().unwrap();
    let target_path = temp_dir.path().join("test.exe");

    let result = vx_bridge::deploy_bridge("PlatformTest", &target_path);
    assert!(result.is_err());

    let err_msg = format!("{}", result.unwrap_err());
    if cfg!(windows) {
        assert!(
            err_msg.contains("PlatformTest.exe"),
            "on Windows, error should reference .exe name: {}",
            err_msg
        );
    } else {
        assert!(
            err_msg.contains("PlatformTest"),
            "on Unix, error should reference plain name: {}",
            err_msg
        );
    }
}
