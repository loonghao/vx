//! Telemetry and metrics collection
//!
//! This module provides telemetry functionality:
//! - Build time tracking
//! - OTLP export
//! - Anonymous metrics

use crate::{BuildTrackingConfig, OtlpConfig, TelemetryConfig};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

/// Telemetry collector
pub struct TelemetryCollector {
    config: TelemetryConfig,
    metrics: Vec<Metric>,
    spans: Vec<Span>,
}

/// Metric data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metric {
    /// Metric name
    pub name: String,
    /// Metric value
    pub value: f64,
    /// Unit
    pub unit: Option<String>,
    /// Timestamp (Unix epoch ms)
    pub timestamp: u64,
    /// Labels
    pub labels: HashMap<String, String>,
}

/// Span for tracing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Span {
    /// Span name
    pub name: String,
    /// Trace ID
    pub trace_id: String,
    /// Span ID
    pub span_id: String,
    /// Parent span ID
    pub parent_span_id: Option<String>,
    /// Start time (Unix epoch ms)
    pub start_time: u64,
    /// End time (Unix epoch ms)
    pub end_time: Option<u64>,
    /// Duration in milliseconds
    pub duration_ms: Option<u64>,
    /// Attributes
    pub attributes: HashMap<String, String>,
    /// Status
    pub status: SpanStatus,
}

/// Span status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum SpanStatus {
    #[default]
    Unset,
    Ok,
    Error,
}

/// Build timing data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildTiming {
    /// Operation name
    pub operation: String,
    /// Duration in milliseconds
    pub duration_ms: u64,
    /// Start time
    pub started_at: String,
    /// End time
    pub ended_at: String,
    /// Success
    pub success: bool,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl TelemetryCollector {
    /// Create a new telemetry collector
    pub fn new(config: TelemetryConfig) -> Self {
        Self {
            config,
            metrics: Vec::new(),
            spans: Vec::new(),
        }
    }

    /// Check if telemetry is enabled
    pub fn is_enabled(&self) -> bool {
        self.config.enabled.unwrap_or(false)
    }

    /// Check if anonymous mode is enabled
    pub fn is_anonymous(&self) -> bool {
        self.config.anonymous.unwrap_or(true)
    }

    /// Record a metric
    pub fn record_metric(&mut self, name: &str, value: f64, labels: HashMap<String, String>) {
        if !self.is_enabled() {
            return;
        }

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        self.metrics.push(Metric {
            name: name.to_string(),
            value,
            unit: None,
            timestamp,
            labels,
        });
    }

    /// Start a span
    pub fn start_span(&mut self, name: &str, parent_id: Option<&str>) -> String {
        let span_id = generate_id();
        let trace_id = parent_id.map(|_| generate_id()).unwrap_or_else(generate_id);

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        self.spans.push(Span {
            name: name.to_string(),
            trace_id,
            span_id: span_id.clone(),
            parent_span_id: parent_id.map(String::from),
            start_time: timestamp,
            end_time: None,
            duration_ms: None,
            attributes: HashMap::new(),
            status: SpanStatus::Unset,
        });

        span_id
    }

    /// End a span
    pub fn end_span(&mut self, span_id: &str, status: SpanStatus) {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        if let Some(span) = self.spans.iter_mut().find(|s| s.span_id == span_id) {
            span.end_time = Some(timestamp);
            span.duration_ms = Some(timestamp - span.start_time);
            span.status = status;
        }
    }

    /// Get all metrics
    pub fn get_metrics(&self) -> &[Metric] {
        &self.metrics
    }

    /// Get all spans
    pub fn get_spans(&self) -> &[Span] {
        &self.spans
    }

    /// Clear collected data
    pub fn clear(&mut self) {
        self.metrics.clear();
        self.spans.clear();
    }

    /// Export metrics to JSON
    pub fn export_json(&self) -> String {
        serde_json::json!({
            "metrics": self.metrics,
            "spans": self.spans,
        })
        .to_string()
    }
}

/// Build time tracker
pub struct BuildTracker {
    config: BuildTrackingConfig,
    timings: Vec<BuildTiming>,
}

impl BuildTracker {
    /// Create a new build tracker
    pub fn new(config: BuildTrackingConfig) -> Self {
        Self {
            config,
            timings: Vec::new(),
        }
    }

    /// Check if tracking is enabled
    pub fn is_enabled(&self) -> bool {
        self.config.enabled.unwrap_or(false)
    }

    /// Track an operation
    pub fn track<F, T>(&mut self, operation: &str, f: F) -> T
    where
        F: FnOnce() -> T,
    {
        let start = Instant::now();
        let started_at = chrono::Utc::now().to_rfc3339();

        let result = f();

        let duration = start.elapsed();
        let ended_at = chrono::Utc::now().to_rfc3339();

        if self.is_enabled() {
            self.timings.push(BuildTiming {
                operation: operation.to_string(),
                duration_ms: duration.as_millis() as u64,
                started_at,
                ended_at,
                success: true,
                metadata: HashMap::new(),
            });
        }

        result
    }

    /// Record a timing manually
    pub fn record(&mut self, timing: BuildTiming) {
        if self.is_enabled() {
            self.timings.push(timing);
        }
    }

    /// Get all timings
    pub fn get_timings(&self) -> &[BuildTiming] {
        &self.timings
    }

    /// Get total duration
    pub fn total_duration_ms(&self) -> u64 {
        self.timings.iter().map(|t| t.duration_ms).sum()
    }

    /// Save timings to file
    pub fn save(&self, path: &std::path::Path) -> std::io::Result<()> {
        let content = serde_json::to_string_pretty(&self.timings)?;
        std::fs::write(path, content)
    }

    /// Generate summary report
    pub fn summary(&self) -> String {
        let mut report = String::new();
        report.push_str("# Build Timing Summary\n\n");
        report.push_str(&format!(
            "Total duration: {}ms\n\n",
            self.total_duration_ms()
        ));

        report.push_str("| Operation | Duration | Status |\n");
        report.push_str("|-----------|----------|--------|\n");

        for timing in &self.timings {
            let status = if timing.success { "✓" } else { "✗" };
            report.push_str(&format!(
                "| {} | {}ms | {} |\n",
                timing.operation, timing.duration_ms, status
            ));
        }

        report
    }
}

/// Generate a random ID
fn generate_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("{:016x}", timestamp)
}

/// OTLP exporter (placeholder for actual implementation)
pub struct OtlpExporter {
    config: OtlpConfig,
}

impl OtlpExporter {
    /// Create a new OTLP exporter
    pub fn new(config: OtlpConfig) -> Self {
        Self { config }
    }

    /// Check if export is enabled
    pub fn is_enabled(&self) -> bool {
        self.config.enabled.unwrap_or(false)
    }

    /// Get endpoint URL
    pub fn endpoint(&self) -> Option<&str> {
        self.config.endpoint.as_deref()
    }

    /// Get service name
    pub fn service_name(&self) -> &str {
        self.config.service_name.as_deref().unwrap_or("vx")
    }

    /// Get export interval in seconds
    pub fn interval(&self) -> u32 {
        self.config.interval.unwrap_or(60)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_telemetry_disabled_by_default() {
        let config = TelemetryConfig::default();
        let collector = TelemetryCollector::new(config);
        assert!(!collector.is_enabled());
    }

    #[test]
    fn test_metric_recording() {
        let config = TelemetryConfig {
            enabled: Some(true),
            ..Default::default()
        };
        let mut collector = TelemetryCollector::new(config);

        collector.record_metric("test_metric", 42.0, HashMap::new());
        assert_eq!(collector.get_metrics().len(), 1);
        assert_eq!(collector.get_metrics()[0].value, 42.0);
    }

    #[test]
    fn test_span_lifecycle() {
        let config = TelemetryConfig {
            enabled: Some(true),
            ..Default::default()
        };
        let mut collector = TelemetryCollector::new(config);

        let span_id = collector.start_span("test_operation", None);
        collector.end_span(&span_id, SpanStatus::Ok);

        let spans = collector.get_spans();
        assert_eq!(spans.len(), 1);
        assert!(spans[0].duration_ms.is_some());
        assert_eq!(spans[0].status, SpanStatus::Ok);
    }

    #[test]
    fn test_build_tracker() {
        let config = BuildTrackingConfig {
            enabled: Some(true),
            ..Default::default()
        };
        let mut tracker = BuildTracker::new(config);

        let result = tracker.track("test_op", || {
            std::thread::sleep(std::time::Duration::from_millis(10));
            42
        });

        assert_eq!(result, 42);
        assert_eq!(tracker.get_timings().len(), 1);
        assert!(tracker.get_timings()[0].duration_ms >= 10);
    }
}
