//! Bun runtime tests

use rstest::rstest;
use tempfile::TempDir;
use vx_provider_bun::{BunProvider, BunRuntime, BunxRuntime};
use vx_runtime::{Ecosystem, Platform, Provider, Runtime, Shim};

#[rstest]
fn test_bun_runtime_name() {
    let runtime = BunRuntime::new();
    assert_eq!(runtime.name(), "bun");
}

#[rstest]
fn test_bun_runtime_ecosystem() {
    let runtime = BunRuntime::new();
    assert_eq!(runtime.ecosystem(), Ecosystem::NodeJs);
}

#[rstest]
fn test_bun_runtime_description() {
    let runtime = BunRuntime::new();
    assert!(runtime.description().contains("JavaScript runtime"));
}

#[rstest]
fn test_bun_runtime_aliases() {
    let runtime = BunRuntime::new();
    // bunx is now handled as a separate RuntimeSpec, not an alias
    assert_eq!(runtime.aliases().len(), 0);
}

#[rstest]
fn test_bun_provider_name() {
    let provider = BunProvider::new();
    assert_eq!(provider.name(), "bun");
}

#[rstest]
fn test_bun_provider_runtimes() {
    let provider = BunProvider::new();
    let runtimes = provider.runtimes();
    // bun and bunx are both defined in provider.toml
    assert_eq!(runtimes.len(), 2);

    let names: Vec<&str> = runtimes.iter().map(|r| r.name()).collect();
    assert!(names.contains(&"bun"));
    assert!(names.contains(&"bunx"));
}

#[rstest]
fn test_bun_provider_supports() {
    let provider = BunProvider::new();
    assert!(provider.supports("bun"));
    // bunx should be supported through alias resolution in the resolver layer
    assert!(!provider.supports("npm"));
}

// ============================================================================
// Bunx Runtime Tests
// ============================================================================

#[rstest]
fn test_bunx_runtime_name() {
    let runtime = BunxRuntime::new();
    assert_eq!(runtime.name(), "bunx");
}

#[rstest]
fn test_bunx_runtime_ecosystem() {
    let runtime = BunxRuntime::new();
    assert_eq!(runtime.ecosystem(), Ecosystem::NodeJs);
}

#[rstest]
fn test_bunx_runtime_description() {
    let runtime = BunxRuntime::new();
    assert!(runtime.description().contains("package runner"));
}

#[rstest]
fn test_bunx_runtime_store_name() {
    let runtime = BunxRuntime::new();
    // bunx is bundled with bun, so it should store under "bun"
    assert_eq!(runtime.store_name(), "bun");
}

#[rstest]
fn test_bunx_runtime_executable_name() {
    let runtime = BunxRuntime::new();
    // bunx uses the bun executable
    assert_eq!(runtime.executable_name(), "bun");
}

#[rstest]
fn test_bunx_runtime_metadata() {
    let runtime = BunxRuntime::new();
    let meta = runtime.metadata();
    assert_eq!(meta.get("bundled_with"), Some(&"bun".to_string()));
    assert_eq!(meta.get("homepage"), Some(&"https://bun.sh/".to_string()));
}

// ============================================================================
// Shim Creation Tests (using vx_runtime::Shim)
// ============================================================================

#[rstest]
fn test_shim_creates_bunx_wrapper() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let platform = Platform::current();

    // Create a dummy bun executable
    let bun_exe = temp_dir
        .path()
        .join(if cfg!(windows) { "bun.exe" } else { "bun" });
    std::fs::write(&bun_exe, b"dummy").expect("Failed to create bun executable");

    // Create bunx shim using the Shim API
    let shim_path = Shim::new("bunx", &bun_exe)
        .with_args(&["x"])
        .create(temp_dir.path(), &platform)
        .expect("Failed to create shim");

    // Verify shim was created with correct name
    let expected_name = if cfg!(windows) { "bunx.cmd" } else { "bunx" };
    assert!(
        shim_path.ends_with(expected_name),
        "Shim should be named {}",
        expected_name
    );
    assert!(shim_path.exists(), "Shim file should exist");

    // Verify shim content
    let content = std::fs::read_to_string(&shim_path).expect("Failed to read shim");

    #[cfg(windows)]
    {
        assert!(content.contains("@echo off"));
        assert!(content.contains(" x %*"));
    }

    #[cfg(not(windows))]
    {
        assert!(content.contains("#!/bin/sh"));
        assert!(content.contains("exec"));
        assert!(content.contains("\" x \"$@\""));

        // Verify executable permissions
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::metadata(&shim_path)
            .expect("Failed to get metadata")
            .permissions();
        assert!(perms.mode() & 0o111 != 0, "Shim should be executable");
    }
}

/// Helper to get the expected archive directory name for the current platform
fn get_expected_archive_dir() -> &'static str {
    let platform = Platform::current();
    match (&platform.os, &platform.arch) {
        (vx_runtime::Os::Windows, vx_runtime::Arch::X86_64) => "bun-windows-x64",
        (vx_runtime::Os::MacOS, vx_runtime::Arch::X86_64) => "bun-darwin-x64",
        (vx_runtime::Os::MacOS, vx_runtime::Arch::Aarch64) => "bun-darwin-aarch64",
        (vx_runtime::Os::Linux, vx_runtime::Arch::X86_64) => "bun-linux-x64",
        (vx_runtime::Os::Linux, vx_runtime::Arch::Aarch64) => "bun-linux-aarch64",
        _ => "bun-linux-x64",
    }
}

#[rstest]
fn test_bun_post_extract_creates_bunx_shim() {
    let runtime = BunRuntime::new();
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Create the platform-specific directory structure
    let _platform = Platform::current();
    let dir_name = get_expected_archive_dir();
    let bun_dir = temp_dir.path().join(dir_name);
    std::fs::create_dir_all(&bun_dir).expect("Failed to create bun dir");

    // Create a dummy bun executable
    let bun_exe = bun_dir.join(if cfg!(windows) { "bun.exe" } else { "bun" });
    std::fs::write(&bun_exe, b"dummy").expect("Failed to create bun executable");

    // Call post_extract
    let install_path = temp_dir.path().to_path_buf();
    let result = runtime.post_extract("1.0.0", &install_path);
    assert!(result.is_ok());

    // Verify bunx shim was created
    let bunx_shim = bun_dir.join(if cfg!(windows) { "bunx.cmd" } else { "bunx" });
    assert!(bunx_shim.exists(), "bunx shim should exist");
}

#[rstest]
fn test_bunx_executable_relative_path_same_as_bun() {
    let bun_runtime = BunRuntime::new();
    let bunx_runtime = BunxRuntime::new();
    let _platform = Platform::current();

    let bun_path = bun_runtime.executable_relative_path("1.0.0", &_platform);
    let bunx_path = bunx_runtime.executable_relative_path("1.0.0", &_platform);

    // bunx should use the same executable path as bun
    assert_eq!(bun_path, bunx_path);
}
