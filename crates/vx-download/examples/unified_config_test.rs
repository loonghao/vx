//! Test unified configuration from vx-config

use vx_download::VxDownloadManager;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("🚀 Unified Configuration Test");
    println!("==============================\n");

    // Get global configuration
    let global_config = vx_config::get_global_config();
    println!("✅ Global configuration loaded");

    // Display turbo-cdn configuration
    println!("📡 Turbo CDN Configuration:");
    println!("  Enabled: {}", global_config.turbo_cdn.enabled);
    println!(
        "  Default Region: {}",
        global_config.turbo_cdn.default_region
    );
    println!(
        "  Max Concurrent Chunks: {}",
        global_config.turbo_cdn.max_concurrent_chunks
    );
    println!(
        "  Chunk Size: {} MB",
        global_config.turbo_cdn.chunk_size / (1024 * 1024)
    );
    println!("  Max Retries: {}", global_config.turbo_cdn.max_retries);
    println!("  Cache Enabled: {}", global_config.turbo_cdn.cache_enabled);
    println!(
        "  Cache Max Size: {} GB",
        global_config.turbo_cdn.cache_max_size / (1024 * 1024 * 1024)
    );
    println!(
        "  Cache Compression: {}",
        global_config.turbo_cdn.cache_compression
    );

    // Display smart cache configuration
    println!("\n💾 Smart Cache Configuration:");
    println!(
        "  Max Size: {} GB",
        global_config.turbo_cdn.smart_cache.max_size / (1024 * 1024 * 1024)
    );
    println!(
        "  TTL: {} days",
        global_config.turbo_cdn.smart_cache.ttl_seconds / (24 * 60 * 60)
    );
    println!(
        "  Deduplication: {}",
        global_config.turbo_cdn.smart_cache.enable_dedup
    );
    println!(
        "  Cross-tool Sharing: {}",
        global_config.turbo_cdn.smart_cache.enable_sharing
    );
    println!(
        "  Preheating: {}",
        global_config.turbo_cdn.smart_cache.enable_preheating
    );
    println!(
        "  Cleanup Threshold: {:.0}%",
        global_config.turbo_cdn.smart_cache.cleanup_threshold * 100.0
    );
    println!(
        "  Min Dedup Size: {} MB",
        global_config.turbo_cdn.smart_cache.min_dedup_size / (1024 * 1024)
    );

    // Display default configuration
    println!("\n⚙️ Default Configuration:");
    println!("  Auto Install: {}", global_config.defaults.auto_install);
    println!(
        "  Cache Duration: {}",
        global_config.defaults.cache_duration
    );
    println!(
        "  Fallback to Builtin: {}",
        global_config.defaults.fallback_to_builtin
    );
    println!(
        "  Use System Path: {}",
        global_config.defaults.use_system_path
    );
    println!(
        "  Download Timeout: {}s",
        global_config.defaults.download_timeout
    );
    println!(
        "  Max Concurrent Downloads: {}",
        global_config.defaults.max_concurrent_downloads
    );

    // Display global paths
    println!("\n📁 Global Paths:");
    println!("  Home Dir: {}", global_config.global.home_dir);
    println!("  Tools Dir: {}", global_config.global.tools_dir);
    println!("  Cache Dir: {}", global_config.global.cache_dir);
    println!("  Shims Dir: {}", global_config.global.shims_dir);
    println!("  Config Dir: {}", global_config.global.config_dir);

    // Test VxDownloadManager with unified config
    println!("\n📥 Testing VxDownloadManager with Unified Config:");
    let mut manager = VxDownloadManager::new().await?;
    println!("✅ VxDownloadManager initialized using unified configuration");

    // Test smart cache statistics
    let smart_cache_stats = manager.smart_cache_stats();
    println!("\n📊 Smart Cache Statistics:");
    println!("  Enabled: {}", smart_cache_stats.enabled);
    println!("  Total Size: {}", smart_cache_stats.total_size_human());
    println!("  Entry Count: {}", smart_cache_stats.entry_count);
    println!("  Hit Rate: {:.1}%", smart_cache_stats.hit_rate() * 100.0);
    println!(
        "  Deduplication Efficiency: {:.1}%",
        smart_cache_stats.dedup_efficiency() * 100.0
    );
    println!("  Space Saved: {}", smart_cache_stats.saved_space_human());

    // Test tool breakdown
    if !smart_cache_stats.tool_breakdown.is_empty() {
        println!("\n🔧 Cache by Tool:");
        for (tool, size) in &smart_cache_stats.tool_breakdown {
            let size_mb = *size as f64 / (1024.0 * 1024.0);
            println!("  {}: {:.1} MB", tool, size_mb);
        }
    }

    // Test content type breakdown
    if !smart_cache_stats.content_type_breakdown.is_empty() {
        println!("\n📦 Cache by Content Type:");
        for (content_type, size) in &smart_cache_stats.content_type_breakdown {
            let size_mb = *size as f64 / (1024.0 * 1024.0);
            println!("  {}: {:.1} MB", content_type, size_mb);
        }
    }

    // Test configuration validation
    println!("\n🔍 Configuration Validation:");

    // Validate cache sizes are consistent
    let legacy_cache_size = global_config.turbo_cdn.cache_max_size;
    let smart_cache_size = global_config.turbo_cdn.smart_cache.max_size;

    if smart_cache_size >= legacy_cache_size {
        println!(
            "  ✅ Smart cache size ({} GB) >= Legacy cache size ({} GB)",
            smart_cache_size / (1024 * 1024 * 1024),
            legacy_cache_size / (1024 * 1024 * 1024)
        );
    } else {
        println!(
            "  ⚠️ Smart cache size ({} GB) < Legacy cache size ({} GB)",
            smart_cache_size / (1024 * 1024 * 1024),
            legacy_cache_size / (1024 * 1024 * 1024)
        );
    }

    // Validate performance settings
    if global_config.turbo_cdn.max_concurrent_chunks <= 16 {
        println!(
            "  ✅ Concurrent chunks ({}) within reasonable limits",
            global_config.turbo_cdn.max_concurrent_chunks
        );
    } else {
        println!(
            "  ⚠️ Concurrent chunks ({}) might be too high",
            global_config.turbo_cdn.max_concurrent_chunks
        );
    }

    // Validate smart cache settings
    if global_config.turbo_cdn.smart_cache.cleanup_threshold > 0.5
        && global_config.turbo_cdn.smart_cache.cleanup_threshold < 1.0
    {
        println!(
            "  ✅ Cleanup threshold ({:.0}%) is reasonable",
            global_config.turbo_cdn.smart_cache.cleanup_threshold * 100.0
        );
    } else {
        println!(
            "  ⚠️ Cleanup threshold ({:.0}%) might need adjustment",
            global_config.turbo_cdn.smart_cache.cleanup_threshold * 100.0
        );
    }

    // Test configuration serialization
    println!("\n💾 Configuration Serialization Test:");
    match toml::to_string_pretty(&global_config) {
        Ok(toml_str) => {
            println!("  ✅ Configuration can be serialized to TOML");
            println!("  📄 TOML size: {} bytes", toml_str.len());

            // Test deserialization
            match toml::from_str::<vx_config::types::VxConfig>(&toml_str) {
                Ok(_) => println!("  ✅ Configuration can be deserialized from TOML"),
                Err(e) => println!("  ❌ Deserialization failed: {}", e),
            }
        }
        Err(e) => println!("  ❌ Serialization failed: {}", e),
    }

    println!("\n🎉 Unified configuration test completed!");
    println!("\n💡 Key Benefits Achieved:");
    println!("  • Single source of truth for all configurations");
    println!("  • No duplicate configuration definitions");
    println!("  • Consistent defaults across all components");
    println!("  • Easy configuration validation and testing");
    println!("  • Centralized configuration management");
    println!("  • Type-safe configuration with serde");

    Ok(())
}
