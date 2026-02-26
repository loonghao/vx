//! E2E tests for build cache providers
//!
//! These tests verify that build cache tools are correctly registered
//! and can be loaded. Manifest-driven providers (nx, turbo, sccache, etc.)
//! are tested via local provider loading since they are not in ProviderRegistry.

use std::env;
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

/// Get the path to the vx binary for testing
fn vx_binary() -> PathBuf {
    let mut path = env::current_exe().unwrap();
    path.pop(); // Remove test binary name
    if path.ends_with("deps") {
        path.pop(); // Remove deps directory
    }
    path.push("vx");
    if cfg!(windows) {
        path.set_extension("exe");
    }
    path
}

/// E2E test environment with isolated VX_HOME
struct E2ETestEnv {
    home: TempDir,
}

impl E2ETestEnv {
    fn new() -> Self {
        Self {
            home: TempDir::new().expect("Failed to create temp dir"),
        }
    }

    fn run(&self, args: &[&str]) -> std::process::Output {
        Command::new(vx_binary())
            .args(args)
            .env("VX_HOME", self.home.path())
            .output()
            .expect("Failed to execute vx command")
    }

    #[allow(dead_code)]
    fn run_success(&self, args: &[&str]) -> String {
        let output = self.run(args);
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        if !output.status.success() {
            panic!(
                "Command failed: vx {}\nstdout: {}\nstderr: {}",
                args.join(" "),
                stdout,
                stderr
            );
        }
        stdout
    }
}

/// Get the path to the vx-providers directory
fn providers_dir() -> PathBuf {
    let mut path = env::current_exe().unwrap();
    path.pop(); // Remove test binary name
    if path.ends_with("deps") {
        path.pop(); // Remove deps directory
    }
    // Navigate to project root
    path.pop(); // Remove debug/release
    path.pop(); // Remove target
    path.push("crates");
    path.push("vx-providers");
    path
}

// ============================================================================
// Local Provider Tests - verify provider.star files are valid
// ============================================================================

#[test]
fn test_local_provider_nx() {
    let nx_dir = providers_dir().join("nx");
    assert!(nx_dir.exists(), "nx provider directory should exist");
    assert!(
        nx_dir.join("provider.star").exists(),
        "provider.star should exist"
    );

    let output = Command::new(vx_binary())
        .args(["test", "--local"])
        .arg(&nx_dir)
        .output()
        .expect("Failed to execute vx test --local");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success(),
        "nx provider test should pass: {}",
        stdout
    );
    assert!(
        stdout.contains("nx"),
        "Output should mention nx: {}",
        stdout
    );
}

#[test]
fn test_local_provider_turbo() {
    let turbo_dir = providers_dir().join("turbo");
    assert!(turbo_dir.exists(), "turbo provider directory should exist");
    assert!(
        turbo_dir.join("provider.star").exists(),
        "provider.star should exist"
    );

    let output = Command::new(vx_binary())
        .args(["test", "--local"])
        .arg(&turbo_dir)
        .output()
        .expect("Failed to execute vx test --local");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success(),
        "turbo provider test should pass: {}",
        stdout
    );
    assert!(
        stdout.contains("turbo"),
        "Output should mention turbo: {}",
        stdout
    );
}

#[test]
fn test_local_provider_sccache() {
    let sccache_dir = providers_dir().join("sccache");
    assert!(
        sccache_dir.exists(),
        "sccache provider directory should exist"
    );
    assert!(
        sccache_dir.join("provider.star").exists(),
        "provider.star should exist"
    );

    let output = Command::new(vx_binary())
        .args(["test", "--local"])
        .arg(&sccache_dir)
        .output()
        .expect("Failed to execute vx test --local");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success(),
        "sccache provider test should pass: {}",
        stdout
    );
    assert!(
        stdout.contains("sccache"),
        "Output should mention sccache: {}",
        stdout
    );
}

#[test]
fn test_local_provider_ccache() {
    let ccache_dir = providers_dir().join("ccache");
    if !ccache_dir.exists() {
        // ccache might not be implemented yet
        return;
    }
    assert!(
        ccache_dir.join("provider.star").exists(),
        "provider.star should exist"
    );

    let output = Command::new(vx_binary())
        .args(["test", "--local"])
        .arg(&ccache_dir)
        .output()
        .expect("Failed to execute vx test --local");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success(),
        "ccache provider test should pass: {}",
        stdout
    );
}

#[test]
fn test_local_provider_buildcache() {
    let buildcache_dir = providers_dir().join("buildcache");
    assert!(
        buildcache_dir.exists(),
        "buildcache provider directory should exist"
    );
    assert!(
        buildcache_dir.join("provider.star").exists(),
        "provider.star should exist"
    );

    let output = Command::new(vx_binary())
        .args(["test", "--local"])
        .arg(&buildcache_dir)
        .output()
        .expect("Failed to execute vx test --local");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success(),
        "buildcache provider test should pass: {}",
        stdout
    );
    assert!(
        stdout.contains("buildcache"),
        "Output should mention buildcache: {}",
        stdout
    );
}

// ============================================================================
// Plugin Registration Tests - verify Rust providers are in ProviderRegistry
// ============================================================================

#[test]
fn test_plugin_list_includes_node() {
    let env = E2ETestEnv::new();
    let output = env.run(&["plugin", "list"]);

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.to_lowercase().contains("node"),
        "Expected 'node' in plugin list: {}",
        stdout
    );
}

#[test]
fn test_plugin_list_includes_go() {
    let env = E2ETestEnv::new();
    let output = env.run(&["plugin", "list"]);

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.to_lowercase().contains("go"),
        "Expected 'go' in plugin list: {}",
        stdout
    );
}

// ============================================================================
// Provider Files Existence Tests
// ============================================================================

#[test]
fn test_nx_provider_files_exist() {
    let nx_dir = providers_dir().join("nx");
    assert!(nx_dir.exists(), "nx provider directory should exist");
    assert!(
        nx_dir.join("provider.star").exists(),
        "provider.star should exist"
    );
    assert!(
        nx_dir.join("Cargo.toml").exists(),
        "Cargo.toml should exist"
    );
    assert!(nx_dir.join("lib.rs").exists(), "lib.rs should exist");
}

#[test]
fn test_turbo_provider_files_exist() {
    let turbo_dir = providers_dir().join("turbo");
    assert!(turbo_dir.exists(), "turbo provider directory should exist");
    assert!(
        turbo_dir.join("provider.star").exists(),
        "provider.star should exist"
    );
    assert!(
        turbo_dir.join("Cargo.toml").exists(),
        "Cargo.toml should exist"
    );
    assert!(turbo_dir.join("lib.rs").exists(), "lib.rs should exist");
}

#[test]
fn test_sccache_provider_files_exist() {
    let sccache_dir = providers_dir().join("sccache");
    assert!(
        sccache_dir.exists(),
        "sccache provider directory should exist"
    );
    assert!(
        sccache_dir.join("provider.star").exists(),
        "provider.star should exist"
    );
    assert!(
        sccache_dir.join("Cargo.toml").exists(),
        "Cargo.toml should exist"
    );
    assert!(sccache_dir.join("lib.rs").exists(), "lib.rs should exist");
}

#[test]
fn test_buildcache_provider_files_exist() {
    let buildcache_dir = providers_dir().join("buildcache");
    assert!(
        buildcache_dir.exists(),
        "buildcache provider directory should exist"
    );
    assert!(
        buildcache_dir.join("provider.star").exists(),
        "provider.star should exist"
    );
    assert!(
        buildcache_dir.join("Cargo.toml").exists(),
        "Cargo.toml should exist"
    );
    assert!(
        buildcache_dir.join("lib.rs").exists(),
        "lib.rs should exist"
    );
}

// ============================================================================
// Help Tests - verify help text is available
// ============================================================================

#[test]
fn test_install_help() {
    let env = E2ETestEnv::new();
    let output = env.run(&["install", "--help"]);

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("install") || stdout.contains("Install"));
    assert!(stdout.contains("TOOL") || stdout.contains("tool"));
}
