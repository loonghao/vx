//! Common test utilities for vx-cli E2E tests

#![allow(dead_code)]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::Once;
use vx_plugin::BundleRegistry;

static INIT: Once = Once::new();

// ============================================================================
// Environment Setup
// ============================================================================

/// Initialize test environment (called once per test run)
pub fn init_test_env() {
    INIT.call_once(|| {
        std::env::set_var("VX_TEST_MODE", "1");
    });
}

/// Clean up test environment
pub fn cleanup_test_env() {
    // Clean up any test artifacts
}

// ============================================================================
// Registry Helpers
// ============================================================================

/// Create a test bundle registry with all tools registered
pub fn create_test_registry() -> BundleRegistry {
    BundleRegistry::new()
}

/// Create a full registry with all available plugins
pub async fn create_full_registry() -> BundleRegistry {
    let registry = BundleRegistry::new();

    let _ = registry
        .register_bundle(Box::new(vx_tool_node::NodePlugin::new()))
        .await;
    let _ = registry
        .register_bundle(Box::new(vx_tool_go::GoPlugin::new()))
        .await;
    let _ = registry
        .register_bundle(Box::new(vx_tool_rust::RustPlugin::new()))
        .await;
    let _ = registry
        .register_bundle(Box::new(vx_tool_uv::UvPlugin::new()))
        .await;
    let _ = registry
        .register_bundle(Box::new(vx_tool_bun::BunPlugin::new()))
        .await;
    let _ = registry
        .register_bundle(Box::<vx_tool_pnpm::PnpmPlugin>::default())
        .await;
    let _ = registry
        .register_bundle(Box::<vx_tool_yarn::YarnPlugin>::default())
        .await;

    registry
}

// ============================================================================
// Binary Helpers
// ============================================================================

/// Get the vx binary name for current platform
pub fn binary_name() -> &'static str {
    if cfg!(windows) {
        "vx.exe"
    } else {
        "vx"
    }
}

/// Get the vx binary path
pub fn vx_binary() -> PathBuf {
    // Check VX_BINARY environment variable first (for CI artifact-based testing)
    if let Ok(path) = std::env::var("VX_BINARY") {
        let p = PathBuf::from(&path);
        if p.exists() {
            return p;
        }
    }

    let cargo_target = std::env::var("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("target"));

    // Check release build first (CI uses release)
    let release_binary = cargo_target.join("release").join(binary_name());
    if release_binary.exists() {
        return release_binary;
    }

    // Check debug build
    let debug_binary = cargo_target.join("debug").join(binary_name());
    if debug_binary.exists() {
        return debug_binary;
    }

    // Fall back to system PATH
    PathBuf::from(binary_name())
}

/// Check if vx binary is available
pub fn vx_available() -> bool {
    vx_binary().exists() || Command::new("vx").arg("--version").output().is_ok()
}

// ============================================================================
// Command Execution Helpers
// ============================================================================

/// Run vx with given arguments
pub fn run_vx(args: &[&str]) -> std::io::Result<Output> {
    Command::new(vx_binary()).args(args).output()
}

/// Run vx in a specific directory
pub fn run_vx_in_dir(dir: &Path, args: &[&str]) -> std::io::Result<Output> {
    Command::new(vx_binary())
        .args(args)
        .current_dir(dir)
        .output()
}

/// Run vx with environment variables
pub fn run_vx_with_env(args: &[&str], env: &[(&str, &str)]) -> std::io::Result<Output> {
    let mut cmd = Command::new(vx_binary());
    cmd.args(args);
    for (key, value) in env {
        cmd.env(key, value);
    }
    cmd.output()
}

// ============================================================================
// Output Helpers
// ============================================================================

/// Check if output indicates success
pub fn is_success(output: &Output) -> bool {
    output.status.success()
}

/// Get stdout as string
pub fn stdout_str(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).to_string()
}

/// Get stderr as string
pub fn stderr_str(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).to_string()
}

/// Combined output for debugging
pub fn combined_output(output: &Output) -> String {
    format!(
        "stdout:\n{}\nstderr:\n{}",
        stdout_str(output),
        stderr_str(output)
    )
}

/// Get exit code
pub fn exit_code(output: &Output) -> Option<i32> {
    output.status.code()
}

// ============================================================================
// Assertion Helpers
// ============================================================================

/// Assert command succeeded
pub fn assert_success(output: &Output, context: &str) {
    assert!(
        is_success(output),
        "{} should succeed: {}",
        context,
        combined_output(output)
    );
}

/// Assert command failed
pub fn assert_failure(output: &Output, context: &str) {
    assert!(
        !is_success(output),
        "{} should fail: {}",
        context,
        combined_output(output)
    );
}

/// Assert stdout contains text
pub fn assert_stdout_contains(output: &Output, text: &str, context: &str) {
    let stdout = stdout_str(output);
    assert!(
        stdout.contains(text),
        "{}: stdout should contain '{}'\nActual: {}",
        context,
        text,
        combined_output(output)
    );
}

/// Assert stderr contains text
pub fn assert_stderr_contains(output: &Output, text: &str, context: &str) {
    let stderr = stderr_str(output);
    assert!(
        stderr.contains(text),
        "{}: stderr should contain '{}'\nActual: {}",
        context,
        text,
        combined_output(output)
    );
}

/// Assert output (stdout or stderr) contains text
pub fn assert_output_contains(output: &Output, text: &str, context: &str) {
    let combined = combined_output(output);
    assert!(
        combined.contains(text),
        "{}: output should contain '{}'\nActual: {}",
        context,
        text,
        combined
    );
}

// ============================================================================
// Skip Helpers
// ============================================================================

/// Skip test if vx is not available
#[macro_export]
macro_rules! skip_if_no_vx {
    () => {
        if !$crate::common::vx_available() {
            eprintln!("Skipping: vx binary not found");
            return;
        }
    };
}

/// Skip test if tool is not installed (check via vx which)
pub fn tool_installed(tool: &str) -> bool {
    if !vx_available() {
        return false;
    }
    run_vx(&["which", tool])
        .map(|o| is_success(&o))
        .unwrap_or(false)
}

// ============================================================================
// Constants
// ============================================================================

/// Supported tools for testing (tools registered via VxTool, not package managers)
pub const SUPPORTED_TOOLS: &[&str] = &["node", "go", "cargo", "uv", "bun"];

/// Supported package managers for testing
pub const SUPPORTED_PACKAGE_MANAGERS: &[&str] = &["npm", "pnpm", "yarn"];

/// Get all registered tool names from the registry
pub fn get_registered_tools(registry: &BundleRegistry) -> Vec<String> {
    registry.list_tools()
}
