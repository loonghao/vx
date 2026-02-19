//! Sandbox security tests for vx-starlark
//!
//! Tests for SandboxConfig path/host/command whitelisting and
//! the permissions-based sandbox construction.

use std::path::PathBuf;
use vx_starlark::SandboxConfig;

// ============================================================
// SandboxConfig construction tests
// ============================================================

#[test]
fn test_default_config_enables_fs_and_http() {
    let config = SandboxConfig::default();
    assert!(config.enable_fs);
    assert!(config.enable_http);
    assert!(!config.enable_execute);
}

#[test]
fn test_restrictive_config_disables_all_io() {
    let config = SandboxConfig::restrictive();
    assert!(!config.enable_fs);
    assert!(!config.enable_http);
    assert!(!config.enable_execute);
    assert_eq!(config.memory_limit, 16 * 1024 * 1024); // 16 MB
}

#[test]
fn test_permissive_config_enables_all() {
    let config = SandboxConfig::permissive();
    assert!(config.enable_fs);
    assert!(config.enable_http);
    assert!(config.enable_execute);
    assert_eq!(config.memory_limit, 256 * 1024 * 1024); // 256 MB
}

// ============================================================
// Path whitelist tests
// ============================================================

#[test]
fn test_path_whitelist_allows_listed_paths() {
    let config = SandboxConfig::new()
        .allow_path("/tmp")
        .allow_path("/home/user/.vx");

    assert!(config.is_path_allowed(&PathBuf::from("/tmp")));
    assert!(config.is_path_allowed(&PathBuf::from("/tmp/subdir")));
    assert!(config.is_path_allowed(&PathBuf::from("/tmp/subdir/file.txt")));
    assert!(config.is_path_allowed(&PathBuf::from("/home/user/.vx")));
    assert!(config.is_path_allowed(&PathBuf::from("/home/user/.vx/store")));
}

#[test]
fn test_path_whitelist_blocks_unlisted_paths() {
    let config = SandboxConfig::new()
        .allow_path("/tmp")
        .allow_path("/home/user/.vx");

    assert!(!config.is_path_allowed(&PathBuf::from("/etc")));
    assert!(!config.is_path_allowed(&PathBuf::from("/etc/passwd")));
    assert!(!config.is_path_allowed(&PathBuf::from("/home/user/secrets")));
}

#[test]
fn test_empty_whitelist_allows_all_when_fs_enabled() {
    // When fs is enabled but no whitelist, all paths are allowed
    let config = SandboxConfig::new(); // enable_fs = true, no whitelist
    assert!(config.is_path_allowed(&PathBuf::from("/etc/passwd")));
    assert!(config.is_path_allowed(&PathBuf::from("/tmp/anything")));
}

#[test]
fn test_fs_disabled_blocks_all_paths() {
    let config = SandboxConfig::new().with_fs(false);
    assert!(!config.is_path_allowed(&PathBuf::from("/tmp")));
    assert!(!config.is_path_allowed(&PathBuf::from("/home/user/.vx")));
}

// ============================================================
// HTTP host whitelist tests
// ============================================================

#[test]
fn test_host_whitelist_allows_exact_match() {
    let config = SandboxConfig::new()
        .allow_host("github.com")
        .allow_host("api.github.com");

    assert!(config.is_host_allowed("github.com"));
    assert!(config.is_host_allowed("api.github.com"));
}

#[test]
fn test_host_whitelist_blocks_unlisted_hosts() {
    let config = SandboxConfig::new().allow_host("github.com");

    assert!(!config.is_host_allowed("example.com"));
    assert!(!config.is_host_allowed("evil.com"));
}

#[test]
fn test_host_wildcard_pattern() {
    let config = SandboxConfig::new()
        .allow_host("github.com")
        .allow_host("*.nodejs.org");

    assert!(config.is_host_allowed("github.com"));
    assert!(config.is_host_allowed("nodejs.org"));
    assert!(config.is_host_allowed("dist.nodejs.org"));
    assert!(config.is_host_allowed("releases.nodejs.org"));
    assert!(!config.is_host_allowed("example.com"));
    assert!(!config.is_host_allowed("evil-nodejs.org"));
}

#[test]
fn test_http_disabled_blocks_all_hosts() {
    let config = SandboxConfig::new().with_http(false);
    assert!(!config.is_host_allowed("github.com"));
    assert!(!config.is_host_allowed("example.com"));
}

// ============================================================
// Command whitelist tests
// ============================================================

#[test]
fn test_command_whitelist_allows_listed_commands() {
    let config = SandboxConfig::new()
        .with_execute(true)
        .allow_command("git")
        .allow_command("npm");

    assert!(config.is_command_allowed("git"));
    assert!(config.is_command_allowed("git clone"));
    assert!(config.is_command_allowed("npm install"));
    assert!(config.is_command_allowed("npm run build"));
}

#[test]
fn test_command_whitelist_blocks_unlisted_commands() {
    let config = SandboxConfig::new().with_execute(true).allow_command("git");

    assert!(!config.is_command_allowed("rm"));
    assert!(!config.is_command_allowed("curl"));
    assert!(!config.is_command_allowed("wget"));
}

#[test]
fn test_execute_disabled_blocks_all_commands() {
    let config = SandboxConfig::new()
        .with_execute(false)
        .allow_command("git"); // whitelist doesn't matter when execute is disabled

    assert!(!config.is_command_allowed("git"));
    assert!(!config.is_command_allowed("npm"));
}

#[test]
fn test_empty_command_whitelist_blocks_all_even_when_execute_enabled() {
    let config = SandboxConfig::new().with_execute(true);
    // No commands in whitelist = no commands allowed
    assert!(!config.is_command_allowed("git"));
    assert!(!config.is_command_allowed("ls"));
}

// ============================================================
// Builder pattern tests
// ============================================================

#[test]
fn test_builder_allow_multiple_paths() {
    let paths = vec![PathBuf::from("/tmp"), PathBuf::from("/home/user/.vx")];
    let config = SandboxConfig::new().allow_paths(paths);

    assert!(config.is_path_allowed(&PathBuf::from("/tmp/file")));
    assert!(config.is_path_allowed(&PathBuf::from("/home/user/.vx/store")));
    assert!(!config.is_path_allowed(&PathBuf::from("/etc/passwd")));
}

#[test]
fn test_builder_allow_multiple_hosts() {
    let hosts = vec!["github.com".to_string(), "nodejs.org".to_string()];
    let config = SandboxConfig::new().allow_hosts(hosts);

    assert!(config.is_host_allowed("github.com"));
    assert!(config.is_host_allowed("nodejs.org"));
    assert!(!config.is_host_allowed("example.com"));
}

#[test]
fn test_builder_timeout_and_memory() {
    use std::time::Duration;
    let config = SandboxConfig::new()
        .with_timeout(Duration::from_secs(120))
        .with_memory_limit(128 * 1024 * 1024);

    assert_eq!(config.execution_timeout, Duration::from_secs(120));
    assert_eq!(config.memory_limit, 128 * 1024 * 1024);
}

// ============================================================
// from_permissions tests (Buck2-inspired declarative permissions)
// ============================================================

#[test]
fn test_from_permissions_basic() {
    use vx_starlark::sandbox::PermissionsDecl;

    let perms = PermissionsDecl {
        fs: vec!["/tmp/vx-test".to_string()],
        http: vec!["api.github.com".to_string()],
        exec: vec!["where".to_string()],
    };

    let config = SandboxConfig::from_permissions(&perms).unwrap();

    // FS: only /tmp/vx-test allowed
    assert!(config.is_path_allowed(&PathBuf::from("/tmp/vx-test/file")));
    assert!(!config.is_path_allowed(&PathBuf::from("/etc/passwd")));

    // HTTP: api.github.com + defaults
    assert!(config.is_host_allowed("api.github.com"));

    // Exec: where allowed
    assert!(config.is_command_allowed("where"));
    assert!(!config.is_command_allowed("rm"));
}

#[test]
fn test_from_permissions_empty_keeps_defaults() {
    use vx_starlark::sandbox::PermissionsDecl;

    let perms = PermissionsDecl {
        fs: vec![],
        http: vec![],
        exec: vec![],
    };

    let config = SandboxConfig::from_permissions(&perms).unwrap();

    // Default HTTP hosts should still be present
    assert!(config.is_host_allowed("api.github.com"));
    assert!(config.is_host_allowed("github.com"));
}
