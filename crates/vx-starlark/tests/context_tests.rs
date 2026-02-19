//! ProviderContext tests for vx-starlark

use std::path::PathBuf;
use vx_starlark::context::{InstallResult, PathManager, PlatformInfo, VersionInfo};
use vx_starlark::{ProviderContext, SandboxConfig};

// ============================================================
// PlatformInfo tests
// ============================================================

#[test]
fn test_platform_info_current_is_non_empty() {
    let platform = PlatformInfo::current();
    assert!(!platform.os.is_empty(), "os should not be empty");
    assert!(!platform.arch.is_empty(), "arch should not be empty");
}

#[test]
fn test_platform_info_os_is_known_value() {
    let platform = PlatformInfo::current();
    assert!(
        ["windows", "macos", "linux", "unknown"].contains(&platform.os.as_str()),
        "os '{}' is not a known value",
        platform.os
    );
}

#[test]
fn test_platform_info_arch_is_known_value() {
    let platform = PlatformInfo::current();
    assert!(
        ["x64", "arm64", "x86", "unknown"].contains(&platform.arch.as_str()),
        "arch '{}' is not a known value",
        platform.arch
    );
}

// ============================================================
// VersionInfo tests
// ============================================================

#[test]
fn test_version_info_builder() {
    let v = VersionInfo::new("20.0.0")
        .with_lts(true)
        .with_stable(true)
        .with_date("2024-01-01");

    assert_eq!(v.version, "20.0.0");
    assert!(v.lts);
    assert!(v.stable);
    assert_eq!(v.date, Some("2024-01-01".to_string()));
}

#[test]
fn test_version_info_defaults() {
    let v = VersionInfo::new("1.0.0");
    assert_eq!(v.version, "1.0.0");
    assert!(!v.lts);
    assert!(v.stable); // stable by default
    assert!(v.date.is_none());
}

// ============================================================
// InstallResult tests
// ============================================================

#[test]
fn test_install_result_success() {
    let result = InstallResult::success(PathBuf::from("/tmp/vx/store/node/20.0.0"));
    assert!(result.success);
    assert_eq!(
        result.install_path,
        PathBuf::from("/tmp/vx/store/node/20.0.0")
    );
    assert!(result.message.is_none());
}

#[test]
fn test_install_result_failure() {
    let result = InstallResult::failure("Download failed: connection timeout");
    assert!(!result.success);
    assert_eq!(
        result.message,
        Some("Download failed: connection timeout".to_string())
    );
}

#[test]
fn test_install_result_with_executable() {
    let result = InstallResult::success(PathBuf::from("/tmp/vx/store/node/20.0.0"))
        .with_executable(PathBuf::from("/tmp/vx/store/node/20.0.0/bin/node"))
        .with_message("Installed successfully");

    assert!(result.success);
    assert_eq!(
        result.executable_path,
        Some(PathBuf::from("/tmp/vx/store/node/20.0.0/bin/node"))
    );
    assert_eq!(result.message, Some("Installed successfully".to_string()));
}

// ============================================================
// PathManager tests
// ============================================================

#[test]
fn test_path_manager_install_dir() {
    let pm = PathManager::new("node", PathBuf::from("/tmp/vx"));
    assert_eq!(
        pm.install_dir("20.0.0"),
        PathBuf::from("/tmp/vx/store/node/20.0.0")
    );
    assert_eq!(
        pm.install_dir("18.0.0"),
        PathBuf::from("/tmp/vx/store/node/18.0.0")
    );
}

#[test]
fn test_path_manager_download_cache() {
    let pm = PathManager::new("node", PathBuf::from("/tmp/vx"));
    assert_eq!(
        pm.download_cache(),
        PathBuf::from("/tmp/vx/cache/downloads")
    );
}

#[test]
fn test_path_manager_provider_temp() {
    let pm = PathManager::new("msvc", PathBuf::from("/tmp/vx"));
    assert_eq!(pm.provider_temp(), PathBuf::from("/tmp/vx/tmp/msvc"));
}

#[test]
fn test_path_manager_current_install_dir_none_without_version() {
    let pm = PathManager::new("node", PathBuf::from("/tmp/vx"));
    assert!(pm.current_install_dir().is_none());
}

#[test]
fn test_path_manager_current_install_dir_with_version() {
    let pm = PathManager::new("node", PathBuf::from("/tmp/vx")).with_version("20.0.0");
    assert_eq!(
        pm.current_install_dir(),
        Some(PathBuf::from("/tmp/vx/store/node/20.0.0"))
    );
}

// ============================================================
// ProviderContext sandbox enforcement tests
// ============================================================

#[test]
fn test_context_sandbox_blocks_unauthorized_path() {
    let ctx = ProviderContext::new("test", PathBuf::from("/tmp/vx"))
        .with_sandbox(SandboxConfig::restrictive());

    // restrictive() disables fs entirely
    let path = PathBuf::from("/etc/passwd");
    assert!(ctx.file_exists(&path).is_err());
}

#[test]
fn test_context_sandbox_allows_whitelisted_path() {
    let sandbox = SandboxConfig::new().allow_path("/tmp/vx-test");
    let ctx = ProviderContext::new("test", PathBuf::from("/tmp/vx")).with_sandbox(sandbox);

    // Path within whitelist should not return access-denied error
    // (it may return Ok(false) if the file doesn't exist, but not an access error)
    let path = PathBuf::from("/tmp/vx-test/some-file.txt");
    let result = ctx.file_exists(&path);
    assert!(result.is_ok(), "Whitelisted path should not be blocked");
}

#[test]
fn test_context_path_join() {
    let ctx = ProviderContext::new("test", PathBuf::from("/tmp/vx"));
    let base = PathBuf::from("/tmp/vx/store");
    let joined = ctx.path_join(&base, "node");
    assert_eq!(joined, PathBuf::from("/tmp/vx/store/node"));
}

#[test]
fn test_context_path_filename() {
    let ctx = ProviderContext::new("test", PathBuf::from("/tmp/vx"));
    let path = PathBuf::from("/tmp/vx/store/node/20.0.0/bin/node");
    assert_eq!(ctx.path_filename(&path), Some("node".to_string()));
}

#[test]
fn test_context_path_extension() {
    let ctx = ProviderContext::new("test", PathBuf::from("/tmp/vx"));
    let path = PathBuf::from("/tmp/downloads/node-v20.0.0.tar.gz");
    // Note: extension() returns "gz" for .tar.gz
    assert_eq!(ctx.path_extension(&path), Some("gz".to_string()));
}

#[test]
fn test_context_string_utilities() {
    let ctx = ProviderContext::new("test", PathBuf::from("/tmp/vx"));

    // join_strings
    let parts = vec!["a".to_string(), "b".to_string(), "c".to_string()];
    assert_eq!(ctx.join_strings(&parts, ";"), "a;b;c");

    // split_string
    let parts = ctx.split_string("a;b;c", ";");
    assert_eq!(parts, vec!["a", "b", "c"]);
}

#[test]
fn test_context_matches_glob() {
    let ctx = ProviderContext::new("test", PathBuf::from("/tmp/vx"));

    // Exact match
    assert!(ctx.matches("node", "node"));
    assert!(!ctx.matches("node", "npm"));

    // Wildcard match
    assert!(ctx.matches("node-v20.0.0-linux-x64.tar.gz", "node-*-linux-x64.tar.gz"));
    assert!(!ctx.matches("node-v20.0.0-darwin-x64.tar.gz", "node-*-linux-x64.tar.gz"));
}

#[test]
fn test_context_builder_methods() {
    use std::collections::HashMap;

    let mut env = HashMap::new();
    env.insert("PATH".to_string(), "/usr/bin".to_string());

    let ctx = ProviderContext::new("test", PathBuf::from("/tmp/vx"))
        .with_env(env)
        .with_dry_run(true)
        .with_verbose(true)
        .with_version("20.0.0");

    assert!(ctx.dry_run);
    assert!(ctx.verbose);
    assert_eq!(ctx.env.get("PATH"), Some(&"/usr/bin".to_string()));
    assert_eq!(
        ctx.paths.current_install_dir(),
        Some(PathBuf::from("/tmp/vx/store/test/20.0.0"))
    );
}
