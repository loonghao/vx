use rstest::rstest;
use vx_metrics::exporter::{JsonFileExporter, SpanRecord};

#[test]
fn test_new_exporter_has_no_spans() {
    let exporter = JsonFileExporter::new();
    let spans = exporter.take_spans();
    assert!(spans.is_empty());
}

#[test]
fn test_take_spans_drains_buffer() {
    let exporter = JsonFileExporter::new();

    // First take: empty
    let spans = exporter.take_spans();
    assert!(spans.is_empty());

    // Second take: still empty (no spans added externally)
    let spans = exporter.take_spans();
    assert!(spans.is_empty());
}

#[test]
fn test_exporter_debug_format() {
    let exporter = JsonFileExporter::new();
    let debug_str = format!("{:?}", exporter);
    assert!(debug_str.contains("JsonFileExporter"));
    assert!(debug_str.contains("span_count"));
    assert!(debug_str.contains("0"));
}

#[test]
fn test_exporter_clone() {
    let exporter = JsonFileExporter::new();
    let cloned = exporter.clone();

    // Both should be empty
    assert!(exporter.take_spans().is_empty());
    assert!(cloned.take_spans().is_empty());
}

#[test]
fn test_exporter_default() {
    let exporter = JsonFileExporter::default();
    assert!(exporter.take_spans().is_empty());
}

#[test]
fn test_span_record_serialization() {
    let record = SpanRecord {
        name: "test_span".to_string(),
        trace_id: "abc123".to_string(),
        span_id: "def456".to_string(),
        parent_span_id: "0000000000000000".to_string(),
        start_time_unix_ns: 1000000,
        end_time_unix_ns: 2000000,
        duration_ms: 1.0,
        status: "ok".to_string(),
        attributes: std::collections::HashMap::new(),
        events: vec![],
    };

    let json = serde_json::to_string(&record).unwrap();
    assert!(json.contains("test_span"));
    assert!(json.contains("abc123"));

    // Should not contain "events" key because it's empty and skip_serializing_if
    assert!(!json.contains("events"));
}

#[rstest]
#[case("ok", false)]
#[case("unset", false)]
#[case("error: something failed", true)]
fn test_span_record_status_variants(#[case] status: &str, #[case] is_error: bool) {
    let record = SpanRecord {
        name: "test".to_string(),
        trace_id: String::new(),
        span_id: String::new(),
        parent_span_id: String::new(),
        start_time_unix_ns: 0,
        end_time_unix_ns: 0,
        duration_ms: 0.0,
        status: status.to_string(),
        attributes: std::collections::HashMap::new(),
        events: vec![],
    };

    assert_eq!(record.status.starts_with("error"), is_error);
}

#[test]
fn test_span_record_deserialization() {
    let json = r#"{
        "name": "resolve",
        "trace_id": "abc",
        "span_id": "def",
        "parent_span_id": "000",
        "start_time_unix_ns": 100,
        "end_time_unix_ns": 200,
        "duration_ms": 0.1,
        "status": "ok",
        "attributes": {"runtime": "node"}
    }"#;

    let record: SpanRecord = serde_json::from_str(json).unwrap();
    assert_eq!(record.name, "resolve");
    assert_eq!(record.duration_ms, 0.1);
    assert_eq!(
        record.attributes.get("runtime"),
        Some(&serde_json::Value::String("node".to_string()))
    );
    assert!(record.events.is_empty());
}
