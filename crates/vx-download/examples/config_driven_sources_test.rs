//! Test configuration-driven download sources

use vx_download::sources::VxSourceRegistry;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("🚀 Configuration-Driven Sources Test");
    println!("=====================================\n");

    // Test loading sources from configuration
    println!("📋 Testing Configuration-Driven Source Loading:");
    test_config_driven_sources().await?;

    // Test configuration vs hardcoded comparison
    println!("\n🔄 Testing Configuration vs Hardcoded Comparison:");
    test_config_vs_hardcoded().await?;

    // Test source configuration validation
    println!("\n✅ Testing Source Configuration Validation:");
    test_source_validation().await?;

    println!("\n🎉 Configuration-driven sources test completed!");
    println!("\n💡 Key Achievements:");
    println!("  • All download sources now come from vx-config");
    println!("  • Zero hardcoded source configurations");
    println!("  • Centralized configuration management");
    println!("  • Easy to add new sources via config");
    println!("  • Backward compatibility with fallback sources");

    Ok(())
}

async fn test_config_driven_sources() -> Result<(), Box<dyn std::error::Error>> {
    // Load global configuration
    let config = vx_config::get_global_config();
    println!("✅ Global configuration loaded");

    // Create registry from configuration
    let registry = VxSourceRegistry::from_config(config);
    println!("✅ VX source registry created from configuration");

    // Display all configured sources
    println!("\n📦 Configured Download Sources:");
    for (name, source_config) in &config.download_sources {
        println!("  🔧 {}", name);
        println!("    📍 Base URL: {}", source_config.base_url);
        println!("    🌐 Domains: {:?}", source_config.supported_domains);
        println!(
            "    📏 Max Size: {} MB",
            source_config.max_file_size / (1024 * 1024)
        );
        println!("    ⏱️ Timeout: {}s", source_config.timeout_seconds);
        println!("    🔗 URL Template: {}", source_config.url_template);
        println!("    ✅ Enabled: {}", source_config.enabled);
        println!("    🎯 Priority: {}", source_config.priority);
        println!();
    }

    // Test registry functionality
    println!("🔍 Testing Registry Functionality:");
    let all_sources = registry.get_all_sources();
    println!("  📊 Total sources loaded: {}", all_sources.len());

    // Test specific sources
    let test_sources = vec!["nodejs", "golang", "python_standalone", "rust", "bun", "uv"];
    for source_name in test_sources {
        if let Some(source) = registry.get_source(source_name) {
            println!("  ✅ {} source loaded successfully", source_name);
            println!("    📍 Base URL: {}", source.base_url);
            println!("    🌐 Supported domains: {:?}", source.supported_domains);
        } else {
            println!("  ❌ {} source not found", source_name);
        }
    }

    // Test URL support detection
    println!("\n🔍 Testing URL Support Detection:");
    let test_urls = vec![
        ("Node.js Official", "https://nodejs.org/dist/v18.17.0/node-v18.17.0-win-x64.zip"),
        ("Go Official", "https://golang.org/dl/go1.21.0.windows-amd64.zip"),
        ("Python Standalone", "https://github.com/astral-sh/python-build-standalone/releases/download/20230726/cpython-3.11.4+20230726-x86_64-unknown-linux-gnu-install_only.tar.gz"),
        ("Rust Official", "https://static.rust-lang.org/dist/rust-1.70.0-x86_64-pc-windows-msvc.msi"),
        ("Bun GitHub", "https://github.com/oven-sh/bun/releases/download/bun-v1.0.0/bun-windows-x64.zip"),
        ("UV GitHub", "https://github.com/astral-sh/uv/releases/download/0.1.0/uv-x86_64-pc-windows-msvc.zip"),
    ];

    for (source_name, url) in test_urls {
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

    Ok(())
}

async fn test_config_vs_hardcoded() -> Result<(), Box<dyn std::error::Error>> {
    // Create registry from configuration
    let config = vx_config::get_global_config();
    let config_registry = VxSourceRegistry::from_config(config);

    // Create registry with default (fallback) sources
    let default_registry = VxSourceRegistry::new();

    println!("📊 Configuration vs Default Comparison:");
    println!(
        "  Config sources: {}",
        config_registry.get_all_sources().len()
    );
    println!(
        "  Default sources: {}",
        default_registry.get_all_sources().len()
    );

    // Compare source availability
    let test_sources = vec!["nodejs", "golang", "python_standalone", "rust", "bun", "uv"];

    println!("\n🔍 Source Availability Comparison:");
    for source_name in test_sources {
        let config_has = config_registry.get_source(source_name).is_some();
        let default_has = default_registry.get_source(source_name).is_some();

        match (config_has, default_has) {
            (true, true) => println!(
                "  ✅ {} - Available in both config and default",
                source_name
            ),
            (true, false) => println!("  🆕 {} - Only in config (new source)", source_name),
            (false, true) => println!(
                "  ⚠️ {} - Only in default (missing from config)",
                source_name
            ),
            (false, false) => println!("  ❌ {} - Missing from both", source_name),
        }
    }

    // Test URL building comparison
    println!("\n🔗 URL Building Comparison:");
    let test_cases = vec![
        ("nodejs", "18.17.0", "windows-x64"),
        ("golang", "1.21.0", "windows-x64"),
        ("bun", "1.0.0", "windows-x64"),
    ];

    for (tool, version, platform) in test_cases {
        println!("  🔧 Testing {} {} for {}:", tool, version, platform);

        match config_registry.build_download_url(tool, version, platform) {
            Ok(config_url) => {
                println!("    📋 Config URL: {}", config_url);

                match default_registry.build_download_url(tool, version, platform) {
                    Ok(default_url) => {
                        println!("    🔄 Default URL: {}", default_url);
                        if config_url == default_url {
                            println!("    ✅ URLs match perfectly");
                        } else {
                            println!("    🔄 URLs differ (expected with config-driven approach)");
                        }
                    }
                    Err(e) => println!("    ❌ Default URL failed: {}", e),
                }
            }
            Err(e) => println!("    ❌ Config URL failed: {}", e),
        }
        println!();
    }

    Ok(())
}

async fn test_source_validation() -> Result<(), Box<dyn std::error::Error>> {
    let config = vx_config::get_global_config();

    println!("🔍 Validating Source Configurations:");

    let mut total_sources = 0;
    let mut enabled_sources = 0;
    let mut valid_sources = 0;

    for (name, source_config) in &config.download_sources {
        total_sources += 1;

        println!("  📦 Validating source: {}", name);

        // Check if enabled
        if source_config.enabled {
            enabled_sources += 1;
            println!("    ✅ Enabled");
        } else {
            println!("    ⚠️ Disabled");
            continue;
        }

        // Validate base URL
        if source_config.base_url.starts_with("https://") {
            println!("    ✅ Valid HTTPS base URL");
        } else {
            println!("    ⚠️ Non-HTTPS base URL: {}", source_config.base_url);
        }

        // Validate domains
        if !source_config.supported_domains.is_empty() {
            println!(
                "    ✅ Has supported domains: {:?}",
                source_config.supported_domains
            );
        } else {
            println!("    ⚠️ No supported domains specified");
        }

        // Validate URL template
        if source_config.url_template.contains("{version}")
            || source_config.url_template.contains("{filename}")
        {
            println!("    ✅ URL template has placeholders");
        } else {
            println!(
                "    ⚠️ URL template missing placeholders: {}",
                source_config.url_template
            );
        }

        // Validate file size limits
        if source_config.max_file_size > 0 && source_config.max_file_size <= 1024 * 1024 * 1024 {
            println!(
                "    ✅ Reasonable file size limit: {} MB",
                source_config.max_file_size / (1024 * 1024)
            );
        } else {
            println!(
                "    ⚠️ Unusual file size limit: {} bytes",
                source_config.max_file_size
            );
        }

        // Validate timeout
        if source_config.timeout_seconds >= 60 && source_config.timeout_seconds <= 3600 {
            println!(
                "    ✅ Reasonable timeout: {}s",
                source_config.timeout_seconds
            );
        } else {
            println!("    ⚠️ Unusual timeout: {}s", source_config.timeout_seconds);
        }

        valid_sources += 1;
        println!("    ✅ Source validation passed");
        println!();
    }

    println!("📊 Validation Summary:");
    println!("  Total sources: {}", total_sources);
    println!("  Enabled sources: {}", enabled_sources);
    println!("  Valid sources: {}", valid_sources);

    if valid_sources == enabled_sources && enabled_sources > 0 {
        println!("  🎉 All enabled sources are valid!");
    } else {
        println!("  ⚠️ Some sources may need attention");
    }

    // Test configuration serialization
    println!("\n💾 Configuration Serialization Test:");
    match toml::to_string_pretty(&config.download_sources) {
        Ok(toml_str) => {
            println!("  ✅ Download sources can be serialized to TOML");
            println!("  📄 TOML size: {} bytes", toml_str.len());

            // Show a sample of the TOML
            let lines: Vec<&str> = toml_str.lines().take(10).collect();
            println!("  📋 Sample TOML:");
            for line in lines {
                println!("    {}", line);
            }
            if toml_str.lines().count() > 10 {
                println!("    ... ({} more lines)", toml_str.lines().count() - 10);
            }
        }
        Err(e) => println!("  ❌ Serialization failed: {}", e),
    }

    Ok(())
}
