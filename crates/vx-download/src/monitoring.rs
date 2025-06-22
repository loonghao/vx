//! Performance monitoring and metrics for vx downloads

use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tracing::{debug, info, warn};

/// Download performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadMetrics {
    /// Download URL
    pub url: String,
    /// Tool name
    pub tool_name: String,
    /// File size in bytes
    pub file_size: u64,
    /// Download duration
    pub duration: Duration,
    /// Average speed in bytes per second
    pub speed_bps: f64,
    /// Source used (CDN, cache, etc.)
    pub source: String,
    /// Success flag
    pub success: bool,
    /// Error message if failed
    pub error: Option<String>,
    /// Timestamp
    pub timestamp: u64,
    /// Retry count
    pub retry_count: u32,
    /// Cache hit flag
    pub cache_hit: bool,
    /// Geographic region
    pub region: String,
}

/// CDN source performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CdnSourceMetrics {
    /// Source name
    pub source_name: String,
    /// Total requests
    pub total_requests: u64,
    /// Successful requests
    pub successful_requests: u64,
    /// Failed requests
    pub failed_requests: u64,
    /// Average response time in milliseconds
    pub avg_response_time: f64,
    /// Total bytes downloaded
    pub total_bytes: u64,
    /// Average speed in bytes per second
    pub avg_speed: f64,
    /// Last health check timestamp
    pub last_health_check: u64,
    /// Health status
    pub health_status: HealthStatus,
}

/// Health status for CDN sources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

/// Cache performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheMetrics {
    /// Total cache hits
    pub hits: u64,
    /// Total cache misses
    pub misses: u64,
    /// Cache hit rate
    pub hit_rate: f64,
    /// Total cache size in bytes
    pub total_size: u64,
    /// Number of cached entries
    pub entry_count: u64,
    /// Deduplication count
    pub dedup_count: u64,
    /// Space saved through deduplication
    pub dedup_saved_bytes: u64,
    /// Average access time in milliseconds
    pub avg_access_time: f64,
}

/// Overall performance summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSummary {
    /// Total downloads
    pub total_downloads: u64,
    /// Successful downloads
    pub successful_downloads: u64,
    /// Failed downloads
    pub failed_downloads: u64,
    /// Success rate
    pub success_rate: f64,
    /// Average download speed in MB/s
    pub avg_speed_mbps: f64,
    /// Total bytes downloaded
    pub total_bytes: u64,
    /// Total time spent downloading
    pub total_duration: Duration,
    /// Cache metrics
    pub cache_metrics: CacheMetrics,
    /// CDN source metrics
    pub cdn_metrics: HashMap<String, CdnSourceMetrics>,
    /// Tool-specific metrics
    pub tool_metrics: HashMap<String, ToolMetrics>,
}

/// Tool-specific performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolMetrics {
    /// Tool name
    pub tool_name: String,
    /// Total downloads for this tool
    pub downloads: u64,
    /// Average download size
    pub avg_size: u64,
    /// Average download time
    pub avg_duration: Duration,
    /// Success rate
    pub success_rate: f64,
    /// Preferred CDN source
    pub preferred_source: String,
}

/// Performance monitor for tracking download metrics
pub struct PerformanceMonitor {
    /// Download metrics history
    metrics_history: Vec<DownloadMetrics>,
    /// CDN source metrics
    cdn_metrics: HashMap<String, CdnSourceMetrics>,
    /// Cache metrics
    cache_metrics: CacheMetrics,
    /// Start time for session
    session_start: Instant,
    /// Maximum history size
    max_history_size: usize,
}

impl PerformanceMonitor {
    /// Create a new performance monitor
    pub fn new() -> Self {
        Self {
            metrics_history: Vec::new(),
            cdn_metrics: HashMap::new(),
            cache_metrics: CacheMetrics {
                hits: 0,
                misses: 0,
                hit_rate: 0.0,
                total_size: 0,
                entry_count: 0,
                dedup_count: 0,
                dedup_saved_bytes: 0,
                avg_access_time: 0.0,
            },
            session_start: Instant::now(),
            max_history_size: 1000, // Keep last 1000 downloads
        }
    }

    /// Record a download metric
    pub fn record_download(&mut self, metrics: DownloadMetrics) {
        info!(
            "Download recorded: {} - {} bytes in {:?} ({:.2} MB/s)",
            metrics.tool_name,
            metrics.file_size,
            metrics.duration,
            metrics.speed_bps / 1_000_000.0
        );

        // Update CDN source metrics
        self.update_cdn_metrics(&metrics);

        // Add to history
        self.metrics_history.push(metrics);

        // Trim history if needed
        if self.metrics_history.len() > self.max_history_size {
            self.metrics_history.remove(0);
        }

        debug!("Total downloads recorded: {}", self.metrics_history.len());
    }

    /// Update CDN source metrics
    fn update_cdn_metrics(&mut self, metrics: &DownloadMetrics) {
        let source_metrics = self
            .cdn_metrics
            .entry(metrics.source.clone())
            .or_insert_with(|| CdnSourceMetrics {
                source_name: metrics.source.clone(),
                total_requests: 0,
                successful_requests: 0,
                failed_requests: 0,
                avg_response_time: 0.0,
                total_bytes: 0,
                avg_speed: 0.0,
                last_health_check: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                health_status: HealthStatus::Unknown,
            });

        source_metrics.total_requests += 1;
        if metrics.success {
            source_metrics.successful_requests += 1;
            source_metrics.total_bytes += metrics.file_size;

            // Update average speed
            let total_successful = source_metrics.successful_requests as f64;
            source_metrics.avg_speed = (source_metrics.avg_speed * (total_successful - 1.0)
                + metrics.speed_bps)
                / total_successful;

            // Update average response time
            let response_time = metrics.duration.as_millis() as f64;
            source_metrics.avg_response_time =
                (source_metrics.avg_response_time * (total_successful - 1.0) + response_time)
                    / total_successful;
        } else {
            source_metrics.failed_requests += 1;
        }

        // Update health status based on recent performance
        // Calculate health status after updating metrics to avoid borrowing issues
        let source_name = metrics.source.clone();
        drop(source_metrics); // Release the mutable borrow
        let health_status = self.calculate_health_status(&source_name);
        self.cdn_metrics
            .get_mut(&source_name)
            .unwrap()
            .health_status = health_status;
    }

    /// Calculate health status for a CDN source
    fn calculate_health_status(&self, source: &str) -> HealthStatus {
        if let Some(metrics) = self.cdn_metrics.get(source) {
            let success_rate = if metrics.total_requests > 0 {
                metrics.successful_requests as f64 / metrics.total_requests as f64
            } else {
                0.0
            };

            match success_rate {
                rate if rate >= 0.95 => HealthStatus::Healthy,
                rate if rate >= 0.80 => HealthStatus::Degraded,
                rate if rate < 0.80 => HealthStatus::Unhealthy,
                _ => HealthStatus::Unknown,
            }
        } else {
            HealthStatus::Unknown
        }
    }

    /// Record cache hit
    pub fn record_cache_hit(&mut self, access_time: Duration) {
        self.cache_metrics.hits += 1;
        self.update_cache_hit_rate();

        // Update average access time
        let total_accesses = self.cache_metrics.hits + self.cache_metrics.misses;
        let access_time_ms = access_time.as_millis() as f64;
        self.cache_metrics.avg_access_time =
            (self.cache_metrics.avg_access_time * (total_accesses - 1) as f64 + access_time_ms)
                / total_accesses as f64;

        debug!(
            "Cache hit recorded, hit rate: {:.1}%",
            self.cache_metrics.hit_rate * 100.0
        );
    }

    /// Record cache miss
    pub fn record_cache_miss(&mut self) {
        self.cache_metrics.misses += 1;
        self.update_cache_hit_rate();
        debug!(
            "Cache miss recorded, hit rate: {:.1}%",
            self.cache_metrics.hit_rate * 100.0
        );
    }

    /// Update cache hit rate
    fn update_cache_hit_rate(&mut self) {
        let total = self.cache_metrics.hits + self.cache_metrics.misses;
        self.cache_metrics.hit_rate = if total > 0 {
            self.cache_metrics.hits as f64 / total as f64
        } else {
            0.0
        };
    }

    /// Update cache size metrics
    pub fn update_cache_size(
        &mut self,
        total_size: u64,
        entry_count: u64,
        dedup_count: u64,
        dedup_saved_bytes: u64,
    ) {
        self.cache_metrics.total_size = total_size;
        self.cache_metrics.entry_count = entry_count;
        self.cache_metrics.dedup_count = dedup_count;
        self.cache_metrics.dedup_saved_bytes = dedup_saved_bytes;
    }

    /// Generate performance summary
    pub fn generate_summary(&self) -> PerformanceSummary {
        let total_downloads = self.metrics_history.len() as u64;
        let successful_downloads = self.metrics_history.iter().filter(|m| m.success).count() as u64;
        let failed_downloads = total_downloads - successful_downloads;

        let success_rate = if total_downloads > 0 {
            successful_downloads as f64 / total_downloads as f64
        } else {
            0.0
        };

        let total_bytes: u64 = self
            .metrics_history
            .iter()
            .filter(|m| m.success)
            .map(|m| m.file_size)
            .sum();

        let total_duration: Duration = self
            .metrics_history
            .iter()
            .filter(|m| m.success)
            .map(|m| m.duration)
            .sum();

        let avg_speed_bps = if successful_downloads > 0 {
            self.metrics_history
                .iter()
                .filter(|m| m.success)
                .map(|m| m.speed_bps)
                .sum::<f64>()
                / successful_downloads as f64
        } else {
            0.0
        };

        let avg_speed_mbps = avg_speed_bps / 1_000_000.0;

        // Generate tool-specific metrics
        let mut tool_metrics = HashMap::new();
        for tool_name in self
            .metrics_history
            .iter()
            .map(|m| &m.tool_name)
            .collect::<std::collections::HashSet<_>>()
        {
            let tool_downloads: Vec<_> = self
                .metrics_history
                .iter()
                .filter(|m| &m.tool_name == tool_name)
                .collect();

            let tool_successful = tool_downloads.iter().filter(|m| m.success).count() as u64;
            let tool_total = tool_downloads.len() as u64;

            let tool_success_rate = if tool_total > 0 {
                tool_successful as f64 / tool_total as f64
            } else {
                0.0
            };

            let avg_size = if tool_successful > 0 {
                tool_downloads
                    .iter()
                    .filter(|m| m.success)
                    .map(|m| m.file_size)
                    .sum::<u64>()
                    / tool_successful
            } else {
                0
            };

            let avg_duration = if tool_successful > 0 {
                let total_duration: Duration = tool_downloads
                    .iter()
                    .filter(|m| m.success)
                    .map(|m| m.duration)
                    .sum();
                total_duration / tool_successful as u32
            } else {
                Duration::from_secs(0)
            };

            // Find preferred source (most used successful source)
            let mut source_counts = HashMap::new();
            for download in tool_downloads.iter().filter(|m| m.success) {
                *source_counts.entry(&download.source).or_insert(0) += 1;
            }
            let preferred_source = source_counts
                .iter()
                .max_by_key(|(_, count)| *count)
                .map(|(source, _)| source.to_string())
                .unwrap_or_else(|| "unknown".to_string());

            tool_metrics.insert(
                tool_name.clone(),
                ToolMetrics {
                    tool_name: tool_name.clone(),
                    downloads: tool_total,
                    avg_size,
                    avg_duration,
                    success_rate: tool_success_rate,
                    preferred_source,
                },
            );
        }

        PerformanceSummary {
            total_downloads,
            successful_downloads,
            failed_downloads,
            success_rate,
            avg_speed_mbps,
            total_bytes,
            total_duration,
            cache_metrics: self.cache_metrics.clone(),
            cdn_metrics: self.cdn_metrics.clone(),
            tool_metrics,
        }
    }

    /// Generate performance report
    pub fn generate_report(&self) -> String {
        let summary = self.generate_summary();
        let session_duration = self.session_start.elapsed();

        format!(
            r#"
📊 VX Download Performance Report
================================

⏱️ Session Duration: {:.1} minutes
📥 Total Downloads: {}
✅ Successful: {} ({:.1}%)
❌ Failed: {} ({:.1}%)
📊 Average Speed: {:.2} MB/s
💾 Total Data: {:.2} GB

💾 Cache Performance:
  Hit Rate: {:.1}%
  Total Size: {:.2} GB
  Entries: {}
  Deduplication: {} files saved {:.2} GB
  Avg Access Time: {:.1}ms

🌐 CDN Source Performance:
{}

🔧 Tool Performance:
{}

💡 Recommendations:
{}
"#,
            session_duration.as_secs_f64() / 60.0,
            summary.total_downloads,
            summary.successful_downloads,
            summary.success_rate * 100.0,
            summary.failed_downloads,
            (summary.failed_downloads as f64 / summary.total_downloads as f64) * 100.0,
            summary.avg_speed_mbps,
            summary.total_bytes as f64 / 1_000_000_000.0,
            summary.cache_metrics.hit_rate * 100.0,
            summary.cache_metrics.total_size as f64 / 1_000_000_000.0,
            summary.cache_metrics.entry_count,
            summary.cache_metrics.dedup_count,
            summary.cache_metrics.dedup_saved_bytes as f64 / 1_000_000_000.0,
            summary.cache_metrics.avg_access_time,
            self.format_cdn_metrics(&summary.cdn_metrics),
            self.format_tool_metrics(&summary.tool_metrics),
            self.generate_recommendations(&summary)
        )
    }

    /// Format CDN metrics for report
    fn format_cdn_metrics(&self, cdn_metrics: &HashMap<String, CdnSourceMetrics>) -> String {
        if cdn_metrics.is_empty() {
            return "  No CDN data available".to_string();
        }

        cdn_metrics
            .iter()
            .map(|(_, metrics)| {
                format!(
                    "  {} - {} requests ({:.1}% success) - {:.2} MB/s avg",
                    metrics.source_name,
                    metrics.total_requests,
                    (metrics.successful_requests as f64 / metrics.total_requests as f64) * 100.0,
                    metrics.avg_speed / 1_000_000.0
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Format tool metrics for report
    fn format_tool_metrics(&self, tool_metrics: &HashMap<String, ToolMetrics>) -> String {
        if tool_metrics.is_empty() {
            return "  No tool data available".to_string();
        }

        tool_metrics
            .iter()
            .map(|(_, metrics)| {
                format!(
                    "  {} - {} downloads ({:.1}% success) - {:.2} MB avg size",
                    metrics.tool_name,
                    metrics.downloads,
                    metrics.success_rate * 100.0,
                    metrics.avg_size as f64 / 1_000_000.0
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Generate performance recommendations
    fn generate_recommendations(&self, summary: &PerformanceSummary) -> String {
        let mut recommendations: Vec<String> = Vec::new();

        // Cache recommendations
        if summary.cache_metrics.hit_rate < 0.3 {
            recommendations
                .push("• Consider increasing cache size or TTL for better hit rates".to_string());
        }

        // Speed recommendations
        if summary.avg_speed_mbps < 5.0 {
            recommendations.push(
                "• Network speed is below optimal, consider using different CDN sources"
                    .to_string(),
            );
        }

        // Success rate recommendations
        if summary.success_rate < 0.95 {
            recommendations.push(
                "• Download success rate is below 95%, check network connectivity".to_string(),
            );
        }

        // CDN recommendations
        for (_, cdn_metrics) in &summary.cdn_metrics {
            let success_rate =
                cdn_metrics.successful_requests as f64 / cdn_metrics.total_requests as f64;
            if success_rate < 0.9 {
                let recommendation = format!(
                    "• {} source has low success rate, consider avoiding",
                    cdn_metrics.source_name
                );
                recommendations.push(recommendation);
            }
        }

        if recommendations.is_empty() {
            "• Performance is optimal, no recommendations".to_string()
        } else {
            recommendations.join("\n")
        }
    }

    /// Get recent download metrics
    pub fn get_recent_metrics(&self, count: usize) -> Vec<&DownloadMetrics> {
        let start = if self.metrics_history.len() > count {
            self.metrics_history.len() - count
        } else {
            0
        };
        self.metrics_history[start..].iter().collect()
    }

    /// Export metrics to JSON
    pub fn export_metrics(&self) -> Result<String> {
        let summary = self.generate_summary();
        serde_json::to_string_pretty(&summary).map_err(|e| {
            crate::error::DownloadError::config(format!("Failed to serialize metrics: {}", e))
        })
    }
}

impl Default for PerformanceMonitor {
    fn default() -> Self {
        Self::new()
    }
}
