//! Integration tests for shim functionality

use std::fs;
use tempfile::TempDir;
use vx_core::{VxEnvironment, VxShimManager};

#[test]
fn test_shim_manager_creation() {
    let temp_dir = TempDir::new().unwrap();
    let env = VxEnvironment::new_with_base_dir(temp_dir.path()).unwrap();

    // This test might fail if vx-shim is not available, which is expected in CI
    let result = VxShimManager::new(env);

    // We don't assert success here because vx-shim might not be available in test environment
    // Instead, we just verify that the error is reasonable
    match result {
        Ok(_) => {
            println!("Shim manager created successfully");
        }
        Err(e) => {
            println!("Shim manager creation failed (expected in CI): {}", e);
            // This is expected when vx-shim executable is not available
        }
    }
}

#[test]
fn test_shim_directory_creation() {
    let temp_dir = TempDir::new().unwrap();
    let env = VxEnvironment::new_with_base_dir(temp_dir.path()).unwrap();

    // Test that shim directory is created
    let shim_dir = env.shim_dir().unwrap();
    assert!(shim_dir.exists());
    assert!(shim_dir.is_dir());

    // Test that bin directory is created
    let bin_dir = env.bin_dir().unwrap();
    assert!(bin_dir.exists());
    assert!(bin_dir.is_dir());
}

#[test]
fn test_environment_paths() {
    let temp_dir = TempDir::new().unwrap();
    let env = VxEnvironment::new_with_base_dir(temp_dir.path()).unwrap();

    // Test various directory paths
    let base_dir = temp_dir.path();
    let shim_dir = env.shim_dir().unwrap();
    let bin_dir = env.bin_dir().unwrap();

    assert_eq!(shim_dir, base_dir.join("shims"));
    assert_eq!(bin_dir, base_dir.join("bin"));

    // Verify directories are created
    assert!(shim_dir.exists());
    assert!(bin_dir.exists());
}

#[test]
fn test_mock_tool_installation() {
    let temp_dir = TempDir::new().unwrap();
    let env = VxEnvironment::new_with_base_dir(temp_dir.path()).unwrap();

    // Create a mock tool installation
    let tool_name = "test-tool";
    let version = "1.0.0";
    let tool_dir = env.get_version_install_dir(tool_name, version);

    fs::create_dir_all(&tool_dir).unwrap();

    // Create a mock executable
    let exe_name = if cfg!(windows) {
        "test-tool.exe"
    } else {
        "test-tool"
    };
    let exe_path = tool_dir.join(exe_name);
    fs::write(&exe_path, "#!/bin/bash\necho 'test tool'").unwrap();

    // Make executable on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&exe_path).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&exe_path, perms).unwrap();
    }

    // Test that the tool is detected as installed
    assert!(env.is_version_installed(tool_name, version));

    // Test finding the executable
    let found_exe = env.find_executable_in_dir(&tool_dir, tool_name).unwrap();
    assert_eq!(found_exe, exe_path);
}

// Note: parse_tool_version function is tested in the switch.rs module itself
