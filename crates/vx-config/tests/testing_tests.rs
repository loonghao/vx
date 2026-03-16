use tempfile::tempdir;
use vx_config::{CoverageConfig, CoverageReporter, TestFramework, TestResult};

#[test]
fn test_framework_detection() {
    let dir = tempdir().unwrap();

    // Create Cargo.toml
    std::fs::write(dir.path().join("Cargo.toml"), "[package]").unwrap();
    assert_eq!(TestFramework::detect(dir.path()), TestFramework::CargoTest);
}

#[test]
fn test_result_pass_rate() {
    let result = TestResult {
        total: 100,
        passed: 95,
        failed: 5,
        skipped: 0,
        duration_ms: 1000,
        coverage: Some(80.0),
        output: String::new(),
    };

    assert_eq!(result.pass_rate(), 95.0);
    assert!(!result.is_success());
}

#[test]
fn test_coverage_threshold() {
    let config = CoverageConfig {
        enabled: Some(true),
        threshold: Some(80),
        ..Default::default()
    };

    let reporter = CoverageReporter::new(config);
    assert!(reporter.check_threshold(85.0).is_ok());
    assert!(reporter.check_threshold(75.0).is_err());
}
