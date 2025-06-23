//! URL optimization test using vx-download

use vx_download::VxDownloadManager;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 VX Download URL Optimization Test");
    println!("=====================================\n");

    // Create download manager
    let manager = VxDownloadManager::new().await?;
    println!("✅ Download manager initialized");

    // Test URLs from various sources
    let test_urls = vec![
        (
            "jsDelivr CDN",
            "https://cdn.jsdelivr.net/gh/jquery/jquery@3.6.0/dist/jquery.min.js",
        ),
        (
            "Cloudflare CDN",
            "https://cdnjs.cloudflare.com/ajax/libs/lodash.js/4.17.21/lodash.min.js",
        ),
        (
            "npm Registry",
            "https://registry.npmjs.org/express/-/express-4.18.2.tgz",
        ),
        (
            "PyPI",
            "https://files.pythonhosted.org/packages/source/c/click/click-8.1.3.tar.gz",
        ),
    ];

    for (source_name, url) in test_urls {
        println!("📦 Testing: {}", source_name);
        println!("🔗 URL: {}", url);

        // Parse URL information
        match manager.parse_url(url) {
            Ok(parsed) => {
                println!("  ✅ Parsed successfully");
                println!("  📦 Repository: {}", parsed.repository);
                println!("  🏷️ Version: {}", parsed.version);
                println!("  📄 Filename: {}", parsed.filename);
                println!("  🔍 Source Type: {:?}", parsed.source_type);
            }
            Err(e) => {
                println!("  ❌ Parse error: {}", e);
            }
        }

        // Test URL optimization
        match manager.get_optimal_url(url).await {
            Ok(optimal_url) => {
                println!("  ⚡ Optimal URL: {}", optimal_url);
                if optimal_url != url {
                    println!("  ✅ URL optimized for your location!");
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

    // Test version availability check
    println!("🔍 Testing version availability...");

    let tools_to_test = vec![
        ("python", "latest"),
        ("node", "latest"),
        ("go", "latest"),
        ("uv", "latest"),
    ];

    for (tool, version) in tools_to_test {
        match manager.is_version_available(tool, version).await {
            Ok(available) => {
                if available {
                    println!("  ✅ {} {} is available", tool, version);
                } else {
                    println!("  ❌ {} {} is not available", tool, version);
                }
            }
            Err(e) => {
                println!("  ❌ Failed to check {} {}: {}", tool, version, e);
            }
        }
    }

    println!("\n🎉 URL optimization test completed!");
    println!("\n💡 Key Benefits Demonstrated:");
    println!("  • Automatic URL parsing for multiple sources");
    println!("  • Geographic optimization (when available)");
    println!("  • Version availability checking");
    println!("  • Unified interface for different CDNs");

    Ok(())
}
