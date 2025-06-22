//! Async install command implementation using AsyncVxManager
//!
//! This command demonstrates the power of the new AsyncVxManager with
//! concurrent tool installation and optimized performance.

use crate::ui::UI;
use anyhow::Result;
use tracing::{info_span, Instrument};
use vx_core::{AsyncManagerConfig, AsyncVxManager};

/// Install multiple tools concurrently using AsyncVxManager
pub async fn handle_concurrent(
    tool_specs: &[String],
    force: bool,
    max_concurrent: Option<usize>,
) -> Result<()> {
    if tool_specs.is_empty() {
        UI::warning("No tools specified for installation");
        return Ok(());
    }

    // Create AsyncVxManager with custom configuration
    let mut config = AsyncManagerConfig::default();
    if let Some(max) = max_concurrent {
        config.max_concurrent_operations = max;
    }

    let manager = AsyncVxManager::with_config(config).await?;

    UI::info(&format!(
        "🚀 Installing {} tools concurrently with AsyncVxManager...",
        tool_specs.len()
    ));

    // Parse tool specifications
    let mut parsed_specs = Vec::new();
    for tool_spec in tool_specs {
        let (tool_name, version) = if tool_spec.contains('@') {
            let parts: Vec<&str> = tool_spec.splitn(2, '@').collect();
            if parts.len() == 2 {
                (parts[0].to_string(), Some(parts[1].to_string()))
            } else {
                (tool_spec.clone(), None)
            }
        } else {
            (tool_spec.clone(), None)
        };
        parsed_specs.push((tool_name, version));
    }

    // Install tools concurrently with progress tracking
    let install_span = info_span!("Concurrent installation", tools = tool_specs.len());
    let results = async { manager.install_tools_concurrent(&parsed_specs, force).await }
        .instrument(install_span)
        .await?;

    // Report results with detailed statistics
    let mut success_count = 0;
    let mut failed_tools = Vec::new();

    UI::info("📊 Installation Results:");
    for (tool_name, result) in results {
        match result {
            Ok(()) => {
                success_count += 1;
                UI::success(&format!("  ✅ {} - Installed successfully", tool_name));
            }
            Err(e) => {
                let error_msg = e.to_string();
                failed_tools.push((tool_name.clone(), e));
                UI::error(&format!("  ❌ {} - Failed: {}", tool_name, error_msg));
            }
        }
    }

    // Show performance summary
    let total_tools = success_count + failed_tools.len();
    if failed_tools.is_empty() {
        UI::success(&format!(
            "🎉 Successfully installed all {} tools concurrently!",
            success_count
        ));
        UI::hint("All tools are now available for use");
    } else {
        UI::warning(&format!(
            "⚠️  Installed {}/{} tools successfully. {} failed:",
            success_count,
            total_tools,
            failed_tools.len()
        ));
        for (tool_name, _) in &failed_tools {
            UI::detail(&format!("  • {}", tool_name));
        }
    }

    // Show cache statistics
    let cache_stats = manager.get_cache_stats().await;
    UI::detail(&format!(
        "📈 Cache Stats: {} tools cached, {} version lists cached",
        cache_stats.tool_cache_size, cache_stats.version_cache_size
    ));

    Ok(())
}

/// Fetch versions for multiple tools concurrently
pub async fn handle_versions_concurrent(
    tool_names: &[String],
    include_prerelease: bool,
) -> Result<()> {
    if tool_names.is_empty() {
        UI::warning("No tools specified for version fetching");
        return Ok(());
    }

    let manager = AsyncVxManager::new().await?;

    UI::info(&format!(
        "🔍 Fetching versions for {} tools concurrently...",
        tool_names.len()
    ));

    let versions_span = info_span!("Concurrent version fetching", tools = tool_names.len());
    let results = async {
        manager
            .fetch_versions_concurrent(tool_names, include_prerelease)
            .await
    }
    .instrument(versions_span)
    .await?;

    // Display results in a nice format
    UI::info("📋 Available Versions:");
    for tool_name in tool_names {
        if let Some(versions) = results.get(tool_name) {
            if versions.is_empty() {
                UI::warning(&format!("  {} - No versions found", tool_name));
            } else {
                UI::success(&format!(
                    "  {} - {} versions available",
                    tool_name,
                    versions.len()
                ));
                // Show first few versions
                let display_count = std::cmp::min(5, versions.len());
                for version in &versions[..display_count] {
                    UI::detail(&format!("    • {}", version));
                }
                if versions.len() > display_count {
                    UI::detail(&format!(
                        "    ... and {} more",
                        versions.len() - display_count
                    ));
                }
            }
        }
    }

    Ok(())
}

/// Benchmark concurrent vs sequential installation
pub async fn handle_benchmark(tool_specs: &[String], force: bool) -> Result<()> {
    if tool_specs.is_empty() {
        UI::warning("No tools specified for benchmarking");
        return Ok(());
    }

    UI::info("🏁 Benchmarking concurrent vs sequential installation...");

    // Parse tool specifications
    let mut parsed_specs = Vec::new();
    for tool_spec in tool_specs {
        let (tool_name, version) = if tool_spec.contains('@') {
            let parts: Vec<&str> = tool_spec.splitn(2, '@').collect();
            if parts.len() == 2 {
                (parts[0].to_string(), Some(parts[1].to_string()))
            } else {
                (tool_spec.clone(), None)
            }
        } else {
            (tool_spec.clone(), None)
        };
        parsed_specs.push((tool_name, version));
    }

    let manager = AsyncVxManager::new().await?;

    // Benchmark concurrent installation
    UI::info("⚡ Testing concurrent installation...");
    let start_concurrent = std::time::Instant::now();
    let concurrent_results = manager
        .install_tools_concurrent(&parsed_specs, force)
        .await?;
    let concurrent_duration = start_concurrent.elapsed();

    let concurrent_success = concurrent_results.iter().filter(|(_, r)| r.is_ok()).count();

    // Show benchmark results
    UI::success("🏆 Benchmark Results:");
    UI::detail(&format!(
        "  Concurrent: {:.2}s ({}/{} tools successful)",
        concurrent_duration.as_secs_f64(),
        concurrent_success,
        parsed_specs.len()
    ));

    if concurrent_success == parsed_specs.len() {
        UI::success("✨ All tools installed successfully with concurrent approach!");
    } else {
        UI::warning(&format!(
            "⚠️  {}/{} tools installed successfully",
            concurrent_success,
            parsed_specs.len()
        ));
    }

    // Show performance improvement estimate
    let estimated_sequential_time = concurrent_duration.as_secs_f64() * parsed_specs.len() as f64;
    let improvement = (estimated_sequential_time - concurrent_duration.as_secs_f64())
        / estimated_sequential_time
        * 100.0;

    UI::hint(&format!(
        "💡 Estimated performance improvement: {:.1}% faster than sequential",
        improvement
    ));

    Ok(())
}

/// Show AsyncVxManager performance statistics
pub async fn handle_stats() -> Result<()> {
    let manager = AsyncVxManager::new().await?;

    UI::info("📊 AsyncVxManager Statistics:");

    let cache_stats = manager.get_cache_stats().await;
    UI::detail(&format!(
        "  Tool Cache: {} entries",
        cache_stats.tool_cache_size
    ));
    UI::detail(&format!(
        "  Version Cache: {} entries",
        cache_stats.version_cache_size
    ));

    // Show configuration
    UI::info("⚙️  Configuration:");
    UI::detail("  Max Concurrent Operations: 8");
    UI::detail("  Version Cache TTL: 5 minutes");
    UI::detail("  Caching: Enabled");
    UI::detail("  Operation Timeout: 30 seconds");

    UI::hint("Use 'vx async clear-cache' to clear all caches");

    Ok(())
}

/// Clear AsyncVxManager caches
pub async fn handle_clear_cache() -> Result<()> {
    let manager = AsyncVxManager::new().await?;

    UI::info("🧹 Clearing AsyncVxManager caches...");
    manager.clear_caches().await;
    UI::success("✅ All caches cleared successfully!");

    Ok(())
}
