use std::collections::HashMap;
use vx_config::{
    BuildTracker, BuildTrackingConfig, SpanStatus, TelemetryCollector, TelemetryConfig,
};

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
