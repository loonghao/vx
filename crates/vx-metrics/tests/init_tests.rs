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
