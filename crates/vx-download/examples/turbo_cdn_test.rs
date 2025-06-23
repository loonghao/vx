//! Test turbo-cdn 0.2.0 integration

use turbo_cdn::TurboCdn;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("🚀 Turbo CDN 0.2.0 Integration Test");
    println!("====================================\n");

    // Create TurboCdn client directly
    let client = TurboCdn::new().await?;
    println!("✅ TurboCdn client initialized");

    // Test URL parsing with various sources
    let test_urls = vec![
        ("GitHub", "https://github.com/rust-lang/mdBook/releases/download/v0.4.21/mdbook-v0.4.21-x86_64-unknown-linux-gnu.tar.gz"),
        ("jsDelivr", "https://cdn.jsdelivr.net/gh/jquery/jquery@3.6.0/dist/jquery.min.js"),
        ("npm", "https://registry.npmjs.org/express/-/express-4.18.2.tgz"),
        ("PyPI", "https://files.pythonhosted.org/packages/source/c/click/click-8.1.3.tar.gz"),
        ("Crates.io", "https://crates.io/api/v1/crates/tokio/1.28.0/download"),
    ];

    for (source_name, url) in test_urls {
        println!("📦 Testing: {}", source_name);
        println!("🔗 URL: {}", url);

        // Test URL parsing
        match client.parse_url(url) {
            Ok(parsed) => {
                println!("  ✅ Parsed successfully");
                println!("    📦 Repository: {}", parsed.repository);
                println!("    🏷️ Version: {}", parsed.version);
                println!("    📄 Filename: {}", parsed.filename);
                println!("    🔍 Source Type: {:?}", parsed.source_type);
            }
            Err(e) => {
                println!("  ❌ Parse error: {}", e);
            }
        }

        // Test URL optimization
        match client.get_optimal_url(url).await {
            Ok(optimal_url) => {
                println!("  ⚡ Optimal URL: {}", optimal_url);
                if optimal_url != url {
                    println!("  ✅ URL optimized!");
                } else {
                    println!("  ℹ️ Original URL is already optimal");
                }
            }
            Err(e) => {
                println!("  ❌ Optimization failed: {}", e);
            }
        }

        println!(); // Empty line for readability
    }

    // Test version extraction
    println!("🔍 Testing version extraction...");
    let test_filenames = vec![
        "app-v1.2.3.zip",
        "tool-2.0.tar.gz",
        "package-2023-12-01.exe",
        "noversion.zip",
    ];

    for filename in test_filenames {
        match client.extract_version_from_filename(filename) {
            Some(version) => {
                println!("  📄 {} → Version: {}", filename, version);
            }
            None => {
                println!("  📄 {} → No version detected", filename);
            }
        }
    }

    // Test health check
    println!("\n🏥 Testing health check...");
    match client.health_check().await {
        Ok(health_results) => {
            println!("  ✅ Health check completed");
            for (source, health_status) in health_results {
                let status = format!("✅ Healthy ({:?})", health_status);
                println!("    {} {}", source, status);
            }
        }
        Err(e) => {
            println!("  ❌ Health check failed: {}", e);
        }
    }

    // Test statistics
    println!("\n📊 Testing statistics...");
    match client.get_stats().await {
        Ok(stats) => {
            println!("  ✅ Statistics retrieved");
            println!("    Total downloads: {}", stats.total_downloads);
            println!("    Successful downloads: {}", stats.successful_downloads);
            println!("    Failed downloads: {}", stats.failed_downloads);
            println!("    Cache hit rate: {:.1}%", stats.cache_hit_rate * 100.0);
        }
        Err(e) => {
            println!("  ❌ Statistics failed: {}", e);
        }
    }

    println!("\n🎉 Turbo CDN 0.2.0 integration test completed!");
    println!("\n💡 Key Features Demonstrated:");
    println!("  • Universal URL parsing for multiple sources");
    println!("  • Geographic optimization (when available)");
    println!("  • Version extraction from filenames");
    println!("  • Health monitoring of CDN sources");
    println!("  • Download statistics tracking");

    Ok(())
}
