//! Common test utilities for vx-cli

use vx_plugin::BundleRegistry;

/// Create a test bundle registry
pub fn create_test_registry() -> BundleRegistry {
    BundleRegistry::new()
}

/// Clean up test environment
pub fn cleanup_test_env() {
    // Clean up any test artifacts
    // This is called after each test
}
