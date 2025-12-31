//! Test support tests.

use rstest::rstest;
use vx_console::TestOutput;

#[rstest]
fn test_output_new() {
    let output = TestOutput::new();
    assert!(output.is_empty());
    assert_eq!(output.len(), 0);
}

#[rstest]
fn test_output_write() {
    let output = TestOutput::new();
    output.write("line 1");
    output.write("line 2");
    assert_eq!(output.len(), 2);
    assert!(!output.is_empty());
}

#[rstest]
fn test_output_lines() {
    let output = TestOutput::new();
    output.write("first");
    output.write("second");
    let lines = output.lines();
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0], "first");
    assert_eq!(lines[1], "second");
}

#[rstest]
fn test_output_output() {
    let output = TestOutput::new();
    output.write("hello");
    output.write("world");
    let full = output.output();
    assert!(full.contains("hello"));
    assert!(full.contains("world"));
}

#[rstest]
fn test_output_contains() {
    let output = TestOutput::new();
    output.write("hello world");
    assert!(output.contains("hello"));
    assert!(output.contains("world"));
    assert!(!output.contains("foo"));
}

#[rstest]
fn test_output_has_success() {
    let output = TestOutput::new();
    output.write("✓ Operation completed");
    assert!(output.has_success("completed"));
    assert!(!output.has_success("failed"));
}

#[rstest]
fn test_output_has_success_minimal() {
    let output = TestOutput::new();
    output.write("[OK] Done");
    assert!(output.has_success("Done"));
}

#[rstest]
fn test_output_has_error() {
    let output = TestOutput::new();
    output.write("✗ Operation failed");
    assert!(output.has_error("failed"));
    assert!(!output.has_error("success"));
}

#[rstest]
fn test_output_has_error_minimal() {
    let output = TestOutput::new();
    output.write("[ERROR] Something went wrong");
    assert!(output.has_error("wrong"));
}

#[rstest]
fn test_output_clear() {
    let output = TestOutput::new();
    output.write("test");
    assert!(!output.is_empty());
    output.clear();
    assert!(output.is_empty());
}

#[rstest]
fn test_output_clone() {
    let output = TestOutput::new();
    output.write("test");
    let cloned = output.clone();
    assert_eq!(cloned.len(), 1);
    assert!(cloned.contains("test"));
}

#[rstest]
fn test_output_default() {
    let output = TestOutput::default();
    assert!(output.is_empty());
}
