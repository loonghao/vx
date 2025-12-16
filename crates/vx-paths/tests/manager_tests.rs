//! PathManager tests

use tempfile::TempDir;
use vx_paths::PathManager;

#[test]
fn test_path_manager_creation() {
    let temp_dir = TempDir::new().unwrap();
    let base_dir = temp_dir.path().join(".vx");
    let manager = PathManager::with_base_dir(&base_dir).unwrap();

    assert!(manager.store_dir().exists());
    assert!(manager.envs_dir().exists());
    assert!(manager.bin_dir().exists());
    assert!(manager.cache_dir().exists());
    assert!(manager.config_dir().exists());
    assert!(manager.tmp_dir().exists());
}

#[test]
fn test_store_paths() {
    let temp_dir = TempDir::new().unwrap();
    let base_dir = temp_dir.path().join(".vx");
    let manager = PathManager::with_base_dir(&base_dir).unwrap();

    let runtime_dir = manager.runtime_store_dir("node");
    let version_dir = manager.version_store_dir("node", "20.0.0");
    let exe_path = manager.store_executable_path("node", "20.0.0");

    assert_eq!(runtime_dir, base_dir.join("store/node"));
    assert_eq!(version_dir, base_dir.join("store/node/20.0.0"));

    if cfg!(target_os = "windows") {
        assert_eq!(exe_path, base_dir.join("store/node/20.0.0/bin/node.exe"));
    } else {
        assert_eq!(exe_path, base_dir.join("store/node/20.0.0/bin/node"));
    }
}

#[test]
fn test_env_paths() {
    let temp_dir = TempDir::new().unwrap();
    let base_dir = temp_dir.path().join(".vx");
    let manager = PathManager::with_base_dir(&base_dir).unwrap();

    let env_dir = manager.env_dir("my-project");
    let default_env = manager.default_env_dir();
    let runtime_path = manager.env_runtime_path("my-project", "node");

    assert_eq!(env_dir, base_dir.join("envs/my-project"));
    assert_eq!(default_env, base_dir.join("envs/default"));
    assert_eq!(runtime_path, base_dir.join("envs/my-project/node"));
}

#[test]
fn test_env_management() {
    let temp_dir = TempDir::new().unwrap();
    let base_dir = temp_dir.path().join(".vx");
    let manager = PathManager::with_base_dir(&base_dir).unwrap();

    // Initially no envs
    assert!(!manager.env_exists("test-env"));
    assert!(manager.list_envs().unwrap().is_empty());

    // Create env
    manager.create_env("test-env").unwrap();
    assert!(manager.env_exists("test-env"));
    assert_eq!(manager.list_envs().unwrap(), vec!["test-env"]);

    // Remove env
    manager.remove_env("test-env").unwrap();
    assert!(!manager.env_exists("test-env"));
}

#[test]
fn test_store_version_check() {
    let temp_dir = TempDir::new().unwrap();
    let base_dir = temp_dir.path().join(".vx");
    let manager = PathManager::with_base_dir(&base_dir).unwrap();

    // Initially not in store
    assert!(!manager.is_version_in_store("node", "20.0.0"));

    // Create version directory
    let version_dir = manager.version_store_dir("node", "20.0.0");
    std::fs::create_dir_all(&version_dir).unwrap();

    // Now it should be detected
    assert!(manager.is_version_in_store("node", "20.0.0"));
    assert_eq!(manager.list_store_versions("node").unwrap(), vec!["20.0.0"]);
    assert_eq!(manager.list_store_runtimes().unwrap(), vec!["node"]);
}
