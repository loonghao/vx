//! VX-specific configuration for turbo-cdn

use crate::error::Result;
use std::collections::HashMap;
use std::time::Duration;
use turbo_cdn::TurboCdnConfig;

/// Create a vx-optimized turbo-cdn configuration
pub fn create_vx_turbo_config() -> Result<TurboCdnConfig> {
    let mut config = TurboCdnConfig::default();

    // 🔧 Meta configuration
    config.meta.version = "1.0".to_string();
    config.meta.schema_version = "2025.1".to_string();

    // 🌐 General settings - optimized for development tools
    config.general.enabled = true;
    config.general.debug_mode = false;
    config.general.max_concurrent_downloads = 8;
    config.general.default_region = "Global".to_string();

    // 🚀 Performance settings - aggressive optimization
    config.performance.max_concurrent_downloads = 8;
    config.performance.chunk_size = "4MB".to_string();
    config.performance.timeout = Duration::from_secs(300); // 5 minutes
    config.performance.retry_attempts = 3;

    // 💾 Cache settings - aggressive caching for development tools
    config.performance.cache.enabled = true;
    config.performance.cache.max_size = "10GB".to_string();
    config.performance.cache.ttl = Duration::from_secs(7 * 24 * 60 * 60); // 7 days

    // 🔒 Security settings - relaxed for vx tools
    config.security.verify_ssl = true;
    config.security.verify_checksums = false; // Disable for faster downloads
    config.security.user_agent = "vx-download/0.4.1 (https://github.com/loonghao/vx)".to_string();
    config.security.allowed_protocols = vec!["https".to_string(), "http".to_string()];

    // 🌍 Region settings
    config.regions.default = "Global".to_string();
    config.regions.auto_detect = true;
    // Note: fallback_order field doesn't exist in turbo-cdn 0.2.0

    // 📊 Logging settings
    config.logging.level = "info".to_string();
    config.logging.format = "json".to_string();
    config.logging.audit_enabled = false; // Disable for performance

    // 🔗 Mirror configurations - simplified for turbo-cdn 0.2.0
    // Note: Mirror configs structure has changed in 0.2.0

    Ok(config)
}

/// Create vx-specific mirror configurations
fn create_vx_mirror_configs() -> HashMap<String, serde_json::Value> {
    let mut configs = HashMap::new();

    // GitHub configuration - relaxed for vx tools
    configs.insert(
        "github".to_string(),
        serde_json::json!({
            "enabled": true,
            "base_url": "https://github.com",
            "api_url": "https://api.github.com",
            "timeout": "60s",
            "rate_limit": 5000,
            "user_agent": "vx-download/0.4.1",
            "verify_ssl": true,
            "allowed_file_types": ["zip", "tar.gz", "exe", "dmg", "pkg", "deb", "rpm"],
            "max_file_size": "2GB"
        }),
    );

    // jsDelivr configuration
    configs.insert(
        "jsdelivr".to_string(),
        serde_json::json!({
            "enabled": true,
            "base_url": "https://cdn.jsdelivr.net",
            "timeout": "30s",
            "verify_ssl": true,
            "allowed_file_types": ["js", "css", "json", "zip", "tar.gz"],
            "max_file_size": "100MB"
        }),
    );

    // Fastly configuration
    configs.insert(
        "fastly".to_string(),
        serde_json::json!({
            "enabled": true,
            "base_url": "https://fastly.jsdelivr.net",
            "timeout": "30s",
            "verify_ssl": true,
            "allowed_file_types": ["js", "css", "json", "zip", "tar.gz"],
            "max_file_size": "100MB"
        }),
    );

    // Cloudflare configuration
    configs.insert(
        "cloudflare".to_string(),
        serde_json::json!({
            "enabled": true,
            "base_url": "https://cdnjs.cloudflare.com",
            "timeout": "30s",
            "verify_ssl": true,
            "allowed_file_types": ["js", "css", "json"],
            "max_file_size": "50MB"
        }),
    );

    // VX-specific sources for development tools
    configs.insert(
        "nodejs".to_string(),
        serde_json::json!({
            "enabled": true,
            "base_url": "https://nodejs.org",
            "timeout": "120s",
            "verify_ssl": true,
            "allowed_file_types": ["zip", "tar.gz", "msi", "pkg"],
            "max_file_size": "50MB"
        }),
    );

    configs.insert(
        "golang".to_string(),
        serde_json::json!({
            "enabled": true,
            "base_url": "https://golang.org",
            "timeout": "120s",
            "verify_ssl": true,
            "allowed_file_types": ["zip", "tar.gz"],
            "max_file_size": "200MB"
        }),
    );

    configs.insert(
        "python_standalone".to_string(),
        serde_json::json!({
            "enabled": true,
            "base_url": "https://github.com/astral-sh/python-build-standalone",
            "timeout": "300s",
            "verify_ssl": true,
            "allowed_file_types": ["tar.gz", "zip"],
            "max_file_size": "100MB"
        }),
    );

    configs
}

/// Create relaxed compliance configuration for vx tools
pub fn create_vx_compliance_config() -> serde_json::Value {
    serde_json::json!({
        "strict_mode": false,
        "validate_source": false,
        "verify_open_source": false,
        "data_protection": {
            "user_consent_required": false,
            "data_retention_days": 30,
            "anonymize_data": true
        },
        "audit_logging": false,
        "allowed_domains": [
            "github.com",
            "githubusercontent.com",
            "cdn.jsdelivr.net",
            "fastly.jsdelivr.net",
            "cdnjs.cloudflare.com",
            "nodejs.org",
            "golang.org",
            "go.dev",
            "files.pythonhosted.org",
            "pypi.org",
            "crates.io",
            "registry.npmjs.org",
            "repo1.maven.org",
            "api.nuget.org",
            "forge.rust-lang.org",
            "static.rust-lang.org"
        ],
        "blocked_patterns": [],
        "whitelist_mode": true
    })
}

/// Apply vx-specific overrides to turbo-cdn config
pub fn apply_vx_overrides(config: &mut TurboCdnConfig) -> Result<()> {
    // Override security settings for vx tools
    config.security.verify_checksums = false;
    config.security.user_agent = "vx-download/0.4.1 (https://github.com/loonghao/vx)".to_string();

    // Override performance settings
    config.performance.max_concurrent_downloads = 8;
    config.performance.chunk_size = "4MB".to_string();
    config.performance.timeout = Duration::from_secs(300);

    // Override cache settings using vx-paths
    config.performance.cache.enabled = true;
    config.performance.cache.max_size = "10GB".to_string();
    config.performance.cache.ttl = Duration::from_secs(7 * 24 * 60 * 60);

    // Use vx-paths for cache directory
    if let Ok(cache_dir) = vx_paths::get_turbo_cdn_cache_dir() {
        config.performance.cache.directory = cache_dir.to_string_lossy().to_string();
    }

    // Override logging settings using vx-paths
    config.logging.audit_enabled = true; // Enable audit logging
    config.logging.level = "info".to_string();

    // Use vx-paths for log directory
    if let Ok(log_file) = vx_paths::get_turbo_cdn_audit_log() {
        config.logging.audit_file = log_file.to_string_lossy().to_string();
    }

    Ok(())
}

/// Get vx-optimized download sources
pub fn get_vx_download_sources() -> Vec<String> {
    vec![
        "github".to_string(),
        "jsdelivr".to_string(),
        "fastly".to_string(),
        "cloudflare".to_string(),
        "nodejs".to_string(),
        "golang".to_string(),
        "python_standalone".to_string(),
    ]
}

/// Check if a URL is allowed for vx tools
pub fn is_url_allowed_for_vx(url: &str) -> bool {
    // Use the comprehensive VX source registry
    let registry = crate::sources::VxSourceRegistry::new();
    registry.is_url_supported(url)
}
