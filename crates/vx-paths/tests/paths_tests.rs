//! VxPaths tests

use std::path::PathBuf;
use vx_paths::{executable_extension, with_executable_extension, VxPaths};

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

// ============================================================================
// Conda-tools Path Tests
// ============================================================================

#[test]
fn test_conda_tools_dir() {
    let paths = VxPaths::with_base_dir("/tmp/test-vx");
    assert_eq!(
        paths.conda_tools_dir,
        PathBuf::from("/tmp/test-vx/conda-tools")
    );
}

#[test]
fn test_conda_tool_dir() {
    let paths = VxPaths::with_base_dir("/tmp/test-vx");
    assert_eq!(
        paths.conda_tool_dir("pytorch"),
        PathBuf::from("/tmp/test-vx/conda-tools/pytorch")
    );
}

#[test]
fn test_conda_tool_version_dir() {
    let paths = VxPaths::with_base_dir("/tmp/test-vx");
    assert_eq!(
        paths.conda_tool_version_dir("pytorch", "2.2.0"),
        PathBuf::from("/tmp/test-vx/conda-tools/pytorch/2.2.0")
    );
}

#[test]
fn test_conda_tool_env_dir() {
    let paths = VxPaths::with_base_dir("/tmp/test-vx");
    assert_eq!(
        paths.conda_tool_env_dir("pytorch", "2.2.0"),
        PathBuf::from("/tmp/test-vx/conda-tools/pytorch/2.2.0/env")
    );
}

#[test]
fn test_conda_tool_bin_dir() {
    let paths = VxPaths::with_base_dir("/tmp/test-vx");
    let bin_dir = paths.conda_tool_bin_dir("pytorch", "2.2.0");

    if cfg!(windows) {
        assert_eq!(
            bin_dir,
            PathBuf::from("/tmp/test-vx/conda-tools/pytorch/2.2.0/env/Scripts")
        );
    } else {
        assert_eq!(
            bin_dir,
            PathBuf::from("/tmp/test-vx/conda-tools/pytorch/2.2.0/env/bin")
        );
    }
}

// ============================================================================
// Pip-tools Path Tests
// ============================================================================

#[test]
fn test_pip_tool_dir() {
    let paths = VxPaths::with_base_dir("/tmp/test-vx");
    assert_eq!(
        paths.pip_tool_dir("rez"),
        PathBuf::from("/tmp/test-vx/pip-tools/rez")
    );
}

#[test]
fn test_pip_tool_version_dir() {
    let paths = VxPaths::with_base_dir("/tmp/test-vx");
    assert_eq!(
        paths.pip_tool_version_dir("rez", "2.114.0"),
        PathBuf::from("/tmp/test-vx/pip-tools/rez/2.114.0")
    );
}

#[test]
fn test_pip_tool_venv_dir() {
    let paths = VxPaths::with_base_dir("/tmp/test-vx");
    assert_eq!(
        paths.pip_tool_venv_dir("rez", "2.114.0"),
        PathBuf::from("/tmp/test-vx/pip-tools/rez/2.114.0/venv")
    );
}

// ============================================================================
// Npm-tools Path Tests
// ============================================================================

#[test]
fn test_npm_tool_dir() {
    let paths = VxPaths::with_base_dir("/tmp/test-vx");
    assert_eq!(
        paths.npm_tool_dir("vite"),
        PathBuf::from("/tmp/test-vx/npm-tools/vite")
    );
}

#[test]
fn test_npm_tool_version_dir() {
    let paths = VxPaths::with_base_dir("/tmp/test-vx");
    assert_eq!(
        paths.npm_tool_version_dir("vite", "5.4.0"),
        PathBuf::from("/tmp/test-vx/npm-tools/vite/5.4.0")
    );
}

// ============================================================================
// Store Path Tests
// ============================================================================

#[test]
fn test_tool_store_dir() {
    let paths = VxPaths::with_base_dir("/tmp/test-vx");
    assert_eq!(
        paths.tool_store_dir("node"),
        PathBuf::from("/tmp/test-vx/store/node")
    );
}
