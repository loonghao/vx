//! Format tests.

use rstest::rstest;
use vx_console::{JsonOutput, OutputMode};

#[rstest]
fn test_output_mode_show_progress() {
    assert!(OutputMode::Standard.show_progress());
    assert!(OutputMode::Verbose.show_progress());
    assert!(!OutputMode::Quiet.show_progress());
    assert!(!OutputMode::Json.show_progress());
    assert!(!OutputMode::Ci.show_progress());
}

#[rstest]
fn test_output_mode_show_colors() {
    assert!(OutputMode::Standard.show_colors());
    assert!(OutputMode::Verbose.show_colors());
    assert!(OutputMode::Ci.show_colors());
    assert!(!OutputMode::Json.show_colors());
    assert!(!OutputMode::Quiet.show_colors());
}

#[rstest]
fn test_output_mode_show_debug() {
    assert!(OutputMode::Verbose.show_debug());
    assert!(!OutputMode::Standard.show_debug());
    assert!(!OutputMode::Quiet.show_debug());
    assert!(!OutputMode::Json.show_debug());
    assert!(!OutputMode::Ci.show_debug());
}

#[rstest]
fn test_output_mode_default() {
    assert_eq!(OutputMode::default(), OutputMode::Standard);
}

#[rstest]
fn test_json_output_info() {
    let output = JsonOutput::info("test message");
    assert_eq!(output.level, "info");
    assert_eq!(output.message, "test message");
    assert!(output.timestamp.is_some());
}

#[rstest]
fn test_json_output_success() {
    let output = JsonOutput::success("done");
    assert_eq!(output.level, "success");
    assert_eq!(output.message, "done");
}

#[rstest]
fn test_json_output_warn() {
    let output = JsonOutput::warn("warning");
    assert_eq!(output.level, "warn");
}

#[rstest]
fn test_json_output_error() {
    let output = JsonOutput::error("error");
    assert_eq!(output.level, "error");
}

#[rstest]
fn test_json_output_debug() {
    let output = JsonOutput::debug("debug info");
    assert_eq!(output.level, "debug");
}

#[rstest]
fn test_json_output_to_json() {
    let output = JsonOutput::info("test");
    let json = output.to_json();
    assert!(json.contains("info"));
    assert!(json.contains("test"));
}

#[rstest]
fn test_json_output_with_context() {
    let output = JsonOutput::info("test").with_context(serde_json::json!({"key": "value"}));
    let json = output.to_json();
    assert!(json.contains("context"));
    assert!(json.contains("key"));
    assert!(json.contains("value"));
}
