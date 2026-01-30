//! VxPaths tests

use std::path::PathBuf;
use vx_paths::{executable_extension, normalize_package_name, with_executable_extension, VxPaths};

#[test]
fn test_vx_paths_creation() {
    let paths = VxPaths::with_base_dir("/tmp/test-vx");

    assert_eq!(paths.base_dir, PathBuf::from("/tmp/test-vx"));
    assert_eq!(paths.store_dir, PathBuf::from("/tmp/test-vx/store"));
    assert_eq!(paths.envs_dir, PathBuf::from("/tmp/test-vx/envs"));
    assert_eq!(paths.bin_dir, PathBuf::from("/tmp/test-vx/bin"));
    assert_eq!(paths.cache_dir, PathBuf::from("/tmp/test-vx/cache"));
    assert_eq!(paths.config_dir, PathBuf::from("/tmp/test-vx/config"));
    assert_eq!(paths.tmp_dir, PathBuf::from("/tmp/test-vx/tmp"));
    // RFC 0025: New directories
    assert_eq!(paths.packages_dir, PathBuf::from("/tmp/test-vx/packages"));
    assert_eq!(paths.shims_dir, PathBuf::from("/tmp/test-vx/shims"));
}

#[test]
fn test_runtime_store_dir() {
    let paths = VxPaths::with_base_dir("/tmp/test-vx");

    assert_eq!(
        paths.runtime_store_dir("node"),
        PathBuf::from("/tmp/test-vx/store/node")
    );
}

#[test]
fn test_version_store_dir() {
    let paths = VxPaths::with_base_dir("/tmp/test-vx");

    assert_eq!(
        paths.version_store_dir("node", "20.0.0"),
        PathBuf::from("/tmp/test-vx/store/node/20.0.0")
    );
}

#[test]
fn test_env_dir() {
    let paths = VxPaths::with_base_dir("/tmp/test-vx");

    assert_eq!(
        paths.env_dir("my-project"),
        PathBuf::from("/tmp/test-vx/envs/my-project")
    );
}

#[test]
fn test_default_env_dir() {
    let paths = VxPaths::with_base_dir("/tmp/test-vx");

    assert_eq!(
        paths.default_env_dir(),
        PathBuf::from("/tmp/test-vx/envs/default")
    );
}

#[test]
fn test_executable_extension() {
    if cfg!(target_os = "windows") {
        assert_eq!(executable_extension(), ".exe");
        assert_eq!(with_executable_extension("node"), "node.exe");
    } else {
        assert_eq!(executable_extension(), "");
        assert_eq!(with_executable_extension("node"), "node");
    }
}

// ========== RFC 0025: Global Packages Tests ==========

#[test]
fn test_ecosystem_packages_dir() {
    let paths = VxPaths::with_base_dir("/tmp/test-vx");

    assert_eq!(
        paths.ecosystem_packages_dir("npm"),
        PathBuf::from("/tmp/test-vx/packages/npm")
    );
    assert_eq!(
        paths.ecosystem_packages_dir("pip"),
        PathBuf::from("/tmp/test-vx/packages/pip")
    );
    assert_eq!(
        paths.ecosystem_packages_dir("cargo"),
        PathBuf::from("/tmp/test-vx/packages/cargo")
    );
}

#[test]
fn test_global_package_dir() {
    let paths = VxPaths::with_base_dir("/tmp/test-vx");

    let pkg_dir = paths.global_package_dir("npm", "typescript", "5.3.3");
    assert!(pkg_dir.ends_with("packages/npm/typescript/5.3.3"));
}

#[test]
fn test_global_package_bin_dir() {
    let paths = VxPaths::with_base_dir("/tmp/test-vx");

    let bin_dir = paths.global_package_bin_dir("npm", "typescript", "5.3.3");
    assert!(bin_dir.ends_with("packages/npm/typescript/5.3.3/bin"));
}

#[test]
fn test_global_pip_venv_dir() {
    let paths = VxPaths::with_base_dir("/tmp/test-vx");

    let venv_dir = paths.global_pip_venv_dir("black", "24.1.0");
    assert!(venv_dir.ends_with("packages/pip/black/24.1.0/venv"));
}

#[test]
fn test_global_npm_node_modules_dir() {
    let paths = VxPaths::with_base_dir("/tmp/test-vx");

    let nm_dir = paths.global_npm_node_modules_dir("typescript", "5.3.3");
    assert!(nm_dir.ends_with("packages/npm/typescript/5.3.3/node_modules"));
}

#[test]
fn test_project_bin_dir() {
    let paths = VxPaths::with_base_dir("/tmp/test-vx");
    let project_root = PathBuf::from("/home/user/my-project");

    let bin_dir = paths.project_bin_dir(&project_root);
    assert_eq!(bin_dir, PathBuf::from("/home/user/my-project/.vx/bin"));
}

#[test]
fn test_global_tools_config() {
    let paths = VxPaths::with_base_dir("/tmp/test-vx");

    let config = paths.global_tools_config();
    assert!(config.ends_with("config/global-tools.toml"));
}

#[test]
fn test_packages_registry_file() {
    let paths = VxPaths::with_base_dir("/tmp/test-vx");

    let registry = paths.packages_registry_file();
    assert!(registry.ends_with("config/packages-registry.json"));
}

#[test]
fn test_normalize_package_name() {
    // On case-insensitive filesystems (Windows/macOS), normalize to lowercase
    // On Linux, keep original case
    let normalized = normalize_package_name("TypeScript");

    #[cfg(any(target_os = "windows", target_os = "macos"))]
    assert_eq!(normalized, "typescript");

    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    assert_eq!(normalized, "TypeScript");
}
