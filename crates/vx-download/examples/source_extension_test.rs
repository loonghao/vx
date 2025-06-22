//! Test VX source extension for development tools

use vx_download::{sources::VxSourceRegistry, VxDownloadManager};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("🚀 VX Source Extension Test");
    println!("============================\n");

    // Test VX source registry
    let registry = VxSourceRegistry::new();
    println!("✅ VX Source Registry initialized");

    // Test all supported sources
    println!("\n📦 Supported VX Sources:");
    for (name, source) in registry.get_all_sources() {
        println!("  🔧 {}: {}", name, source.base_url);
        println!("    📋 Domains: {:?}", source.supported_domains);
        println!(
            "    📏 Max Size: {} MB",
            source.max_file_size / (1024 * 1024)
        );
        println!("    ⏱️ Timeout: {}s", source.timeout_seconds);
        println!();
    }

    // Test comprehensive domain whitelist
    let domains = vx_download::sources::create_vx_domain_whitelist();
    println!(
        "🌐 Comprehensive Domain Whitelist ({} domains):",
        domains.len()
    );
    for domain in &domains {
        println!("  ✅ {}", domain);
    }

    // Test URL support detection
    println!("\n🔍 URL Support Detection Test:");
    let test_urls = vec![
        ("Node.js Official", "https://nodejs.org/dist/v18.17.0/node-v18.17.0-win-x64.zip"),
        ("Go Official", "https://golang.org/dl/go1.21.0.windows-amd64.zip"),
        ("Python Standalone", "https://github.com/astral-sh/python-build-standalone/releases/download/20230726/cpython-3.11.4+20230726-x86_64-unknown-linux-gnu-install_only.tar.gz"),
        ("Rust Official", "https://static.rust-lang.org/dist/rust-1.70.0-x86_64-pc-windows-msvc.msi"),
        ("Bun GitHub", "https://github.com/oven-sh/bun/releases/download/bun-v1.0.0/bun-windows-x64.zip"),
        ("UV GitHub", "https://github.com/astral-sh/uv/releases/download/0.1.0/uv-x86_64-pc-windows-msvc.zip"),
        ("Maven Central", "https://repo1.maven.org/maven2/org/apache/maven/apache-maven/3.9.0/apache-maven-3.9.0-bin.zip"),
        ("NuGet", "https://api.nuget.org/v3-flatcontainer/newtonsoft.json/13.0.3/newtonsoft.json.13.0.3.nupkg"),
        ("Unsupported", "https://example.com/some-file.zip"),
    ];

    for (source_name, url) in &test_urls {
        let is_supported = registry.is_url_supported(url);
        let status = if is_supported {
            "✅ Supported"
        } else {
            "❌ Not supported"
        };
        println!("  {} {}: {}", status, source_name, url);

        if is_supported {
            if let Some(source) = registry.get_source_for_url(url) {
                println!("    📦 Detected source: {}", source.name);
            }
        }
    }

    // Test download URL building
    println!("\n🔗 Download URL Building Test:");
    let tools_to_test = vec![
        ("nodejs", "18.17.0"),
        ("golang", "1.21.0"),
        ("bun", "1.0.0"),
        ("uv", "0.1.0"),
    ];

    for (tool, version) in tools_to_test {
        match registry.build_download_url(tool, version, "windows-x64") {
            Ok(url) => {
                println!("  ✅ {} {}: {}", tool, version, url);
            }
            Err(e) => {
                println!("  ❌ {} {}: {}", tool, version, e);
            }
        }
    }

    // Test with VxDownloadManager
    println!("\n📥 VxDownloadManager Integration Test:");
    let mut manager = VxDownloadManager::new().await?;
    println!("✅ VxDownloadManager initialized");

    // Test URL validation with extended sources
    println!("\n🔍 Extended URL Validation:");
    for (source_name, url) in test_urls {
        let is_allowed = vx_download::vx_config::is_url_allowed_for_vx(url);
        let status = if is_allowed {
            "✅ Allowed"
        } else {
            "❌ Blocked"
        };
        println!("  {} {}: {}", status, source_name, url);
    }

    // Test build download URL with fallback
    println!("\n🔗 Download URL Building with Fallback:");
    let tools_to_test = vec![
        ("python", "latest"),
        ("node", "latest"),
        ("go", "latest"),
        ("uv", "latest"),
        ("bun", "latest"),
    ];

    for (tool, version) in tools_to_test {
        match manager.build_download_url(tool, version).await {
            Ok(url) => {
                println!("  ✅ {} {}: {}", tool, version, url);
            }
            Err(e) => {
                println!("  ❌ {} {}: {}", tool, version, e);
            }
        }
    }

    // Test URL parsing with extended sources
    println!("\n📋 URL Parsing with Extended Sources:");
    let extended_test_urls = vec![
        "https://nodejs.org/dist/v18.17.0/node-v18.17.0-win-x64.zip",
        "https://golang.org/dl/go1.21.0.windows-amd64.zip",
    ];

    for url in extended_test_urls {
        println!("\n🔗 Testing: {}", url);
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
    }

    println!("\n🎉 VX Source Extension test completed!");
    println!("\n💡 Key Extensions Added:");
    println!("  • Node.js official source (nodejs.org)");
    println!("  • Go official source (golang.org, go.dev)");
    println!("  • Python Build Standalone (GitHub)");
    println!("  • Rust official source (static.rust-lang.org)");
    println!("  • Bun GitHub releases");
    println!("  • UV GitHub releases");
    println!("  • Maven Central repository");
    println!("  • NuGet package registry");
    println!("  • Comprehensive domain whitelist");
    println!("  • Intelligent URL building with platform detection");

    Ok(())
}
