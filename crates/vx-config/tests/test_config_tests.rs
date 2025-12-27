//! Test configuration tests
//!
//! Tests for test-related configuration parsing.

use rstest::rstest;
use vx_config::parse_config_str;

// ============================================
// Test Config Parsing Tests
// ============================================

#[test]
fn test_parse_test_config_basic() {
    let content = r#"
[test]
framework = "jest"
parallel = true
workers = 4
"#;
    let config = parse_config_str(content).unwrap();
    let test = config.test.unwrap();
    assert_eq!(test.framework, Some("jest".to_string()));
    assert_eq!(test.parallel, Some(true));
    assert_eq!(test.workers, Some(4));
}

#[rstest]
#[case("jest")]
#[case("pytest")]
#[case("cargo-test")]
#[case("go-test")]
#[case("vitest")]
#[case("mocha")]
fn test_parse_test_frameworks(#[case] framework: &str) {
    let content = format!(
        r#"
[test]
framework = "{}"
"#,
        framework
    );
    let config = parse_config_str(&content).unwrap();
    let test = config.test.unwrap();
    assert_eq!(test.framework, Some(framework.to_string()));
}

#[test]
fn test_parse_test_parallel_disabled() {
    let content = r#"
[test]
parallel = false
"#;
    let config = parse_config_str(content).unwrap();
    let test = config.test.unwrap();
    assert_eq!(test.parallel, Some(false));
}

#[rstest]
#[case(1)]
#[case(2)]
#[case(4)]
#[case(8)]
#[case(16)]
fn test_parse_test_workers(#[case] workers: u32) {
    let content = format!(
        r#"
[test]
workers = {}
"#,
        workers
    );
    let config = parse_config_str(&content).unwrap();
    let test = config.test.unwrap();
    assert_eq!(test.workers, Some(workers));
}

#[test]
fn test_parse_test_coverage_config() {
    let content = r#"
[test.coverage]
enabled = true
threshold = 80
exclude = ["tests/", "fixtures/", "*.test.ts"]
"#;
    let config = parse_config_str(content).unwrap();
    let test = config.test.unwrap();
    let coverage = test.coverage.unwrap();

    assert_eq!(coverage.enabled, Some(true));
    assert_eq!(coverage.threshold, Some(80));
    assert_eq!(
        coverage.exclude,
        vec![
            "tests/".to_string(),
            "fixtures/".to_string(),
            "*.test.ts".to_string()
        ]
    );
}

#[test]
fn test_parse_test_coverage_formats() {
    let content = r#"
[test.coverage]
enabled = true
formats = ["html", "lcov", "json"]
output = "coverage/"
"#;
    let config = parse_config_str(content).unwrap();
    let test = config.test.unwrap();
    let coverage = test.coverage.unwrap();

    assert_eq!(
        coverage.formats,
        vec!["html".to_string(), "lcov".to_string(), "json".to_string()]
    );
    assert_eq!(coverage.output, Some("coverage/".to_string()));
}

#[test]
fn test_parse_test_hooks() {
    let content = r#"
[test.hooks]
before_all = "npm run lint"
after_all = "npm run report"
"#;
    let config = parse_config_str(content).unwrap();
    let test = config.test.unwrap();
    let hooks = test.hooks.unwrap();

    assert_eq!(hooks.before_all, Some("npm run lint".to_string()));
    assert_eq!(hooks.after_all, Some("npm run report".to_string()));
}

#[test]
fn test_parse_test_timeout() {
    let content = r#"
[test]
timeout = 30
"#;
    let config = parse_config_str(content).unwrap();
    let test = config.test.unwrap();
    assert_eq!(test.timeout, Some(30));
}

#[test]
fn test_parse_full_test_config() {
    let content = r#"
[test]
framework = "pytest"
parallel = true
workers = 4
timeout = 60

[test.coverage]
enabled = true
threshold = 85
formats = ["html", "xml"]
output = "htmlcov/"

[test.hooks]
before_all = "python -m black --check ."
after_all = "python -m coverage report"
"#;
    let config = parse_config_str(content).unwrap();
    let test = config.test.unwrap();

    assert_eq!(test.framework, Some("pytest".to_string()));
    assert_eq!(test.parallel, Some(true));
    assert_eq!(test.workers, Some(4));
    assert_eq!(test.timeout, Some(60));
    assert!(test.coverage.is_some());
    assert!(test.hooks.is_some());
}

#[test]
fn test_test_config_empty() {
    let content = r#"
[test]
"#;
    let config = parse_config_str(content).unwrap();
    let test = config.test.unwrap();

    assert!(test.framework.is_none());
    assert!(test.parallel.is_none());
    assert!(test.workers.is_none());
    assert!(test.coverage.is_none());
}

#[rstest]
#[case(50)]
#[case(75)]
#[case(80)]
#[case(90)]
#[case(100)]
fn test_parse_coverage_thresholds(#[case] threshold: u32) {
    let content = format!(
        r#"
[test.coverage]
threshold = {}
"#,
        threshold
    );
    let config = parse_config_str(&content).unwrap();
    let test = config.test.unwrap();
    let coverage = test.coverage.unwrap();

    assert_eq!(coverage.threshold, Some(threshold));
}
