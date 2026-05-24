use vx_metrics::report::{CommandMetrics, TokenSavingsRecord};
use vx_metrics::{
    build_token_savings_record, estimate_tokens, render_token_savings, summarize_token_savings,
};

#[test]
fn test_estimate_tokens_uses_stable_char_heuristic() {
    assert_eq!(estimate_tokens(""), 0);
    assert_eq!(estimate_tokens("abcd"), 1);
    assert_eq!(estimate_tokens("abcde"), 2);
    assert_eq!(estimate_tokens("你好"), 2);
}

#[test]
fn test_build_token_savings_record() {
    let record = build_token_savings_record(
        "ListOutput",
        "toon",
        "json",
        r#"{"runtimes":[{"name":"node"}]}"#,
        "runtimes[1]{name}:\n  node",
    );

    assert_eq!(record.output_type, "ListOutput");
    assert_eq!(record.output_format, "toon");
    assert_eq!(record.baseline_format, "json");
    assert!(record.baseline_tokens >= record.actual_tokens);
    assert!(record.token_delta >= 0);
}

#[test]
fn test_build_token_savings_record_preserves_negative_delta() {
    let record = build_token_savings_record("ListOutput", "toon", "json", "abcd", "abcdabcd");

    assert_eq!(record.token_delta, -1);
    assert!(record.savings_ratio < 0.0);
}

#[test]
fn test_summarize_token_savings_by_command() {
    let mut first = CommandMetrics::new("vx --output-format toon list".to_string());
    first.token_savings = vec![TokenSavingsRecord {
        output_type: "ListOutput".to_string(),
        output_format: "toon".to_string(),
        baseline_format: "json".to_string(),
        baseline_bytes: 400,
        actual_bytes: 200,
        baseline_tokens: 100,
        actual_tokens: 50,
        token_delta: 50,
        savings_ratio: 0.5,
    }];

    let mut second = CommandMetrics::new("vx --compact list node".to_string());
    second.token_savings = vec![TokenSavingsRecord {
        output_type: "VersionsOutput".to_string(),
        output_format: "compact".to_string(),
        baseline_format: "text".to_string(),
        baseline_bytes: 80,
        actual_bytes: 20,
        baseline_tokens: 20,
        actual_tokens: 5,
        token_delta: 15,
        savings_ratio: 0.75,
    }];

    let summary = summarize_token_savings(&[first, second]);

    assert_eq!(summary.runs, 2);
    assert_eq!(summary.records, 2);
    assert_eq!(summary.baseline_tokens, 120);
    assert_eq!(summary.actual_tokens, 55);
    assert_eq!(summary.net_saved_tokens, 65);
    assert_eq!(summary.commands.len(), 2);
    assert_eq!(summary.commands[0].command, "vx --output-format toon list");
}

#[test]
fn test_summarize_token_savings_reports_contributing_and_inspected_runs() {
    let mut first = CommandMetrics::new("vx --output-format toon list".to_string());
    first.token_savings = vec![TokenSavingsRecord {
        output_type: "ListOutput".to_string(),
        output_format: "toon".to_string(),
        baseline_format: "json".to_string(),
        baseline_bytes: 400,
        actual_bytes: 200,
        baseline_tokens: 100,
        actual_tokens: 50,
        token_delta: 50,
        savings_ratio: 0.5,
    }];

    let empty = CommandMetrics::new("vx metrics --json".to_string());

    let summary = summarize_token_savings(&[first, empty]);

    assert_eq!(summary.inspected_runs, 2);
    assert_eq!(summary.runs, 1);
    assert_eq!(summary.records, 1);
}

#[test]
fn test_render_token_savings_empty() {
    let summary = summarize_token_savings(&[]);
    let rendered = render_token_savings(&summary);
    assert!(rendered.contains("No token savings data"));
}

#[test]
fn test_command_metrics_deserializes_without_token_savings() {
    let json = r#"{
        "version": "1",
        "timestamp": "2026-02-07T10:30:00Z",
        "command": "vx list",
        "total_duration_ms": 12.0,
        "spans": []
    }"#;

    let metrics: CommandMetrics = serde_json::from_str(json).unwrap();
    assert!(metrics.stages.is_empty());
    assert!(metrics.token_savings.is_empty());
}
