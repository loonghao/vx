use tempfile::TempDir;
use vx_paths::PathManager;

#[test]
fn test_path_manager_creation() {
    let temp_dir = TempDir::new().unwrap();
    let base_dir = temp_dir.path().join(".vx");
    let manager = PathManager::with_base_dir(&base_dir).unwrap();

    assert!(manager.tools_dir().exists());
    assert!(manager.cache_dir().exists());
    assert!(manager.config_dir().exists());
    assert!(manager.tmp_dir().exists());
}

#[test]
fn test_tool_paths() {
    let temp_dir = TempDir::new().unwrap();
    let base_dir = temp_dir.path().join(".vx");
    let manager = PathManager::with_base_dir(&base_dir).unwrap();

    let tool_dir = manager.tool_dir("node");
    let version_dir = manager.tool_version_dir("node", "18.17.0");
    let exe_path = manager.tool_executable_path("node", "18.17.0");

    assert_eq!(tool_dir, base_dir.join("tools/node"));
    assert_eq!(version_dir, base_dir.join("tools/node/18.17.0"));

    if cfg!(target_os = "windows") {
        assert_eq!(exe_path, base_dir.join("tools/node/18.17.0/node.exe"));
    } else {
        assert_eq!(exe_path, base_dir.join("tools/node/18.17.0/node"));
    }
}

#[test]
fn test_tool_version_management() {
    let temp_dir = TempDir::new().unwrap();
    let base_dir = temp_dir.path().join(".vx");
    let manager = PathManager::with_base_dir(&base_dir).unwrap();

    // Initially no versions
    assert!(!manager.is_tool_version_installed("node", "18.17.0"));
    assert_eq!(
        manager.list_tool_versions("node").unwrap(),
        Vec::<String>::new()
    );

    // Create version directory and executable
    let _version_dir = manager.create_tool_version_dir("node", "18.17.0").unwrap();
    let exe_path = manager.tool_executable_path("node", "18.17.0");
    std::fs::write(&exe_path, "fake executable").unwrap();

    // Now it should be detected
    assert!(manager.is_tool_version_installed("node", "18.17.0"));
    assert_eq!(manager.list_tool_versions("node").unwrap(), vec!["18.17.0"]);
    assert_eq!(
        manager.get_latest_tool_version("node").unwrap(),
        Some("18.17.0".to_string())
    );
    assert_eq!(manager.list_installed_tools().unwrap(), vec!["node"]);
}

#[test]
fn test_tool_executable_path_node() {
    let temp_dir = TempDir::new().unwrap();
    let base_dir = temp_dir.path().join(".vx");
    let manager = PathManager::with_base_dir(&base_dir).unwrap();

    let exe_path = manager.tool_executable_path("node", "20.11.0");

    // Node executable path should be correct
    if cfg!(target_os = "windows") {
        assert!(exe_path.to_string_lossy().contains("node.exe"));
    } else {
        assert!(exe_path.to_string_lossy().contains("node"));
    }
}

#[test]
fn test_tool_executable_path_go() {
    let temp_dir = TempDir::new().unwrap();
    let base_dir = temp_dir.path().join(".vx");
    let manager = PathManager::with_base_dir(&base_dir).unwrap();

    let exe_path = manager.tool_executable_path("go", "1.21.6");

    // Go executable path should be correct
    if cfg!(target_os = "windows") {
        assert!(exe_path.to_string_lossy().contains("go.exe"));
    } else {
        assert!(exe_path.to_string_lossy().contains("go"));
    }
}

#[test]
fn test_tool_executable_path_yarn() {
    let temp_dir = TempDir::new().unwrap();
    let base_dir = temp_dir.path().join(".vx");
    let manager = PathManager::with_base_dir(&base_dir).unwrap();

    let exe_path = manager.tool_executable_path("yarn", "1.22.19");

    // Yarn executable path should be correct
    assert!(exe_path.to_string_lossy().contains("yarn"));
}

#[test]
fn test_current_version_paths() {
    let temp_dir = TempDir::new().unwrap();
    let base_dir = temp_dir.path().join(".vx");
    let manager = PathManager::with_base_dir(&base_dir).unwrap();

    let current_dir = manager.tool_current_dir("node");
    let current_exe = manager.tool_current_executable_path("node");
    let current_config = manager.tool_current_shim_config_path("node");

    assert_eq!(
        current_dir,
        base_dir.join("tools").join("node").join("current")
    );
    assert_eq!(current_config, current_dir.join("node.shim.toml"));

    if cfg!(target_os = "windows") {
        assert_eq!(current_exe, current_dir.join("node.bat"));
    } else {
        assert_eq!(current_exe, current_dir.join("node"));
    }
}

#[test]
fn test_cache_and_tmp_paths() {
    let temp_dir = TempDir::new().unwrap();
    let base_dir = temp_dir.path().join(".vx");
    let manager = PathManager::with_base_dir(&base_dir).unwrap();

    let cache_dir = manager.tool_cache_dir("node");
    let tmp_dir = manager.tool_tmp_dir("node", "20.11.0");

    assert_eq!(cache_dir, base_dir.join("cache").join("node"));
    assert_eq!(tmp_dir, base_dir.join("tmp").join("node-20.11.0"));
}

#[test]
fn test_multiple_tool_versions() {
    let temp_dir = TempDir::new().unwrap();
    let base_dir = temp_dir.path().join(".vx");
    let manager = PathManager::with_base_dir(&base_dir).unwrap();

    // Create multiple versions
    let versions = ["18.17.0", "20.11.0", "21.0.0"];
    for version in &versions {
        let _version_dir = manager.create_tool_version_dir("node", version).unwrap();
        let exe_path = manager.tool_executable_path("node", version);
        std::fs::write(&exe_path, "fake executable").unwrap();
    }

    let installed_versions = manager.list_tool_versions("node").unwrap();
    assert_eq!(installed_versions.len(), 3);

    // Versions should be sorted
    for version in &versions {
        assert!(installed_versions.contains(&version.to_string()));
    }
}

#[test]
fn test_tool_removal() {
    let temp_dir = TempDir::new().unwrap();
    let base_dir = temp_dir.path().join(".vx");
    let manager = PathManager::with_base_dir(&base_dir).unwrap();

    // Create and install a version
    let _version_dir = manager.create_tool_version_dir("node", "18.17.0").unwrap();
    let exe_path = manager.tool_executable_path("node", "18.17.0");
    std::fs::write(&exe_path, "fake executable").unwrap();

    assert!(manager.is_tool_version_installed("node", "18.17.0"));

    // Remove the version
    manager.remove_tool_version("node", "18.17.0").unwrap();

    assert!(!manager.is_tool_version_installed("node", "18.17.0"));
    assert_eq!(
        manager.list_tool_versions("node").unwrap(),
        Vec::<String>::new()
    );
}

#[test]
fn test_has_current_version() {
    let temp_dir = TempDir::new().unwrap();
    let base_dir = temp_dir.path().join(".vx");
    let manager = PathManager::with_base_dir(&base_dir).unwrap();

    // Initially no current version
    assert!(!manager.has_current_version("node"));

    // Create current directory and config
    let _current_dir = manager.create_tool_current_dir("node").unwrap();
    let config_path = manager.tool_current_shim_config_path("node");
    std::fs::write(&config_path, "fake config").unwrap();

    // Now should have current version
    assert!(manager.has_current_version("node"));
}
