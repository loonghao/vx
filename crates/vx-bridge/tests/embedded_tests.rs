//! Tests for the embedded bridge registry.

use tempfile::TempDir;

// ============================================
// register_embedded_bridge Tests
// ============================================

#[test]
fn test_register_and_deploy_embedded_bridge() {
    let temp_dir = TempDir::new().unwrap();
    let target_path = temp_dir.path().join("TestBridge.exe");

    // Register a fake bridge binary using a leaked Vec for 'static lifetime
    let data = Vec::from(b"fake-bridge-binary-content" as &[u8]);
    let static_data: &'static [u8] = Box::leak(data.into_boxed_slice());
    vx_bridge::register_embedded_bridge("TestBridge", static_data);

    // Deploy it
    let result = vx_bridge::deploy_embedded_bridge("TestBridge", &target_path);
    assert!(result.is_ok(), "deploy should succeed after registration");

    let deployed_path = result.unwrap();
    assert_eq!(deployed_path, target_path);

    // Verify the file was written with correct content
    let content = std::fs::read(&target_path).unwrap();
    assert_eq!(content, b"fake-bridge-binary-content");
}

#[test]
fn test_deploy_unregistered_bridge_fails() {
    let temp_dir = TempDir::new().unwrap();
    let target_path = temp_dir.path().join("NonExistent.exe");

    let result = vx_bridge::deploy_embedded_bridge("NonExistentBridge", &target_path);
    assert!(
        result.is_err(),
        "deploy should fail for unregistered bridge"
    );
}

#[test]
fn test_register_empty_data_is_ignored() {
    let temp_dir = TempDir::new().unwrap();
    let target_path = temp_dir.path().join("EmptyBridge.exe");

    // Register empty data — should be silently ignored
    let empty_data: &'static [u8] = &[];
    vx_bridge::register_embedded_bridge("EmptyBridge", empty_data);

    // Deploying should fail since empty data was not registered
    let result = vx_bridge::deploy_embedded_bridge("EmptyBridge", &target_path);
    assert!(result.is_err(), "deploy should fail for empty bridge data");
}

#[test]
fn test_deploy_creates_parent_directories() {
    let temp_dir = TempDir::new().unwrap();
    let target_path = temp_dir
        .path()
        .join("deep")
        .join("nested")
        .join("dir")
        .join("Bridge.exe");

    let data = Vec::from(b"nested-bridge" as &[u8]);
    let static_data: &'static [u8] = Box::leak(data.into_boxed_slice());
    vx_bridge::register_embedded_bridge("NestedBridge", static_data);

    let result = vx_bridge::deploy_embedded_bridge("NestedBridge", &target_path);
    assert!(
        result.is_ok(),
        "deploy should create parent directories: {:?}",
        result.err()
    );
    assert!(target_path.exists());
}

#[test]
fn test_deploy_overwrites_existing_file() {
    let temp_dir = TempDir::new().unwrap();
    let target_path = temp_dir.path().join("OverwriteBridge.exe");

    // Write an existing file
    std::fs::write(&target_path, b"old-content").unwrap();

    let data = Vec::from(b"new-bridge-content-v2" as &[u8]);
    let static_data: &'static [u8] = Box::leak(data.into_boxed_slice());
    vx_bridge::register_embedded_bridge("OverwriteBridge", static_data);

    let result = vx_bridge::deploy_embedded_bridge("OverwriteBridge", &target_path);
    assert!(result.is_ok());

    // Verify content was overwritten
    let content = std::fs::read(&target_path).unwrap();
    assert_eq!(content, b"new-bridge-content-v2");
}

// ============================================
// DeployEmbeddedError Display Tests
// ============================================

#[test]
fn test_deploy_embedded_error_not_registered_display() {
    let temp_dir = TempDir::new().unwrap();
    let target_path = temp_dir.path().join("Missing.exe");

    let err = vx_bridge::deploy_embedded_bridge("MissingBridge_Display", &target_path).unwrap_err();
    let msg = format!("{}", err);
    assert!(
        msg.contains("not registered"),
        "error message should mention registration: {}",
        msg
    );
}

// ============================================
// deploy_bridge Tests (filesystem + embedded fallback)
// ============================================

#[test]
fn test_deploy_bridge_uses_embedded_fallback() {
    let temp_dir = TempDir::new().unwrap();
    let target_path = temp_dir
        .path()
        .join("MSBuild")
        .join("Current")
        .join("Bin")
        .join("MSBuild.exe");

    // Register a bridge with embedded data (unique name to avoid conflicts)
    let data = Vec::from(b"embedded-msbuild-fallback" as &[u8]);
    let static_data: &'static [u8] = Box::leak(data.into_boxed_slice());
    vx_bridge::register_embedded_bridge("EmbeddedFallbackBridge", static_data);

    // deploy_bridge should fall through filesystem search and use embedded fallback
    let result = vx_bridge::deploy_bridge("EmbeddedFallbackBridge", &target_path);
    assert!(
        result.is_ok(),
        "deploy_bridge should succeed via embedded fallback: {:?}",
        result.err()
    );
    assert!(target_path.exists());
    let content = std::fs::read(&target_path).unwrap();
    assert_eq!(content, b"embedded-msbuild-fallback");
}

#[test]
fn test_deploy_bridge_filesystem_copy() {
    let temp_dir = TempDir::new().unwrap();

    // Create a "bridge" file in a directory we control via VX_BRIDGE_DIR
    let bridge_dir = temp_dir.path().join("bridges");
    std::fs::create_dir_all(&bridge_dir).unwrap();

    let bridge_name = if cfg!(windows) {
        "TestFSBridge.exe"
    } else {
        "TestFSBridge"
    };
    let source_bridge = bridge_dir.join(bridge_name);
    std::fs::write(&source_bridge, b"fs-bridge-content").unwrap();

    // Set VX_BRIDGE_DIR to our temp directory
    // SAFETY: This test is not parallelized with other tests that use VX_BRIDGE_DIR
    unsafe {
        std::env::set_var("VX_BRIDGE_DIR", bridge_dir.to_str().unwrap());
    }

    let target_path = temp_dir.path().join("deployed").join(bridge_name);
    let result = vx_bridge::deploy_bridge("TestFSBridge", &target_path);

    // Clean up env var
    unsafe {
        std::env::remove_var("VX_BRIDGE_DIR");
    }

    assert!(
        result.is_ok(),
        "deploy_bridge should succeed via filesystem: {:?}",
        result.err()
    );
    assert!(target_path.exists());
    let content = std::fs::read(&target_path).unwrap();
    assert_eq!(content, b"fs-bridge-content");
}

#[test]
fn test_deploy_bridge_not_found_anywhere() {
    let temp_dir = TempDir::new().unwrap();
    let target_path = temp_dir.path().join("NoWhere.exe");

    let result = vx_bridge::deploy_bridge("CompletelyNonExistentBridge12345", &target_path);
    assert!(result.is_err(), "deploy_bridge should fail when not found");

    let err = result.unwrap_err();
    let msg = format!("{}", err);
    assert!(
        msg.contains("not found"),
        "error should indicate not found: {}",
        msg
    );
}
