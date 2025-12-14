//! Common test utilities for vx-cli integration tests

#![allow(dead_code)]

use std::sync::Once;
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

/// Supported tools for testing
/// Note: "rust" is registered as "cargo", not "rust"
pub const SUPPORTED_TOOLS: &[&str] = &["node", "go", "cargo", "uv", "bun"];

/// Get all registered tool names from the registry
#[allow(dead_code)]
pub fn get_registered_tools(registry: &BundleRegistry) -> Vec<String> {
    registry.list_tools()
}
