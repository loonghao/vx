//! Tests for the visualization module.

use std::collections::HashMap;
use vx_metrics::report::{CommandMetrics, StageMetrics};
use vx_metrics::visualize::{
    generate_ai_summary, generate_html_report, load_metrics, render_comparison, render_insights,
    render_summary,
};

fn sample_metrics(
    total: f64,
    resolve: f64,
    ensure: f64,
    prepare: f64,
    execute: f64,
) -> CommandMetrics {
    let mut stages = HashMap::new();
    stages.insert(
        "resolve".to_string(),
        StageMetrics {
            duration_ms: resolve,
            success: true,
            error: None,
        },
    );
    stages.insert(
        "ensure".to_string(),
        StageMetrics {
            duration_ms: ensure,
            success: true,
            error: None,
        },
    );
    stages.insert(
        "prepare".to_string(),
        StageMetrics {
            duration_ms: prepare,
            success: true,
            error: None,
        },
    );
    stages.insert(
        "execute".to_string(),
        StageMetrics {
            duration_ms: execute,
            success: true,
            error: None,
        },
    );

    CommandMetrics {
        version: "1".to_string(),
        timestamp: "2026-02-07T16:00:00+00:00".to_string(),
        command: "vx node --version".to_string(),
        exit_code: Some(0),
        total_duration_ms: total,
        stages,
        spans: Vec::new(),
    }
}

#[test]
fn test_render_summary_contains_stages() {
    let m = sample_metrics(1000.0, 100.0, 5.0, 200.0, 600.0);
    let output = render_summary(&m);

    assert!(output.contains("vx node --version"));
    assert!(output.contains("1000.00ms"));
    assert!(output.contains("resolve"));
    assert!(output.contains("prepare"));
    assert!(output.contains("execute"));
    assert!(output.contains("ensure"));
}

#[test]
fn test_render_summary_shows_overhead() {
    // total=1000, stages sum=905, overhead=95
    let m = sample_metrics(1000.0, 100.0, 5.0, 200.0, 600.0);
    let output = render_summary(&m);
    assert!(output.contains("overhead"));
}

#[test]
fn test_render_comparison_empty() {
    let output = render_comparison(&[]);
    assert!(output.contains("No metrics data"));
}

#[test]
fn test_render_comparison_single() {
    let m = sample_metrics(500.0, 50.0, 1.0, 100.0, 300.0);
    let output = render_comparison(&[m]);
    assert!(output.contains("Performance History"));
    assert!(output.contains("vx node --version"));
}

#[test]
fn test_render_comparison_multiple_with_stats() {
    let runs = vec![
        sample_metrics(500.0, 50.0, 1.0, 100.0, 300.0),
        sample_metrics(600.0, 60.0, 2.0, 120.0, 350.0),
        sample_metrics(450.0, 40.0, 1.0, 90.0, 280.0),
    ];
    let output = render_comparison(&runs);
    assert!(output.contains("Stats (3 runs)"));
    assert!(output.contains("avg="));
    assert!(output.contains("Stage Averages"));
}

#[test]
fn test_render_insights_empty() {
    let output = render_insights(&[]);
    assert!(output.is_empty());
}

#[test]
fn test_render_insights_slow_prepare() {
    let m = sample_metrics(500.0, 50.0, 1.0, 200.0, 200.0);
    let output = render_insights(&[m]);
    assert!(output.contains("prepare"));
    assert!(output.contains("slow"));
}

#[test]
fn test_render_insights_slow_resolve() {
    let m = sample_metrics(500.0, 200.0, 1.0, 50.0, 200.0);
    let output = render_insights(&[m]);
    assert!(output.contains("resolve"));
}

#[test]
fn test_render_insights_bottleneck() {
    // execute takes >50% of total
    let m = sample_metrics(1000.0, 50.0, 1.0, 50.0, 850.0);
    let output = render_insights(&[m]);
    assert!(output.contains("Bottleneck"));
    assert!(output.contains("execute"));
}

#[test]
fn test_generate_html_report_not_empty() {
    let runs = vec![sample_metrics(500.0, 50.0, 1.0, 100.0, 300.0)];
    let html = generate_html_report(&runs);
    assert!(html.contains("<!DOCTYPE html>"));
    assert!(html.contains("VX Performance Report"));
    assert!(html.contains("chart.js"));
    assert!(html.contains("pieChart"));
    assert!(html.contains("lineChart"));
}

#[test]
fn test_generate_ai_summary_structure() {
    let runs = vec![
        sample_metrics(500.0, 50.0, 1.0, 100.0, 300.0),
        sample_metrics(600.0, 60.0, 2.0, 120.0, 350.0),
    ];
    let summary = generate_ai_summary(&runs);

    assert_eq!(summary["runs_analyzed"], 2);
    assert!(summary["total_ms"]["avg"].as_f64().unwrap() > 0.0);
    assert!(summary["stages"]["resolve"]["avg_ms"].as_f64().unwrap() > 0.0);
    assert!(summary["latest_run"]["command"].as_str().unwrap() == "vx node --version");
}

#[test]
fn test_generate_ai_summary_detects_bottlenecks() {
    // prepare > 100ms should trigger bottleneck
    let runs = vec![sample_metrics(500.0, 50.0, 1.0, 200.0, 200.0)];
    let summary = generate_ai_summary(&runs);
    let bottlenecks = summary["bottlenecks"].as_array().unwrap();
    assert!(!bottlenecks.is_empty());
    assert!(bottlenecks[0]["stage"].as_str().unwrap() == "prepare");
}

#[test]
fn test_load_metrics_nonexistent_dir() {
    let result = load_metrics(std::path::Path::new("/nonexistent/dir"), 10);
    assert!(result.is_err());
}

#[test]
fn test_load_metrics_empty_dir() {
    let tmp = tempfile::TempDir::new().unwrap();
    let result = load_metrics(tmp.path(), 10).unwrap();
    assert!(result.is_empty());
}

#[test]
fn test_load_metrics_with_files() {
    let tmp = tempfile::TempDir::new().unwrap();

    // Write two sample files
    let m1 = sample_metrics(500.0, 50.0, 1.0, 100.0, 300.0);
    let m2 = sample_metrics(600.0, 60.0, 2.0, 120.0, 350.0);

    std::fs::write(
        tmp.path().join("20260207_160000_000.json"),
        serde_json::to_string(&m1).unwrap(),
    )
    .unwrap();
    std::fs::write(
        tmp.path().join("20260207_160100_000.json"),
        serde_json::to_string(&m2).unwrap(),
    )
    .unwrap();

    let result = load_metrics(tmp.path(), 10).unwrap();
    assert_eq!(result.len(), 2);
    // Newest first
    assert_eq!(result[0].total_duration_ms, 600.0);
    assert_eq!(result[1].total_duration_ms, 500.0);
}

#[test]
fn test_load_metrics_respects_limit() {
    let tmp = tempfile::TempDir::new().unwrap();

    for i in 0..5 {
        let m = sample_metrics(100.0 * (i + 1) as f64, 10.0, 1.0, 20.0, 50.0);
        std::fs::write(
            tmp.path().join(format!("2026020{}_160000_000.json", i)),
            serde_json::to_string(&m).unwrap(),
        )
        .unwrap();
    }

    let result = load_metrics(tmp.path(), 3).unwrap();
    assert_eq!(result.len(), 3);
}
