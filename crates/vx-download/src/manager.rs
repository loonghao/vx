//! Unified download manager implementation

use crate::cache::DownloadCache;
use crate::error::{DownloadError, Result};
use crate::monitoring::{DownloadMetrics, PerformanceMonitor};
use crate::progress::ProgressCallback;
use crate::smart_cache::SmartCacheManager;
use std::path::Path;
use std::time::Duration;
use tracing::{debug, info, warn};
use turbo_cdn::{DownloadOptions, DownloadResult, Region, Source, TurboCdn, TurboCdnConfig};

/// Unified download manager for vx tools
pub struct VxDownloadManager {
    /// Turbo CDN client
    turbo_cdn: TurboCdn,
    /// Download cache (legacy)
    cache: DownloadCache,
    /// Smart cache manager
    smart_cache: SmartCacheManager,
    /// Performance monitor
    performance_monitor: PerformanceMonitor,
    /// Configuration
    config: vx_config::types::TurboCdnConfig,
}

impl VxDownloadManager {
    /// Create a new download manager
    pub async fn new() -> Result<Self> {
        let config = vx_config::get_global_config().turbo_cdn.clone();

        // Create turbo-cdn configuration optimized for vx tools
        let mut turbo_config = crate::vx_config::create_vx_turbo_config()?;

        // Apply vx-specific overrides
        crate::vx_config::apply_vx_overrides(&mut turbo_config)?;

        // Get vx-optimized sources
        let _vx_sources = crate::vx_config::get_vx_download_sources();

        let turbo_cdn = TurboCdn::builder()
            .with_config(turbo_config)
            .with_sources(&[
                Source::github(),
                Source::jsdelivr(),
                Source::fastly(),
                Source::cloudflare(),
            ])
            .build()
            .await
            .map_err(|e| DownloadError::config(format!("Failed to initialize TurboCdn: {}", e)))?;

        // Initialize legacy cache
        let cache = DownloadCache::new(&config)?;

        // Initialize smart cache using vx-config
        let smart_cache = SmartCacheManager::new(config.smart_cache.clone())?;

        // Initialize performance monitor
        let performance_monitor = PerformanceMonitor::new();

        info!(
            "VxDownloadManager initialized with turbo-cdn, smart cache, and performance monitoring"
        );
        debug!("Cache enabled: {}", config.cache_enabled);
        debug!("Max concurrent chunks: {}", config.max_concurrent_chunks);
        debug!("Smart cache deduplication: enabled");
        debug!("Performance monitoring: enabled");

        Ok(Self {
            turbo_cdn,
            cache,
            smart_cache,
            performance_monitor,
            config,
        })
    }

    /// Create vx-optimized turbo-cdn configuration with relaxed compliance
    fn create_vx_optimized_config(
        config: &vx_config::types::TurboCdnConfig,
    ) -> Result<TurboCdnConfig> {
        let mut turbo_config = TurboCdnConfig::default();

        // 🔧 Security settings - relaxed for vx tools
        turbo_config.security.verify_ssl = true;
        turbo_config.security.verify_checksums = false; // Disable for faster downloads
        turbo_config.security.user_agent =
            "vx-download/0.4.1 (https://github.com/loonghao/vx)".to_string();
        turbo_config.security.allowed_protocols = vec!["https".to_string(), "http".to_string()];

        // 🚀 Performance settings - optimized for development tools
        turbo_config.performance.max_concurrent_downloads = config.max_concurrent_chunks as usize;
        turbo_config.performance.chunk_size = "4MB".to_string(); // Larger chunks for better performance
        turbo_config.performance.timeout = Duration::from_secs(300); // 5 minutes timeout
        turbo_config.performance.retry_attempts = config.max_retries as usize;

        // 💾 Cache settings - aggressive caching for development tools
        turbo_config.performance.cache.enabled = config.cache_enabled;
        turbo_config.performance.cache.max_size =
            format!("{}GB", config.cache_max_size / (1024 * 1024 * 1024));
        turbo_config.performance.cache.ttl = Duration::from_secs(7 * 24 * 60 * 60); // 7 days cache TTL

        // 🌍 Region settings - auto-detect with fallback
        turbo_config.regions.default = Self::parse_region(&config.default_region).to_string();
        turbo_config.regions.auto_detect = true;

        // 📊 Logging settings - detailed for debugging
        turbo_config.logging.level = "info".to_string();
        turbo_config.logging.format = "json".to_string();
        turbo_config.logging.audit_enabled = false; // Disable audit for performance

        // 🔓 Relaxed compliance for vx tools - THIS IS THE KEY CHANGE
        // Note: These would need to be added to turbo-cdn's config structure
        // For now, we'll use the default and hope turbo-cdn 0.2.0 is more permissive

        Ok(turbo_config)
    }

    /// Parse region string to Region enum
    fn parse_region(region_str: &str) -> Region {
        match region_str.to_lowercase().as_str() {
            "china" => Region::China,
            "us" | "north_america" => Region::Global,
            "eu" | "europe" => Region::Global,
            "asia" | "asia_pacific" => Region::Global,
            _ => Region::Global,
        }
    }

    /// Download a tool from URL with optimization
    pub async fn download_tool(
        &mut self,
        tool_name: &str,
        version: &str,
        url: &str,
        output_path: &Path,
        progress_callback: Option<ProgressCallback>,
    ) -> Result<DownloadResult> {
        let start_time = std::time::Instant::now();
        info!("Downloading {} {} from {}", tool_name, version, url);

        // Validate URL for vx tools
        if !crate::vx_config::is_url_allowed_for_vx(url) {
            warn!(
                "URL {} is not in vx allowed domains, but proceeding anyway",
                url
            );
        }

        // Check smart cache first
        if let Some(cached_path) = self.smart_cache.get(url) {
            let access_time = start_time.elapsed();
            info!("Found smart cached download for {}", url);
            let file_size = cached_path.metadata().map(|m| m.len()).unwrap_or(0);

            // Record cache hit
            self.performance_monitor.record_cache_hit(access_time);

            // Record download metrics
            let metrics = DownloadMetrics {
                url: url.to_string(),
                tool_name: tool_name.to_string(),
                file_size,
                duration: access_time,
                speed_bps: 0.0, // Instant from cache
                source: "smart-cache".to_string(),
                success: true,
                error: None,
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                retry_count: 0,
                cache_hit: true,
                region: self.config.default_region.clone(),
            };
            self.performance_monitor.record_download(metrics);

            return Ok(DownloadResult {
                path: cached_path,
                size: file_size,
                speed: 0.0,
                duration: access_time,
                source: "smart-cache".to_string(),
                url: url.to_string(),
                from_cache: true,
                checksum: None,
            });
        }

        // Fallback to legacy cache
        if let Some(cached_path) = self.cache.get(url) {
            info!("Found legacy cached download for {}", url);
            return Ok(DownloadResult {
                path: cached_path,
                size: 0, // Will be filled by actual file size
                speed: 0.0,
                duration: std::time::Duration::from_secs(0),
                source: "legacy-cache".to_string(),
                url: url.to_string(),
                from_cache: true,
                checksum: None,
            });
        }

        // Create download options
        let options = self.create_download_options(progress_callback)?;

        // Download using turbo-cdn's new API
        let result = self
            .turbo_cdn
            .download_from_url(url, Some(options))
            .await
            .map_err(|e| DownloadError::network(format!("Download failed: {}", e)))?;

        // Move file to target location if different
        let final_path = if result.path != output_path {
            std::fs::create_dir_all(output_path.parent().unwrap_or(Path::new("."))).map_err(
                |e| DownloadError::filesystem(format!("Failed to create directory: {}", e)),
            )?;

            std::fs::copy(&result.path, output_path)
                .map_err(|e| DownloadError::filesystem(format!("Failed to copy file: {}", e)))?;

            output_path.to_path_buf()
        } else {
            result.path.clone()
        };

        // Add to cache
        self.cache.put(url, &final_path, None)?;

        info!(
            "Successfully downloaded {} {} ({:.2} MB/s)",
            tool_name,
            version,
            result.speed / 1_000_000.0
        );

        // Record cache miss and download metrics
        self.performance_monitor.record_cache_miss();
        let download_metrics = DownloadMetrics {
            url: url.to_string(),
            tool_name: tool_name.to_string(),
            file_size: result.size,
            duration: result.duration,
            speed_bps: result.speed,
            source: result.source.clone(),
            success: true,
            error: None,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            retry_count: 0, // TODO: Track actual retries
            cache_hit: result.from_cache,
            region: self.config.default_region.clone(),
        };
        self.performance_monitor.record_download(download_metrics);

        Ok(DownloadResult {
            path: final_path,
            size: result.size,
            speed: result.speed,
            duration: result.duration,
            source: result.source,
            url: result.url,
            from_cache: result.from_cache,
            checksum: result.checksum,
        })
    }

    /// Get optimal URL for download
    pub async fn get_optimal_url(&self, url: &str) -> Result<String> {
        self.turbo_cdn
            .get_optimal_url(url)
            .await
            .map_err(|e| DownloadError::network(format!("Failed to get optimal URL: {}", e)))
    }

    /// Parse URL information
    pub fn parse_url(&self, url: &str) -> Result<turbo_cdn::ParsedUrl> {
        self.turbo_cdn
            .parse_url(url)
            .map_err(|e| DownloadError::invalid_url(url, &e.to_string()))
    }

    /// Create download options from configuration
    fn create_download_options(
        &self,
        progress_callback: Option<ProgressCallback>,
    ) -> Result<DownloadOptions> {
        let mut options = DownloadOptions::default();

        // Set configuration values using new API
        options.verify_checksum = false; // Disable for faster downloads
        options.use_cache = self.config.cache_enabled;
        options.timeout = Duration::from_secs(300); // 5 minutes default

        // Set progress callback if provided
        if let Some(callback) = progress_callback {
            options.progress_callback = Some(Box::new(move |progress| {
                let info = crate::progress::ProgressInfo {
                    total_bytes: Some(progress.total_size),
                    downloaded_bytes: progress.downloaded_size,
                    speed_bps: progress.speed,
                    percentage: progress.percentage,
                    eta: progress.eta,
                    elapsed: progress.elapsed,
                    filename: "download".to_string(), // turbo-cdn doesn't provide filename
                };
                callback(info);
            }));
        }

        Ok(options)
    }

    /// Get cache statistics (legacy)
    pub fn cache_stats(&self) -> crate::cache::CacheStats {
        self.cache.stats()
    }

    /// Get smart cache statistics
    pub fn smart_cache_stats(&self) -> crate::smart_cache::SmartCacheStats {
        self.smart_cache.stats()
    }

    /// Clear download cache (legacy)
    pub fn clear_cache(&mut self) -> Result<()> {
        self.cache.clear()
    }

    /// Clear smart cache
    pub fn clear_smart_cache(&mut self) -> Result<()> {
        self.smart_cache.clear()
    }

    /// Get performance summary
    pub fn performance_summary(&self) -> crate::monitoring::PerformanceSummary {
        self.performance_monitor.generate_summary()
    }

    /// Generate performance report
    pub fn performance_report(&self) -> String {
        self.performance_monitor.generate_report()
    }

    /// Export performance metrics to JSON
    pub fn export_performance_metrics(&self) -> Result<String> {
        self.performance_monitor.export_metrics()
    }

    /// Update cache size metrics for monitoring
    pub fn update_cache_metrics(&mut self) {
        let smart_stats = self.smart_cache.stats();
        self.performance_monitor.update_cache_size(
            smart_stats.total_size,
            smart_stats.entry_count as u64,
            smart_stats.dedup_count,
            smart_stats.dedup_saved_bytes,
        );
    }

    /// Build download URL for a tool
    pub async fn build_download_url(&self, tool_name: &str, version: &str) -> Result<String> {
        // First try vx-config
        let config = vx_config::get_global_config();
        if let Some(url) = vx_config::get_tool_download_url(&config, tool_name, version) {
            return Ok(url);
        }

        // Fallback to VX source registry
        let registry = crate::sources::VxSourceRegistry::new();
        let platform = self.detect_platform();
        registry.build_download_url(tool_name, version, &platform)
    }

    /// Detect current platform
    fn detect_platform(&self) -> String {
        #[cfg(target_os = "windows")]
        {
            if cfg!(target_arch = "x86_64") {
                "windows-x64".to_string()
            } else {
                "windows-x86".to_string()
            }
        }
        #[cfg(target_os = "macos")]
        {
            if cfg!(target_arch = "aarch64") {
                "darwin-arm64".to_string()
            } else {
                "darwin-x64".to_string()
            }
        }
        #[cfg(target_os = "linux")]
        {
            if cfg!(target_arch = "x86_64") {
                "linux-x64".to_string()
            } else {
                "linux-x86".to_string()
            }
        }
        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        {
            "unknown".to_string()
        }
    }

    /// Check if a tool version is available for download
    pub async fn is_version_available(&self, tool_name: &str, version: &str) -> Result<bool> {
        match self.build_download_url(tool_name, version).await {
            Ok(url) => {
                // Try to get optimal URL to verify availability
                match self.get_optimal_url(&url).await {
                    Ok(_) => Ok(true),
                    Err(_) => Ok(false),
                }
            }
            Err(_) => Ok(false),
        }
    }

    /// Get download speed estimate for a URL
    pub async fn estimate_download_speed(&self, _url: &str) -> Result<f64> {
        // This would use turbo-cdn's speed estimation features
        // For now, return a default estimate
        warn!("Speed estimation not yet implemented, returning default");
        Ok(10_000_000.0) // 10 MB/s default
    }
}
