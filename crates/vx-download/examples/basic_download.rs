//! Basic download example using vx-download

use tempfile::TempDir;
use vx_download::{ProgressInfo, VxDownloadManager};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("🚀 VX Download Manager Test");
    println!("============================\n");

    // Create download manager
    let mut manager = VxDownloadManager::new().await?;
    println!("✅ Download manager initialized");

    // Test URL optimization with turbo-cdn 0.2.0
    let test_urls = vec![
        ("GitHub Releases", "https://github.com/rust-lang/mdBook/releases/download/v0.4.21/mdbook-v0.4.21-x86_64-unknown-linux-gnu.tar.gz"),
        ("jsDelivr CDN", "https://cdn.jsdelivr.net/gh/jquery/jquery@3.6.0/dist/jquery.min.js"),
        ("npm Registry", "https://registry.npmjs.org/express/-/express-4.18.2.tgz"),
    ];

    for (source_name, test_url) in test_urls {
        println!("\n📦 Testing: {}", source_name);
        println!("🔗 URL: {}", test_url);

        // Get optimal URL
        match manager.get_optimal_url(test_url).await {
            Ok(optimal_url) => {
                println!("  ⚡ Optimal URL: {}", optimal_url);
                if optimal_url != test_url {
                    println!("  ✅ URL optimized!");
                } else {
                    println!("  ℹ️ Original URL is already optimal");
                }
            }
            Err(e) => {
                println!("  ❌ Failed to get optimal URL: {}", e);
            }
        }

        // Parse URL information
        match manager.parse_url(test_url) {
            Ok(parsed) => {
                println!("  📋 Parsed Information:");
                println!("    Repository: {}", parsed.repository);
                println!("    Version: {}", parsed.version);
                println!("    Filename: {}", parsed.filename);
                println!("    Source Type: {:?}", parsed.source_type);
            }
            Err(e) => {
                println!("  ❌ Failed to parse URL: {}", e);
            }
        }
    }

    // Test download with progress
    let temp_dir = TempDir::new()?;
    let output_path = temp_dir.path().join("test_download.tar.gz");

    println!("\n📥 Starting download test...");

    let progress_callback = Box::new(|progress: ProgressInfo| {
        println!(
            "📊 Progress: {:.1}% ({}) - {} - ETA: {}",
            progress.percentage,
            progress.size_human(),
            progress.speed_human(),
            progress.eta_human()
        );
    });

    // Test smaller file for faster testing
    let small_test_url = "https://cdn.jsdelivr.net/gh/jquery/jquery@3.6.0/dist/jquery.min.js";

    match manager
        .download_tool(
            "test",
            "latest",
            small_test_url,
            &output_path,
            Some(progress_callback),
        )
        .await
    {
        Ok(result) => {
            println!("✅ Download completed!");
            println!("  📁 Path: {}", result.path.display());
            println!("  📊 Size: {} bytes", result.size);
            println!("  ⚡ Speed: {:.2} MB/s", result.speed / 1_000_000.0);
            println!("  🌐 Source: {}", result.source);
            println!("  💾 From cache: {}", result.from_cache);
        }
        Err(e) => {
            println!("❌ Download failed: {}", e);
        }
    }

    // Test cache statistics
    let cache_stats = manager.cache_stats();
    println!("\n💾 Cache Statistics:");
    println!("  Enabled: {}", cache_stats.enabled);
    println!("  Entries: {}", cache_stats.entry_count);
    println!("  Size: {}", cache_stats.total_size_human());
    println!("  Usage: {:.1}%", cache_stats.usage_percentage());

    println!("\n🎉 Test completed!");
    Ok(())
}
