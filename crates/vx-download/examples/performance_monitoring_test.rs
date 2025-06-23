//! Test performance monitoring and reporting

use std::time::Duration;
use vx_download::monitoring::{DownloadMetrics, PerformanceMonitor};
use vx_download::VxDownloadManager;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("🚀 Performance Monitoring Test");
    println!("===============================\n");

    // Test standalone performance monitor
    println!("📊 Testing Standalone Performance Monitor:");
    test_standalone_monitor().await?;

    // Test integrated performance monitoring
    println!("\n📥 Testing Integrated Performance Monitoring:");
    test_integrated_monitoring().await?;

    println!("\n🎉 Performance monitoring test completed!");
    println!("\n💡 Key Features Demonstrated:");
    println!("  • Real-time download metrics tracking");
    println!("  • Cache hit/miss rate monitoring");
    println!("  • CDN source performance analysis");
    println!("  • Tool-specific performance breakdown");
    println!("  • Intelligent performance recommendations");
    println!("  • JSON metrics export capability");
    println!("  • Human-readable performance reports");

    Ok(())
}

async fn test_standalone_monitor() -> Result<(), Box<dyn std::error::Error>> {
    let mut monitor = PerformanceMonitor::new();
    println!("✅ Performance monitor initialized");

    // Simulate various download scenarios
    println!("\n🔄 Simulating Download Scenarios:");

    // Scenario 1: Fast cache hit
    let cache_hit_metrics = DownloadMetrics {
        url: "https://github.com/astral-sh/uv/releases/download/0.1.0/uv-windows-x64.zip"
            .to_string(),
        tool_name: "uv".to_string(),
        file_size: 15_000_000,               // 15MB
        duration: Duration::from_millis(50), // Very fast from cache
        speed_bps: 0.0,                      // Instant from cache
        source: "smart-cache".to_string(),
        success: true,
        error: None,
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        retry_count: 0,
        cache_hit: true,
        region: "Global".to_string(),
    };

    monitor.record_cache_hit(Duration::from_millis(50));
    monitor.record_download(cache_hit_metrics);
    println!("  ✅ Cache hit scenario recorded");

    // Scenario 2: CDN download
    let cdn_download_metrics = DownloadMetrics {
        url: "https://nodejs.org/dist/v18.17.0/node-v18.17.0-windows-x64.zip".to_string(),
        tool_name: "nodejs".to_string(),
        file_size: 45_000_000,            // 45MB
        duration: Duration::from_secs(8), // 8 seconds
        speed_bps: 5_625_000.0,           // ~5.6 MB/s
        source: "nodejs-official".to_string(),
        success: true,
        error: None,
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        retry_count: 0,
        cache_hit: false,
        region: "Global".to_string(),
    };

    monitor.record_cache_miss();
    monitor.record_download(cdn_download_metrics);
    println!("  ✅ CDN download scenario recorded");

    // Scenario 3: Slow download
    let slow_download_metrics = DownloadMetrics {
        url: "https://golang.org/dl/go1.21.0.windows-x64.zip".to_string(),
        tool_name: "golang".to_string(),
        file_size: 120_000_000,            // 120MB
        duration: Duration::from_secs(45), // 45 seconds
        speed_bps: 2_666_667.0,            // ~2.7 MB/s
        source: "golang-official".to_string(),
        success: true,
        error: None,
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        retry_count: 1,
        cache_hit: false,
        region: "Global".to_string(),
    };

    monitor.record_cache_miss();
    monitor.record_download(slow_download_metrics);
    println!("  ✅ Slow download scenario recorded");

    // Scenario 4: Failed download
    let failed_download_metrics = DownloadMetrics {
        url: "https://example.com/nonexistent-tool.zip".to_string(),
        tool_name: "unknown".to_string(),
        file_size: 0,
        duration: Duration::from_secs(30), // Timeout
        speed_bps: 0.0,
        source: "unknown".to_string(),
        success: false,
        error: Some("Connection timeout".to_string()),
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        retry_count: 3,
        cache_hit: false,
        region: "Global".to_string(),
    };

    monitor.record_cache_miss();
    monitor.record_download(failed_download_metrics);
    println!("  ❌ Failed download scenario recorded");

    // Update cache size metrics
    monitor.update_cache_size(
        500_000_000, // 500MB total cache
        25,          // 25 entries
        5,           // 5 deduplicated files
        100_000_000, // 100MB saved through deduplication
    );
    println!("  📊 Cache size metrics updated");

    // Generate performance summary
    println!("\n📈 Performance Summary:");
    let summary = monitor.generate_summary();
    println!("  Total Downloads: {}", summary.total_downloads);
    println!("  Success Rate: {:.1}%", summary.success_rate * 100.0);
    println!("  Average Speed: {:.2} MB/s", summary.avg_speed_mbps);
    println!(
        "  Cache Hit Rate: {:.1}%",
        summary.cache_metrics.hit_rate * 100.0
    );
    println!(
        "  Deduplication Efficiency: {:.1}%",
        summary.cache_metrics.dedup_saved_bytes as f64
            / (summary.cache_metrics.total_size + summary.cache_metrics.dedup_saved_bytes) as f64
            * 100.0
    );

    // Test CDN source performance
    println!("\n🌐 CDN Source Performance:");
    for (source, metrics) in &summary.cdn_metrics {
        let success_rate = metrics.successful_requests as f64 / metrics.total_requests as f64;
        println!(
            "  {} - {:.1}% success, {:.2} MB/s avg",
            source,
            success_rate * 100.0,
            metrics.avg_speed / 1_000_000.0
        );
    }

    // Test tool-specific metrics
    println!("\n🔧 Tool-Specific Performance:");
    for (tool, metrics) in &summary.tool_metrics {
        println!(
            "  {} - {} downloads, {:.1}% success, preferred source: {}",
            tool,
            metrics.downloads,
            metrics.success_rate * 100.0,
            metrics.preferred_source
        );
    }

    // Generate full performance report
    println!("\n📋 Full Performance Report:");
    let report = monitor.generate_report();
    println!("{}", report);

    // Test JSON export
    println!("\n💾 JSON Export Test:");
    match monitor.export_metrics() {
        Ok(json) => {
            println!("  ✅ Metrics exported to JSON ({} bytes)", json.len());
            // Verify it's valid JSON
            match serde_json::from_str::<serde_json::Value>(&json) {
                Ok(_) => println!("  ✅ JSON is valid and parseable"),
                Err(e) => println!("  ❌ JSON parsing failed: {}", e),
            }
        }
        Err(e) => println!("  ❌ JSON export failed: {}", e),
    }

    // Test recent metrics retrieval
    println!("\n🕒 Recent Metrics Test:");
    let recent_metrics = monitor.get_recent_metrics(3);
    println!("  📊 Retrieved {} recent metrics", recent_metrics.len());
    for (i, metrics) in recent_metrics.iter().enumerate() {
        println!(
            "    {}. {} - {} ({:.2} MB/s)",
            i + 1,
            metrics.tool_name,
            if metrics.success { "✅" } else { "❌" },
            metrics.speed_bps / 1_000_000.0
        );
    }

    Ok(())
}

async fn test_integrated_monitoring() -> Result<(), Box<dyn std::error::Error>> {
    let mut manager = VxDownloadManager::new().await?;
    println!("✅ VxDownloadManager with integrated monitoring initialized");

    // Update cache metrics
    manager.update_cache_metrics();
    println!("✅ Cache metrics updated");

    // Get performance summary
    let summary = manager.performance_summary();
    println!("\n📊 Integrated Performance Summary:");
    println!("  Total Downloads: {}", summary.total_downloads);
    println!(
        "  Cache Hit Rate: {:.1}%",
        summary.cache_metrics.hit_rate * 100.0
    );
    println!(
        "  Cache Size: {:.2} GB",
        summary.cache_metrics.total_size as f64 / 1_000_000_000.0
    );

    // Generate performance report
    println!("\n📋 Integrated Performance Report:");
    let report = manager.performance_report();
    println!("{}", report);

    // Test JSON export
    println!("\n💾 Integrated JSON Export Test:");
    match manager.export_performance_metrics() {
        Ok(json) => {
            println!(
                "  ✅ Integrated metrics exported to JSON ({} bytes)",
                json.len()
            );

            // Parse and display key metrics
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&json) {
                if let Some(total_downloads) = parsed.get("total_downloads") {
                    println!("    📥 Total Downloads: {}", total_downloads);
                }
                if let Some(cache_metrics) = parsed.get("cache_metrics") {
                    if let Some(hit_rate) = cache_metrics.get("hit_rate") {
                        println!(
                            "    💾 Cache Hit Rate: {:.1}%",
                            hit_rate.as_f64().unwrap_or(0.0) * 100.0
                        );
                    }
                }
            }
        }
        Err(e) => println!("  ❌ Integrated JSON export failed: {}", e),
    }

    // Test cache statistics integration
    println!("\n💾 Cache Statistics Integration:");
    let smart_stats = manager.smart_cache_stats();
    println!("  Smart Cache Entries: {}", smart_stats.entry_count);
    println!("  Smart Cache Size: {}", smart_stats.total_size_human());
    println!("  Deduplication Count: {}", smart_stats.dedup_count);
    println!("  Space Saved: {}", smart_stats.saved_space_human());

    Ok(())
}
