use rstest::rstest;
use tempfile::TempDir;
use vx_metrics::MetricsConfig;

#[test]
fn test_metrics_config_default() {
    let config = MetricsConfig::default();
    assert!(!config.debug);
    assert!(!config.verbose);
    assert!(config.command.is_empty());
    assert!(config.metrics_dir.is_none());
}

#[test]
fn test_metrics_guard_exit_code() {
    let temp = TempDir::new().unwrap();
    let guard = vx_metrics::init(MetricsConfig {
        command: "test".to_string(),
        metrics_dir: Some(temp.path().to_path_buf()),
        ..Default::default()
    });

    // Default exit code is 0
    let handle = guard.exit_code_handle();
    assert_eq!(handle.load(std::sync::atomic::Ordering::Relaxed), 0);

    // Set exit code
    guard.set_exit_code(42);
    assert_eq!(handle.load(std::sync::atomic::Ordering::Relaxed), 42);
}

#[test]
fn test_metrics_guard_exit_code_handle_shared() {
    let temp = TempDir::new().unwrap();
    let guard = vx_metrics::init(MetricsConfig {
        command: "test".to_string(),
        metrics_dir: Some(temp.path().to_path_buf()),
        ..Default::default()
    });

    let handle1 = guard.exit_code_handle();
    let handle2 = guard.exit_code_handle();

    // Both handles should point to the same atomic
    guard.set_exit_code(7);
    assert_eq!(handle1.load(std::sync::atomic::Ordering::Relaxed), 7);
    assert_eq!(handle2.load(std::sync::atomic::Ordering::Relaxed), 7);
}

// ============================================================================
// Per-layer filter directive tests
// ============================================================================

#[test]
fn test_fmt_filter_normal_mode() {
    // Ensure RUST_LOG is not set for this test
    std::env::remove_var("RUST_LOG");

    let config = MetricsConfig::default();
    let filter = vx_metrics::fmt_filter_directive(&config);

    // Normal mode: only warn and error should be shown on stderr
    assert_eq!(filter, "warn,error");
    // Must NOT contain vx=trace or vx=debug
    assert!(!filter.contains("trace"));
    assert!(!filter.contains("debug"));
}

#[test]
fn test_fmt_filter_verbose_mode() {
    std::env::remove_var("RUST_LOG");

    let config = MetricsConfig {
        verbose: true,
        ..Default::default()
    };
    let filter = vx_metrics::fmt_filter_directive(&config);

    assert_eq!(filter, "vx=debug,info");
}

#[test]
fn test_fmt_filter_debug_mode() {
    std::env::remove_var("RUST_LOG");

    let config = MetricsConfig {
        debug: true,
        ..Default::default()
    };
    let filter = vx_metrics::fmt_filter_directive(&config);

    assert_eq!(filter, "debug");
}

#[test]
fn test_fmt_filter_debug_takes_precedence_over_verbose() {
    std::env::remove_var("RUST_LOG");

    let config = MetricsConfig {
        debug: true,
        verbose: true,
        ..Default::default()
    };
    let filter = vx_metrics::fmt_filter_directive(&config);

    // debug mode should take precedence
    assert_eq!(filter, "debug");
}

#[test]
fn test_otel_filter_always_captures_vx_trace() {
    std::env::remove_var("RUST_LOG");

    let filter = vx_metrics::otel_filter_directive();

    // OTel filter must always include vx=trace for metrics collection
    assert!(filter.contains("vx=trace"));
    assert!(filter.contains("warn"));
    assert!(filter.contains("error"));
}

#[rstest]
#[case(false, false, "warn,error")]
#[case(true, false, "vx=debug,info")]
#[case(false, true, "debug")]
#[case(true, true, "debug")]
fn test_fmt_filter_matrix(#[case] verbose: bool, #[case] debug: bool, #[case] expected: &str) {
    std::env::remove_var("RUST_LOG");

    let config = MetricsConfig {
        verbose,
        debug,
        ..Default::default()
    };
    let filter = vx_metrics::fmt_filter_directive(&config);
    assert_eq!(filter, expected);
}

#[test]
fn test_otel_filter_independent_of_config() {
    std::env::remove_var("RUST_LOG");

    // OTel filter should be the same regardless of verbose/debug settings
    let filter = vx_metrics::otel_filter_directive();
    assert_eq!(filter, "vx=trace,warn,error");
}
