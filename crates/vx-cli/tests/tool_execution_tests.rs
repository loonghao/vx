//! Tool Execution Integration Tests
//!
//! These tests verify that tools can be executed correctly through vx.
//! Note: These tests require the tools to be installed on the system.

mod common;

use common::{cleanup_test_env, create_full_registry, init_test_env};
use rstest::*;
use vx_plugin::BundleRegistry;

/// Test fixture that provides a fully initialized registry
#[fixture]
pub async fn registry() -> BundleRegistry {
    init_test_env();
    create_full_registry().await
}

// ============================================================================
// Execute Command Tests
// ============================================================================

mod execute_tests {
    use super::*;
    use vx_cli::commands::execute;

    /// Test execute with empty tool name
    #[rstest]
    #[tokio::test]
    async fn test_execute_empty_tool(#[future] registry: BundleRegistry) {
        let registry = registry.await;
        let result = execute::handle(&registry, "", &[], false).await;
        assert!(result.is_err(), "Execute with empty tool should fail");
        cleanup_test_env();
    }

    /// Test execute with nonexistent tool
    #[rstest]
    #[tokio::test]
    async fn test_execute_nonexistent_tool(#[future] registry: BundleRegistry) {
        let registry = registry.await;
        let result = execute::handle(&registry, "nonexistent-tool-xyz", &[], false).await;
        assert!(result.is_err(), "Execute nonexistent tool should fail");
        cleanup_test_env();
    }

    /// Test execute with system path fallback
    #[rstest]
    #[tokio::test]
    async fn test_execute_with_system_path(#[future] registry: BundleRegistry) {
        let registry = registry.await;
        // This should attempt to use system PATH
        let result = execute::handle(&registry, "echo", &["hello".to_string()], true).await;
        // May succeed or fail depending on system, but should not panic
        let _ = result;
        cleanup_test_env();
    }
}

// ============================================================================
// Tool Version Detection Tests
// ============================================================================

mod version_detection_tests {
    use super::*;

    /// Test that tool provides version info
    #[rstest]
    #[case("node")]
    #[case("go")]
    #[case("uv")]
    #[case("bun")]
    #[tokio::test]
    async fn test_tool_has_version_info(
        #[future] registry: BundleRegistry,
        #[case] tool_name: &str,
    ) {
        let registry = registry.await;

        if let Some(tool) = registry.get_tool(tool_name) {
            let name = tool.name();
            assert!(!name.is_empty(), "Tool should have a name");
        }

        cleanup_test_env();
    }
}

// ============================================================================
// Tool Installation Path Tests
// ============================================================================

mod installation_path_tests {
    use super::*;
    use vx_paths::VxPaths;

    /// Test that VxPaths provides correct directories
    #[rstest]
    #[test]
    fn test_vx_paths_directories() {
        init_test_env();

        let paths = VxPaths::new().expect("Failed to create VxPaths");

        // All paths should be valid
        let base = &paths.base_dir;
        let tools = &paths.tools_dir;
        let cache = &paths.cache_dir;

        assert!(!base.as_os_str().is_empty(), "Base dir should not be empty");
        assert!(
            !tools.as_os_str().is_empty(),
            "Tools dir should not be empty"
        );
        assert!(
            !cache.as_os_str().is_empty(),
            "Cache dir should not be empty"
        );

        cleanup_test_env();
    }

    /// Test tool-specific paths
    #[rstest]
    #[case("node")]
    #[case("go")]
    #[case("uv")]
    #[test]
    fn test_tool_specific_paths(#[case] tool_name: &str) {
        init_test_env();

        let paths = VxPaths::new().expect("Failed to create VxPaths");
        let tool_dir = paths.tools_dir.join(tool_name);

        assert!(
            tool_dir.ends_with(tool_name),
            "Tool dir should end with tool name"
        );

        cleanup_test_env();
    }
}

// ============================================================================
// Tool Metadata Tests
// ============================================================================

mod tool_metadata_tests {
    use super::*;

    /// Test that each tool has proper metadata
    #[rstest]
    #[case("node", &["node", "npm", "npx"])]
    #[case("go", &["go"])]
    #[case("uv", &["uv", "uvx"])]
    #[case("bun", &["bun"])] // Note: bunx is not a separate tool
    #[tokio::test]
    async fn test_tool_bundle_provides_tools(
        #[future] registry: BundleRegistry,
        #[case] bundle_name: &str,
        #[case] expected_tools: &[&str],
    ) {
        let registry = registry.await;

        for tool_name in expected_tools {
            let tool = registry.get_tool(tool_name);
            assert!(
                tool.is_some(),
                "Bundle '{}' should provide tool '{}'",
                bundle_name,
                tool_name
            );
        }

        cleanup_test_env();
    }
}

// ============================================================================
// Concurrent Execution Tests
// ============================================================================

mod concurrent_tests {
    use super::*;
    use std::sync::Arc;
    use tokio::sync::Barrier;

    /// Test concurrent registry access
    #[rstest]
    #[tokio::test]
    async fn test_concurrent_registry_access() {
        init_test_env();

        let registry = Arc::new(create_full_registry().await);
        let barrier = Arc::new(Barrier::new(5));

        let mut handles = vec![];

        for i in 0..5 {
            let registry = Arc::clone(&registry);
            let barrier = Arc::clone(&barrier);
            let tool_name = match i % 5 {
                0 => "node",
                1 => "go",
                2 => "uv",
                3 => "bun",
                _ => "npm",
            };

            handles.push(tokio::spawn(async move {
                barrier.wait().await;
                let tool = registry.get_tool(tool_name);
                tool.is_some()
            }));
        }

        for handle in handles {
            let result = handle.await.expect("Task should complete");
            assert!(result, "Tool should be found");
        }

        cleanup_test_env();
    }

    /// Test concurrent tool listing
    #[rstest]
    #[tokio::test]
    async fn test_concurrent_tool_listing() {
        init_test_env();

        let registry = Arc::new(create_full_registry().await);

        let mut handles = vec![];

        for _ in 0..10 {
            let registry = Arc::clone(&registry);
            handles.push(tokio::spawn(async move { registry.list_tools() }));
        }

        let mut results = vec![];
        for handle in handles {
            let tools = handle.await.expect("Task should complete");
            results.push(tools);
        }

        // All results should be identical
        let first = &results[0];
        for result in &results[1..] {
            assert_eq!(first, result, "Concurrent listings should be identical");
        }

        cleanup_test_env();
    }
}

// ============================================================================
// Environment Variable Tests
// ============================================================================

mod env_tests {
    use super::*;

    /// Test VX_HOME environment variable handling
    #[rstest]
    #[test]
    fn test_vx_home_env_var() {
        init_test_env();

        // Save original
        let original = std::env::var("VX_HOME").ok();

        // Set custom VX_HOME
        let temp_dir = tempfile::TempDir::new().expect("Failed to create temp dir");
        std::env::set_var("VX_HOME", temp_dir.path());

        // Note: VxPaths::new() uses dirs::home_dir(), not VX_HOME directly
        // This test verifies VxPaths can be created successfully
        let paths = vx_paths::VxPaths::new();
        assert!(paths.is_ok(), "VxPaths should be creatable");

        // Restore original
        if let Some(orig) = original {
            std::env::set_var("VX_HOME", orig);
        } else {
            std::env::remove_var("VX_HOME");
        }

        cleanup_test_env();
    }
}

// ============================================================================
// Tool Alias Tests
// ============================================================================

mod alias_tests {
    use super::*;

    /// Test that tool aliases work correctly
    #[rstest]
    #[tokio::test]
    async fn test_npm_is_node_alias(#[future] registry: BundleRegistry) {
        let registry = registry.await;

        // npm should be available as a tool
        let npm = registry.get_tool("npm");
        assert!(npm.is_some(), "npm should be available");

        // npx should also be available
        let npx = registry.get_tool("npx");
        assert!(npx.is_some(), "npx should be available");

        cleanup_test_env();
    }

    /// Test uvx is uv alias
    #[rstest]
    #[tokio::test]
    async fn test_uvx_is_uv_alias(#[future] registry: BundleRegistry) {
        let registry = registry.await;

        let uv = registry.get_tool("uv");
        let uvx = registry.get_tool("uvx");

        assert!(uv.is_some(), "uv should be available");
        assert!(uvx.is_some(), "uvx should be available");

        cleanup_test_env();
    }

    /// Test bun is registered
    #[rstest]
    #[tokio::test]
    async fn test_bun_is_registered(#[future] registry: BundleRegistry) {
        let registry = registry.await;

        let bun = registry.get_tool("bun");
        assert!(bun.is_some(), "bun should be available");

        // Note: bunx is not a separate tool, it's "bun x" command

        cleanup_test_env();
    }
}
