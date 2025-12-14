//! Common test utilities for vx-cli integration tests

use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::Once;
use tempfile::TempDir;
use vx_plugin::BundleRegistry;

static INIT: Once = Once::new();

/// Initialize test environment (called once per test run)
pub fn init_test_env() {
    INIT.call_once(|| {
        // Set up any global test configuration
        std::env::set_var("VX_TEST_MODE", "1");
    });
}

/// Create a test bundle registry with all tools registered
pub fn create_test_registry() -> BundleRegistry {
    BundleRegistry::new()
}

/// Create a full registry with all available plugins
pub async fn create_full_registry() -> BundleRegistry {
    let registry = BundleRegistry::new();

    // Register all available plugins
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

    registry
}

/// Clean up test environment
pub fn cleanup_test_env() {
    // Clean up any test artifacts
}

/// Test context that manages temporary directories and environment
pub struct TestContext {
    pub temp_dir: TempDir,
    pub home_dir: PathBuf,
    pub project_dir: PathBuf,
}

impl TestContext {
    /// Create a new test context with isolated directories
    pub fn new() -> anyhow::Result<Self> {
        let temp_dir = TempDir::new()?;
        let home_dir = temp_dir.path().join("home");
        let project_dir = temp_dir.path().join("project");

        std::fs::create_dir_all(&home_dir)?;
        std::fs::create_dir_all(&project_dir)?;

        Ok(Self {
            temp_dir,
            home_dir,
            project_dir,
        })
    }

    /// Get the path to the vx binary
    pub fn vx_binary() -> PathBuf {
        // Try to find the vx binary in common locations
        let cargo_target = std::env::var("CARGO_TARGET_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("target"));

        // Check debug build first
        let debug_binary = cargo_target.join("debug").join(Self::binary_name());
        if debug_binary.exists() {
            return debug_binary;
        }

        // Check release build
        let release_binary = cargo_target.join("release").join(Self::binary_name());
        if release_binary.exists() {
            return release_binary;
        }

        // Fall back to system PATH
        PathBuf::from(Self::binary_name())
    }

    /// Get the binary name for the current platform
    fn binary_name() -> &'static str {
        if cfg!(windows) {
            "vx.exe"
        } else {
            "vx"
        }
    }

    /// Run vx command with given arguments
    pub fn run_vx(&self, args: &[&str]) -> std::io::Result<Output> {
        Command::new(Self::vx_binary())
            .args(args)
            .current_dir(&self.project_dir)
            .env("HOME", &self.home_dir)
            .env("USERPROFILE", &self.home_dir)
            .env("VX_HOME", self.home_dir.join(".vx"))
            .output()
    }

    /// Run vx command and assert success
    pub fn run_vx_success(&self, args: &[&str]) -> Output {
        let output = self.run_vx(args).expect("Failed to execute vx");
        if !output.status.success() {
            eprintln!("Command failed: vx {}", args.join(" "));
            eprintln!("stdout: {}", String::from_utf8_lossy(&output.stdout));
            eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
        }
        output
    }

    /// Create a .vx.toml file in the project directory
    pub fn create_vx_toml(&self, content: &str) -> anyhow::Result<()> {
        let path = self.project_dir.join(".vx.toml");
        std::fs::write(path, content)?;
        Ok(())
    }
}

impl Default for TestContext {
    fn default() -> Self {
        Self::new().expect("Failed to create test context")
    }
}

/// Helper to check if a string contains expected substrings
pub fn assert_output_contains(output: &Output, expected: &[&str]) {
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{}\n{}", stdout, stderr);

    for exp in expected {
        assert!(
            combined.contains(exp),
            "Expected output to contain '{}'\nActual output:\n{}",
            exp,
            combined
        );
    }
}

/// Helper to check if output does not contain certain strings
pub fn assert_output_not_contains(output: &Output, unexpected: &[&str]) {
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{}\n{}", stdout, stderr);

    for unexp in unexpected {
        assert!(
            !combined.contains(unexp),
            "Output should not contain '{}'\nActual output:\n{}",
            unexp,
            combined
        );
    }
}

/// Supported tools for testing
/// Note: "rust" is registered as "cargo", not "rust"
pub const SUPPORTED_TOOLS: &[&str] = &["node", "go", "cargo", "uv", "bun"];

/// Get all registered tool names from the registry
pub fn get_registered_tools(registry: &BundleRegistry) -> Vec<String> {
    registry.list_tools()
}
