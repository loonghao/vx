//! CLI Integration Tests for vx
//!
//! These tests verify the CLI commands work correctly end-to-end.
//! They test the actual CLI behavior using the registry and command handlers.

mod common;

use common::{
    SUPPORTED_TOOLS, cleanup_test_env, create_full_registry, create_test_context, init_test_env,
};
use rstest::*;
use vx_runtime::ProviderRegistry;

/// Test fixture that provides a fully initialized registry
#[fixture]
pub async fn registry() -> ProviderRegistry {
    init_test_env();
    create_full_registry().await
}

// ============================================================================
// Version Command Tests
// ============================================================================

mod version_tests {
    use super::*;
    use vx_cli::commands::version;

    #[rstest]
    #[tokio::test]
    async fn test_version_command_executes() {
        init_test_env();
        let result = version::handle().await;
        assert!(result.is_ok(), "Version command should succeed");
        cleanup_test_env();
    }
}

// ============================================================================
// List Command Tests
// ============================================================================

mod list_tests {
    use super::*;
    use vx_cli::commands::list;

    #[rstest]
    #[tokio::test]
    async fn test_list_all_tools(#[future] registry: ProviderRegistry) {
        let registry = registry.await;
        let ctx = create_test_context();
        let result = list::handle_list(
            &registry,
            &ctx,
            None,
            false,
            false,
            false,
            vx_cli::OutputFormat::Text,
        )
        .await;
        assert!(result.is_ok(), "List command should succeed");
        cleanup_test_env();
    }

    #[rstest]
    #[tokio::test]
    async fn test_list_with_status(#[future] registry: ProviderRegistry) {
        let registry = registry.await;
        let ctx = create_test_context();
        let result = list::handle_list(
            &registry,
            &ctx,
            None,
            true,
            false,
            false,
            vx_cli::OutputFormat::Text,
        )
        .await;
        assert!(result.is_ok(), "List with status should succeed");
        cleanup_test_env();
    }

    #[rstest]
    #[case("node")]
    #[case("go")]
    #[case("rustc")]
    #[case("uv")]
    #[case("bun")]
    #[tokio::test]
    async fn test_list_specific_tool(
        #[future] registry: ProviderRegistry,
        #[case] tool_name: &str,
    ) {
        let registry = registry.await;
        let ctx = create_test_context();
        let result = list::handle_list(
            &registry,
            &ctx,
            Some(tool_name),
            false,
            false,
            false,
            vx_cli::OutputFormat::Text,
        )
        .await;
        assert!(
            result.is_ok(),
            "List for {} should succeed: {:?}",
            tool_name,
            result
        );
        cleanup_test_env();
    }

    #[rstest]
    #[tokio::test]
    async fn test_list_nonexistent_tool(#[future] registry: ProviderRegistry) {
        let registry = registry.await;
        let ctx = create_test_context();
        let result = list::handle_list(
            &registry,
            &ctx,
            Some("nonexistent-tool-xyz"),
            false,
            false,
            false,
            vx_cli::OutputFormat::Text,
        )
        .await;
        // Should either succeed with empty result or return an error
        // The important thing is it doesn't panic
        let _ = result;
        cleanup_test_env();
    }
}

// ============================================================================
// Search Command Tests
// ============================================================================

mod search_tests {
    use super::*;
    use vx_cli::cli::OutputFormat;
    use vx_cli::commands::search;

    #[rstest]
    #[tokio::test]
    async fn test_search_no_query(#[future] registry: ProviderRegistry) {
        let registry = registry.await;
        let result = search::handle(
            &registry,
            None,
            None,
            false,
            false,
            OutputFormat::Text,
            false,
        )
        .await;
        assert!(result.is_ok(), "Search without query should succeed");
        cleanup_test_env();
    }

    #[rstest]
    #[case("node")]
    #[case("python")]
    #[case("go")]
    #[tokio::test]
    async fn test_search_with_query(#[future] registry: ProviderRegistry, #[case] query: &str) {
        let registry = registry.await;
        let result = search::handle(
            &registry,
            Some(query.to_string()),
            None,
            false,
            false,
            OutputFormat::Text,
            false,
        )
        .await;
        assert!(result.is_ok(), "Search for '{}' should succeed", query);
        cleanup_test_env();
    }

    #[rstest]
    #[tokio::test]
    async fn test_search_installed_only(#[future] registry: ProviderRegistry) {
        let registry = registry.await;
        let result = search::handle(
            &registry,
            None,
            None,
            true, // installed_only
            false,
            OutputFormat::Text,
            false,
        )
        .await;
        assert!(result.is_ok(), "Search installed only should succeed");
        cleanup_test_env();
    }

    #[rstest]
    #[tokio::test]
    async fn test_search_json_format(#[future] registry: ProviderRegistry) {
        let registry = registry.await;
        let result = search::handle(
            &registry,
            None,
            None,
            false,
            false,
            OutputFormat::Json,
            false,
        )
        .await;
        assert!(result.is_ok(), "Search with JSON format should succeed");
        cleanup_test_env();
    }
}

// ============================================================================
// Versions/Fetch Command Tests
// ============================================================================

mod versions_tests {
    use super::*;
    use vx_cli::commands::fetch;

    #[rstest]
    #[case("node")]
    #[case("go")]
    #[case("uv")]
    #[tokio::test]
    async fn test_fetch_versions(#[future] registry: ProviderRegistry, #[case] tool_name: &str) {
        let registry = registry.await;
        let ctx = create_test_context();
        // Fetch with latest=5 to limit network requests
        let result = fetch::handle(
            &registry,
            &ctx,
            tool_name,
            Some(5), // latest
            false,   // include_prerelease
            false,   // detailed
            false,   // interactive
            vx_cli::cli::OutputFormat::Text,
        )
        .await;
        // This may fail due to network issues, but should not panic
        let _ = result;
        cleanup_test_env();
    }

    #[rstest]
    #[tokio::test]
    async fn test_fetch_versions_with_prerelease(#[future] registry: ProviderRegistry) {
        let registry = registry.await;
        let ctx = create_test_context();
        let result = fetch::handle(
            &registry,
            &ctx,
            "node",
            Some(3),
            true,  // include prerelease
            false, // detailed
            false, // interactive
            vx_cli::cli::OutputFormat::Text,
        )
        .await;
        let _ = result;
        cleanup_test_env();
    }

    #[rstest]
    #[tokio::test]
    async fn test_fetch_nonexistent_tool(#[future] registry: ProviderRegistry) {
        let registry = registry.await;
        let ctx = create_test_context();
        let result = fetch::handle(
            &registry,
            &ctx,
            "nonexistent-tool-xyz",
            Some(5),
            false, // include_prerelease
            false, // detailed
            false, // interactive
            vx_cli::cli::OutputFormat::Text,
        )
        .await;
        // Should return an error for unknown tool
        assert!(result.is_err(), "Fetch for unknown tool should fail");
        cleanup_test_env();
    }
}

// ============================================================================
// Config Command Tests
// ============================================================================

mod config_tests {
    use super::*;
    use vx_cli::commands::config;

    #[rstest]
    #[tokio::test]
    async fn test_config_show() {
        init_test_env();
        let result = config::handle().await;
        assert!(result.is_ok(), "Config show should succeed");
        cleanup_test_env();
    }

    #[rstest]
    #[tokio::test]
    async fn test_config_get_nonexistent_key() {
        init_test_env();
        let result = config::handle_get("nonexistent.key").await;
        // Should handle gracefully
        let _ = result;
        cleanup_test_env();
    }
}

// ============================================================================
// Init Command Tests
// ============================================================================

mod init_tests {
    use super::*;
    use tempfile::TempDir;
    use vx_cli::commands::init;

    #[rstest]
    #[tokio::test]
    async fn test_init_list_templates() {
        init_test_env();
        let result = init::handle(
            false, // interactive
            None,  // template
            None,  // tools
            false, // force
            false, // dry_run
            true,  // list_templates
        )
        .await;
        assert!(result.is_ok(), "List templates should succeed");
        cleanup_test_env();
    }

    #[rstest]
    #[tokio::test]
    async fn test_init_dry_run() {
        init_test_env();
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let _guard = std::env::set_current_dir(temp_dir.path());

        let result = init::handle(
            false,                       // interactive
            None,                        // template
            Some("node,go".to_string()), // tools
            false,                       // force
            true,                        // dry_run
            false,                       // list_templates
        )
        .await;
        // Dry run should succeed without creating files
        let _ = result;
        cleanup_test_env();
    }

    #[rstest]
    #[case("node")]
    #[case("python")]
    #[case("go")]
    #[case("rust")]
    #[case("fullstack")]
    #[tokio::test]
    async fn test_init_with_template(#[case] template: &str) {
        init_test_env();
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let original_dir = std::env::current_dir().ok();

        let _ = std::env::set_current_dir(temp_dir.path());

        let result = init::handle(
            false,
            Some(template.to_string()),
            None,
            false,
            true, // dry_run to avoid file creation
            false,
        )
        .await;

        // Restore original directory
        if let Some(dir) = original_dir {
            let _ = std::env::set_current_dir(dir);
        }

        let _ = result;
        cleanup_test_env();
    }
}

// ============================================================================
// Clean Command Tests
// ============================================================================

// Note: cleanup command has been removed from vx-cli. These tests are commented out.
// If the command is re-added, uncomment these tests.
/*
mod clean_tests {
    use super::*;
    use vx_cli::commands::cleanup;

    #[rstest]
    #[tokio::test]
    async fn test_cleanup_dry_run() {
        init_test_env();
        let result = cleanup::handle(
            true,  // dry_run
            false, // cache_only
            false, // orphaned_only
            false, // force
            None,  // older_than
            false, // verbose
        )
        .await;
        assert!(result.is_ok(), "Cleanup dry run should succeed");
        cleanup_test_env();
    }

    #[rstest]
    #[tokio::test]
    async fn test_cleanup_cache_only_dry_run() {
        init_test_env();
        let result = cleanup::handle(
            true,  // dry_run
            true,  // cache_only
            false, // orphaned_only
            false, // force
            None,  // older_than
            true,  // verbose
        )
        .await;
        assert!(result.is_ok(), "Cleanup cache only should succeed");
        cleanup_test_env();
    }

    #[rstest]
    #[tokio::test]
    async fn test_cleanup_with_older_than() {
        init_test_env();
        let result = cleanup::handle(
            true,     // dry_run
            false,    // cache_only
            false,    // orphaned_only
            false,    // force
            Some(30), // older_than 30 days
            false,    // verbose
        )
        .await;
        assert!(result.is_ok(), "Cleanup with older_than should succeed");
        cleanup_test_env();
    }
}
*/

// ============================================================================
// Stats Command Tests
// ============================================================================

// Note: stats command has been removed from vx-cli. These tests are commented out.
// If the command is re-added, uncomment these tests.
/*
mod stats_tests {
    use super::*;
    use vx_cli::commands::stats;

    #[rstest]
    #[tokio::test]
    async fn test_stats_command(#[future] registry: ProviderRegistry) {
        let registry = registry.await;
        let result = stats::handle(&registry).await;
        assert!(result.is_ok(), "Stats command should succeed");
        cleanup_test_env();
    }
}
*/

// ============================================================================
// Plugin Command Tests
// ============================================================================

mod plugin_tests {
    use super::*;
    use vx_cli::cli::PluginCommand;
    use vx_cli::commands::plugin;

    #[rstest]
    #[tokio::test]
    async fn test_plugin_list(#[future] registry: ProviderRegistry) {
        let registry = registry.await;
        let result = plugin::handle(
            &registry,
            PluginCommand::List {
                enabled: false,
                category: None,
            },
        )
        .await;
        assert!(result.is_ok(), "Plugin list should succeed");
        cleanup_test_env();
    }

    #[rstest]
    #[tokio::test]
    async fn test_plugin_list_enabled_only(#[future] registry: ProviderRegistry) {
        let registry = registry.await;
        let result = plugin::handle(
            &registry,
            PluginCommand::List {
                enabled: true,
                category: None,
            },
        )
        .await;
        assert!(result.is_ok(), "Plugin list enabled should succeed");
        cleanup_test_env();
    }

    #[rstest]
    #[tokio::test]
    async fn test_plugin_stats(#[future] registry: ProviderRegistry) {
        let registry = registry.await;
        let result = plugin::handle(&registry, PluginCommand::Stats).await;
        assert!(result.is_ok(), "Plugin stats should succeed");
        cleanup_test_env();
    }

    #[rstest]
    #[case("node")]
    #[case("go")]
    #[case("uv")]
    #[tokio::test]
    async fn test_plugin_info(#[future] registry: ProviderRegistry, #[case] plugin_name: &str) {
        let registry = registry.await;
        let result = plugin::handle(
            &registry,
            PluginCommand::Info {
                name: plugin_name.to_string(),
            },
        )
        .await;
        // May fail if plugin not found, but should not panic
        let _ = result;
        cleanup_test_env();
    }

    #[rstest]
    #[tokio::test]
    async fn test_plugin_search(#[future] registry: ProviderRegistry) {
        let registry = registry.await;
        let result = plugin::handle(
            &registry,
            PluginCommand::Search {
                query: "node".to_string(),
            },
        )
        .await;
        assert!(result.is_ok(), "Plugin search should succeed");
        cleanup_test_env();
    }
}

// ============================================================================
// Shell Command Tests
// ============================================================================

mod shell_tests {
    use super::*;
    use vx_cli::commands::shell;

    #[rstest]
    #[case("bash")]
    #[case("zsh")]
    #[case("fish")]
    #[case("powershell")]
    #[tokio::test]
    async fn test_shell_completions(#[case] shell_name: &str) {
        init_test_env();
        let result = shell::handle_completion(shell_name.to_string()).await;
        assert!(
            result.is_ok(),
            "Shell completions for {} should succeed",
            shell_name
        );
        cleanup_test_env();
    }

    #[rstest]
    #[case("bash")]
    #[case("zsh")]
    #[case("fish")]
    #[case("powershell")]
    #[tokio::test]
    async fn test_shell_init(#[case] shell_name: &str) {
        init_test_env();
        let result = shell::handle_shell_init(Some(shell_name.to_string())).await;
        assert!(
            result.is_ok(),
            "Shell init for {} should succeed",
            shell_name
        );
        cleanup_test_env();
    }

    #[rstest]
    #[tokio::test]
    async fn test_shell_init_auto_detect() {
        init_test_env();
        let result = shell::handle_shell_init(None).await;
        // Auto-detection may fail in test environment, but should not panic
        let _ = result;
        cleanup_test_env();
    }
}

// ============================================================================
// Where/Which Command Tests
// ============================================================================

mod where_tests {
    use super::*;

    // Note: where_cmd::handle calls process::exit(1) when tool is not found,
    // which terminates the test process. These tests are moved to e2e_tests.rs
    // where we can test the CLI binary directly.

    #[rstest]
    #[test]
    fn test_where_command_exists() {
        // Just verify the module compiles and exists
        init_test_env();
        cleanup_test_env();
    }
}

// ============================================================================
// Switch Command Tests
// ============================================================================

mod switch_tests {
    use super::*;
    use vx_cli::commands::switch;

    #[rstest]
    #[tokio::test]
    async fn test_switch_invalid_format(#[future] registry: ProviderRegistry) {
        let registry = registry.await;
        // Invalid format (missing version)
        let result = switch::handle(&registry, "node", false).await;
        // Should fail gracefully
        let _ = result;
        cleanup_test_env();
    }

    #[rstest]
    #[tokio::test]
    async fn test_switch_nonexistent_version(#[future] registry: ProviderRegistry) {
        let registry = registry.await;
        let result = switch::handle(&registry, "node@99.99.99", false).await;
        // Note: Switch command is not fully implemented yet, so it may succeed
        // This test just verifies it doesn't panic
        let _ = result;
        cleanup_test_env();
    }
}

// ============================================================================
// Sync Command Tests
// ============================================================================

mod sync_tests {
    use super::*;
    use tempfile::TempDir;
    use vx_cli::commands::sync;

    #[rstest]
    #[tokio::test]
    async fn test_sync_check_no_config(#[future] registry: ProviderRegistry) {
        let registry = registry.await;
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let original_dir = std::env::current_dir().ok();

        let _ = std::env::set_current_dir(temp_dir.path());

        let result = sync::handle(
            &registry, true,  // check
            false, // force
            false, // dry_run
            false, // verbose
            false, // no_parallel
            false, // no_auto_install
        )
        .await;

        // Restore original directory
        if let Some(dir) = original_dir {
            let _ = std::env::set_current_dir(dir);
        }

        // Should handle missing config gracefully
        let _ = result;
        cleanup_test_env();
    }

    #[rstest]
    #[tokio::test]
    async fn test_sync_dry_run(#[future] registry: ProviderRegistry) {
        let registry = registry.await;
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let original_dir = std::env::current_dir().ok();

        // Create a vx.toml file
        let config_path = temp_dir.path().join("vx.toml");
        std::fs::write(
            &config_path,
            r#"
[tools]
node = "20"
"#,
        )
        .expect("Failed to write config");

        let _ = std::env::set_current_dir(temp_dir.path());

        let result = sync::handle(
            &registry, false, // check
            false, // force
            true,  // dry_run
            true,  // verbose
            false, // no_parallel
            false, // no_auto_install
        )
        .await;

        // Restore original directory
        if let Some(dir) = original_dir {
            let _ = std::env::set_current_dir(dir);
        }

        let _ = result;
        cleanup_test_env();
    }
}

// ============================================================================
// Install Command Tests (Dry/Mock)
// ============================================================================

mod install_tests {
    use super::*;
    use vx_cli::commands::install;

    #[rstest]
    #[tokio::test]
    async fn test_install_nonexistent_tool(#[future] registry: ProviderRegistry) {
        let registry = registry.await;
        let ctx = create_test_context();
        let result = install::handle_install(
            &registry,
            &ctx,
            &["nonexistent-tool-xyz@1.0.0".to_string()],
            false,
        )
        .await;
        assert!(result.is_err(), "Install nonexistent tool should fail");
        cleanup_test_env();
    }

    // Note: Actual installation tests are skipped in CI to avoid network dependencies
    // They can be run locally with: cargo test --features integration
}

// ============================================================================
// Uninstall/Remove Command Tests
// ============================================================================

mod uninstall_tests {
    use super::*;
    use vx_cli::commands::remove;

    #[rstest]
    #[tokio::test]
    async fn test_uninstall_nonexistent_tool(#[future] registry: ProviderRegistry) {
        let registry = registry.await;
        let ctx = create_test_context();
        let result = remove::handle(
            &registry,
            &ctx,
            "nonexistent-tool-xyz",
            None,
            true, // force
        )
        .await;
        // Should fail gracefully
        assert!(result.is_err(), "Uninstall nonexistent tool should fail");
        cleanup_test_env();
    }

    #[rstest]
    #[tokio::test]
    async fn test_uninstall_nonexistent_version(#[future] registry: ProviderRegistry) {
        let registry = registry.await;
        let ctx = create_test_context();
        let result = remove::handle(
            &registry,
            &ctx,
            "node",
            Some("99.99.99"),
            true, // force
        )
        .await;
        // Should fail because version doesn't exist
        let _ = result;
        cleanup_test_env();
    }
}

// ============================================================================
// Registry Tests
// ============================================================================

mod registry_tests {
    use super::*;

    #[rstest]
    #[tokio::test]
    async fn test_registry_has_all_tools(#[future] registry: ProviderRegistry) {
        let registry = registry.await;
        let runtime_names = registry.runtime_names();

        // Verify all expected tools are registered
        for expected_tool in SUPPORTED_TOOLS {
            let found = runtime_names.iter().any(|name| name == *expected_tool)
                || registry.get_runtime(expected_tool).is_some();
            assert!(
                found,
                "Expected tool '{}' not found in registry. Available: {:?}",
                expected_tool, runtime_names
            );
        }
        cleanup_test_env();
    }

    #[rstest]
    #[tokio::test]
    async fn test_registry_get_runtime(#[future] registry: ProviderRegistry) {
        let registry = registry.await;

        for tool_name in SUPPORTED_TOOLS {
            let runtime = registry.get_runtime(tool_name);
            assert!(
                runtime.is_some(),
                "Runtime '{}' should be retrievable from registry",
                tool_name
            );
        }
        cleanup_test_env();
    }

    #[rstest]
    #[tokio::test]
    async fn test_registry_list_providers(#[future] registry: ProviderRegistry) {
        let registry = registry.await;
        let providers = registry.providers();

        // Should have at least the registered providers
        assert!(
            !providers.is_empty(),
            "Registry should have registered providers"
        );
        cleanup_test_env();
    }
}

// ============================================================================
// Tool-Specific Tests
// ============================================================================

mod tool_specific_tests {
    use super::*;

    #[rstest]
    #[tokio::test]
    async fn test_node_plugin_registered(#[future] registry: ProviderRegistry) {
        let registry = registry.await;

        // Node provider should provide node, npm, npx
        let node = registry.get_runtime("node");
        assert!(node.is_some(), "Node runtime should be registered");

        let npm = registry.get_runtime("npm");
        assert!(npm.is_some(), "NPM runtime should be registered");

        let npx = registry.get_runtime("npx");
        assert!(npx.is_some(), "NPX runtime should be registered");

        cleanup_test_env();
    }

    #[rstest]
    #[tokio::test]
    async fn test_go_plugin_registered(#[future] registry: ProviderRegistry) {
        let registry = registry.await;

        let go = registry.get_runtime("go");
        assert!(go.is_some(), "Go runtime should be registered");

        cleanup_test_env();
    }

    #[rstest]
    #[tokio::test]
    async fn test_rust_plugin_registered(#[future] registry: ProviderRegistry) {
        let registry = registry.await;

        let rustc = registry.get_runtime("rustc");
        let cargo = registry.get_runtime("cargo");
        let rustup = registry.get_runtime("rustup");

        // At least one Rust runtime should be registered
        assert!(
            rustc.is_some() || cargo.is_some() || rustup.is_some(),
            "At least one Rust runtime should be registered"
        );

        cleanup_test_env();
    }

    #[rstest]
    #[tokio::test]
    async fn test_uv_plugin_registered(#[future] registry: ProviderRegistry) {
        let registry = registry.await;

        let uv = registry.get_runtime("uv");
        assert!(uv.is_some(), "UV runtime should be registered");

        let uvx = registry.get_runtime("uvx");
        assert!(uvx.is_some(), "UVX runtime should be registered");

        cleanup_test_env();
    }

    #[rstest]
    #[tokio::test]
    async fn test_bun_plugin_registered(#[future] registry: ProviderRegistry) {
        let registry = registry.await;

        let bun = registry.get_runtime("bun");
        assert!(bun.is_some(), "Bun runtime should be registered");

        // Note: bunx is not a separate runtime in the current implementation
        // It's executed as "bun x" command

        cleanup_test_env();
    }
}

// ============================================================================
// Error Handling Tests
// ============================================================================

mod error_handling_tests {
    use super::*;
    use vx_cli::commands::{fetch, install, remove, switch};

    #[rstest]
    #[tokio::test]
    async fn test_graceful_error_on_invalid_tool(#[future] registry: ProviderRegistry) {
        let registry = registry.await;
        let ctx = create_test_context();

        // All these should fail gracefully without panicking
        // Note: where_cmd::handle uses process::exit, so we skip it here
        let _ = fetch::handle(
            &registry,
            &ctx,
            "invalid-tool-xyz",
            Some(5),
            false, // include_prerelease
            false, // detailed
            false, // interactive
            vx_cli::OutputFormat::Text,
        )
        .await;
        let _ = install::handle_install(&registry, &ctx, &["invalid-tool-xyz".to_string()], false)
            .await;
        let _ = remove::handle(&registry, &ctx, "invalid-tool-xyz", None, true).await;
        let _ = switch::handle(&registry, "invalid-tool-xyz", false).await;

        cleanup_test_env();
    }

    #[rstest]
    #[tokio::test]
    async fn test_graceful_error_on_special_characters(#[future] registry: ProviderRegistry) {
        let registry = registry.await;
        let ctx = create_test_context();

        // Tools with special characters should be handled gracefully
        let special_names = vec!["../../../etc/passwd", "tool;rm -rf /", "tool$(whoami)"];

        for name in special_names {
            // These should return errors but not panic
            let _ = install::handle_install(&registry, &ctx, &[name.to_string()], false).await;
            let _ = remove::handle(&registry, &ctx, name, None, true).await;
        }

        cleanup_test_env();
    }
}
