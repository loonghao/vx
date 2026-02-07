use rstest::rstest;
use vx_metrics::report::{CommandMetrics, StageMetrics};
use vx_metrics::exporter::SpanRecord;

#[test]
fn test_command_metrics_new() {
    let metrics = CommandMetrics::new("vx node --version".to_string());
    assert_eq!(metrics.version, "1");
    assert_eq!(metrics.command, "vx node --version");
    assert!(metrics.exit_code.is_none());
    assert_eq!(metrics.total_duration_ms, 0.0);
    assert!(metrics.stages.is_empty());
    assert!(metrics.spans.is_empty());
}

#[test]
fn test_command_metrics_serialization() {
    let metrics = CommandMetrics::new("vx go build".to_string());
    let json = serde_json::to_string_pretty(&metrics).unwrap();
    assert!(json.contains("vx go build"));
    assert!(json.contains("\"version\": \"1\""));
    // exit_code is None, should be skipped
    assert!(!json.contains("exit_code"));
    // stages is empty, should be skipped
    assert!(!json.contains("stages"));
}

#[test]
fn test_command_metrics_with_exit_code() {
    let mut metrics = CommandMetrics::new("vx node".to_string());
    metrics.exit_code = Some(0);
    let json = serde_json::to_string(&metrics).unwrap();
    assert!(json.contains("\"exit_code\":0"));
}

#[test]
fn test_extract_stages_from_spans() {
    let mut metrics = CommandMetrics::new("vx node --version".to_string());
    metrics.spans = vec![
        make_span("resolve", 50.0, "ok"),
        make_span("ensure", 800.0, "ok"),
        make_span("prepare", 10.0, "ok"),
        make_span("execute_process", 374.0, "ok"),
    ];

    metrics.extract_stages_from_spans();

    assert_eq!(metrics.stages.len(), 4);
    assert_eq!(metrics.stages["resolve"].duration_ms, 50.0);
    assert!(metrics.stages["resolve"].success);
    assert_eq!(metrics.stages["ensure"].duration_ms, 800.0);
    assert_eq!(metrics.stages["prepare"].duration_ms, 10.0);
    assert_eq!(metrics.stages["execute"].duration_ms, 374.0);
}

#[test]
fn test_extract_stages_with_error() {
    let mut metrics = CommandMetrics::new("vx unknown".to_string());
    metrics.spans = vec![make_span("resolve", 5.0, "error: not found")];

    metrics.extract_stages_from_spans();

    assert_eq!(metrics.stages.len(), 1);
    assert!(!metrics.stages["resolve"].success);
    assert_eq!(
        metrics.stages["resolve"].error.as_deref(),
        Some("error: not found")
    );
}

#[test]
fn test_extract_stages_deduplicates() {
    let mut metrics = CommandMetrics::new("test".to_string());
    // Two spans named "resolve" - only the first should be kept
    metrics.spans = vec![
        make_span("resolve", 10.0, "ok"),
        make_span("resolve", 20.0, "ok"),
    ];

    metrics.extract_stages_from_spans();

    assert_eq!(metrics.stages.len(), 1);
    assert_eq!(metrics.stages["resolve"].duration_ms, 10.0);
}

#[test]
fn test_compute_total_duration_from_root_span() {
    let mut metrics = CommandMetrics::new("test".to_string());
    metrics.spans = vec![
        SpanRecord {
            name: "root".to_string(),
            parent_span_id: "0000000000000000".to_string(),
            duration_ms: 1234.0,
            ..make_span("root", 1234.0, "ok")
        },
        make_span("resolve", 50.0, "ok"),
    ];
    metrics.stages.insert(
        "resolve".to_string(),
        StageMetrics {
            duration_ms: 50.0,
            success: true,
            error: None,
        },
    );

    metrics.compute_total_duration();

    assert_eq!(metrics.total_duration_ms, 1234.0);
}

#[test]
fn test_compute_total_duration_from_stages_sum() {
    let mut metrics = CommandMetrics::new("test".to_string());
    // No root span - should sum stages
    metrics.stages.insert(
        "resolve".to_string(),
        StageMetrics {
            duration_ms: 50.0,
            success: true,
            error: None,
        },
    );
    metrics.stages.insert(
        "ensure".to_string(),
        StageMetrics {
            duration_ms: 100.0,
            success: true,
            error: None,
        },
    );

    metrics.compute_total_duration();

    assert_eq!(metrics.total_duration_ms, 150.0);
}

#[test]
fn test_stage_metrics_serialization() {
    let stage = StageMetrics {
        duration_ms: 42.5,
        success: true,
        error: None,
    };
    let json = serde_json::to_string(&stage).unwrap();
    assert!(json.contains("42.5"));
    assert!(json.contains("true"));
    // error is None, should be skipped
    assert!(!json.contains("error"));
}

#[rstest]
#[case("resolve", true)]
#[case("RESOLVE", true)]
#[case("ensure", true)]
#[case("prepare", true)]
#[case("execute", true)]
#[case("execute_process", true)]
#[case("random_span", false)]
fn test_stage_name_matching(#[case] span_name: &str, #[case] should_match: bool) {
    let mut metrics = CommandMetrics::new("test".to_string());
    metrics.spans = vec![make_span(span_name, 10.0, "ok")];

    metrics.extract_stages_from_spans();

    assert_eq!(!metrics.stages.is_empty(), should_match);
}

#[test]
fn test_command_metrics_roundtrip() {
    let mut metrics = CommandMetrics::new("vx node --version".to_string());
    metrics.exit_code = Some(0);
    metrics.total_duration_ms = 1234.5;
    metrics.stages.insert(
        "resolve".to_string(),
        StageMetrics {
            duration_ms: 50.0,
            success: true,
            error: None,
        },
    );
    metrics.spans = vec![make_span("resolve", 50.0, "ok")];

    let json = serde_json::to_string_pretty(&metrics).unwrap();
    let deserialized: CommandMetrics = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.command, "vx node --version");
    assert_eq!(deserialized.exit_code, Some(0));
    assert_eq!(deserialized.total_duration_ms, 1234.5);
    assert_eq!(deserialized.stages.len(), 1);
    assert_eq!(deserialized.spans.len(), 1);
}

// Helper to create a test SpanRecord
fn make_span(name: &str, duration_ms: f64, status: &str) -> SpanRecord {
    SpanRecord {
        name: name.to_string(),
        trace_id: "trace123".to_string(),
        span_id: "span456".to_string(),
        parent_span_id: "parent789".to_string(),
        start_time_unix_ns: 0,
        end_time_unix_ns: (duration_ms * 1_000_000.0) as u64,
        duration_ms,
        status: status.to_string(),
        attributes: std::collections::HashMap::new(),
        events: vec![],
    }
}
