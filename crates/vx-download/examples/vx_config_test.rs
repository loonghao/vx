//! Test vx-optimized configuration for turbo-cdn

use vx_download::VxDownloadManager;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("🚀 VX Configuration Test for turbo-cdn 0.2.0");
    println!("==============================================\n");

    // Create download manager with vx-optimized config
    let mut manager = VxDownloadManager::new().await?;
    println!("✅ VxDownloadManager initialized with optimized config");

    // Test cache statistics
    let cache_stats = manager.cache_stats();
    println!("\n💾 Cache Configuration:");
    println!("  Enabled: {}", cache_stats.enabled);
    println!("  Max Size: {}", cache_stats.total_size_human());
    println!("  Usage: {:.1}%", cache_stats.usage_percentage());

    // Test URL validation for vx tools
    let test_urls = vec![
        ("GitHub Releases", "https://github.com/rust-lang/mdBook/releases/download/v0.4.21/mdbook-v0.4.21-x86_64-unknown-linux-gnu.tar.gz"),
        ("Node.js Official", "https://nodejs.org/dist/v18.17.0/node-v18.17.0-win-x64.zip"),
        ("Go Official", "https://golang.org/dl/go1.21.0.windows-amd64.zip"),
        ("Python Standalone", "https://github.com/astral-sh/python-build-standalone/releases/download/20230726/cpython-3.11.4+20230726-x86_64-unknown-linux-gnu-install_only.tar.gz"),
        ("jsDelivr CDN", "https://cdn.jsdelivr.net/gh/jquery/jquery@3.6.0/dist/jquery.min.js"),
        ("npm Registry", "https://registry.npmjs.org/express/-/express-4.18.2.tgz"),
        ("PyPI", "https://files.pythonhosted.org/packages/source/c/click/click-8.1.3.tar.gz"),
        ("Crates.io", "https://crates.io/api/v1/crates/tokio/1.28.0/download"),
    ];

    println!("\n🔍 URL Validation Test:");
    for (source_name, url) in &test_urls {
        let is_allowed = vx_download::vx_config::is_url_allowed_for_vx(url);
        let status = if is_allowed {
            "✅ Allowed"
        } else {
            "❌ Blocked"
        };
        println!("  {} {}: {}", status, source_name, url);
    }

    // Test URL parsing with relaxed compliance
    println!("\n📋 URL Parsing Test (with relaxed compliance):");
    for (source_name, url) in test_urls {
        println!("\n📦 Testing: {}", source_name);

        match manager.parse_url(url) {
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
        match manager.get_optimal_url(url).await {
            Ok(optimal_url) => {
                if optimal_url != url {
                    println!("  ⚡ Optimized: {}", optimal_url);
                } else {
                    println!("  ℹ️ Already optimal");
                }
            }
            Err(e) => {
                println!("  ⚠️ Optimization failed: {}", e);
            }
        }
    }

    // Test version availability checking
    println!("\n🔍 Version Availability Test:");
    let tools_to_test = vec![
        ("python", "latest"),
        ("node", "latest"),
        ("go", "latest"),
        ("uv", "latest"),
    ];

    for (tool, version) in tools_to_test {
        match manager.is_version_available(tool, version).await {
            Ok(available) => {
                let status = if available {
                    "✅ Available"
                } else {
                    "❌ Not available"
                };
                println!("  {} {} {}", status, tool, version);
            }
            Err(e) => {
                println!("  ❌ Check failed for {} {}: {}", tool, version, e);
            }
        }
    }

    // Test download speed estimation
    println!("\n⚡ Download Speed Estimation:");
    let speed_test_urls = vec![
        "https://cdn.jsdelivr.net/gh/jquery/jquery@3.6.0/dist/jquery.min.js",
        "https://registry.npmjs.org/express/-/express-4.18.2.tgz",
    ];

    for url in speed_test_urls {
        match manager.estimate_download_speed(url).await {
            Ok(speed) => {
                println!("  📊 {}: {:.2} MB/s", url, speed / 1_000_000.0);
            }
            Err(e) => {
                println!("  ❌ Speed estimation failed for {}: {}", url, e);
            }
        }
    }

    println!("\n🎉 VX Configuration test completed!");
    println!("\n💡 Key Optimizations Applied:");
    println!("  • Relaxed compliance for development tools");
    println!("  • Extended timeout (5 minutes) for large downloads");
    println!("  • Aggressive caching (7 days TTL)");
    println!("  • Optimized chunk size (4MB) for better performance");
    println!("  • Custom User-Agent for vx tools");
    println!("  • Extended domain whitelist for development sources");

    Ok(())
}
