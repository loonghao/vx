//! Testing utilities for vx-console.

use std::io::Write;
use std::sync::{Arc, Mutex};

/// A test output buffer that captures console output.
#[derive(Debug, Clone, Default)]
pub struct TestOutput {
    buffer: Arc<Mutex<Vec<String>>>,
}

impl TestOutput {
    /// Create a new test output buffer.
    pub fn new() -> Self {
        Self {
            buffer: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Write a line to the buffer.
    pub fn write(&self, line: &str) {
        if let Ok(mut buffer) = self.buffer.lock() {
            buffer.push(line.to_string());
        }
    }

    /// Get all lines in the buffer.
    pub fn lines(&self) -> Vec<String> {
        self.buffer.lock().map(|b| b.clone()).unwrap_or_default()
    }

    /// Get the full output as a single string.
    pub fn output(&self) -> String {
        self.lines().join("\n")
    }

    /// Check if the output contains a string.
    pub fn contains(&self, needle: &str) -> bool {
        self.output().contains(needle)
    }

    /// Check if there's a success message.
    pub fn has_success(&self, message: &str) -> bool {
        self.lines()
            .iter()
            .any(|line| (line.contains("✓") || line.contains("[OK]")) && line.contains(message))
    }

    /// Check if there's an error message.
    pub fn has_error(&self, message: &str) -> bool {
        self.lines()
            .iter()
            .any(|line| (line.contains("✗") || line.contains("[ERROR]")) && line.contains(message))
    }

    /// Check if there's a warning message.
    pub fn has_warning(&self, message: &str) -> bool {
        self.lines()
            .iter()
            .any(|line| (line.contains("⚠") || line.contains("[WARN]")) && line.contains(message))
    }

    /// Check if there's an info message.
    pub fn has_info(&self, message: &str) -> bool {
        self.lines()
            .iter()
            .any(|line| (line.contains("ℹ") || line.contains("[INFO]")) && line.contains(message))
    }

    /// Check if there's a hint message.
    pub fn has_hint(&self, message: &str) -> bool {
        self.lines()
            .iter()
            .any(|line| (line.contains("💡") || line.contains("[HINT]")) && line.contains(message))
    }

    /// Check if there's a debug message.
    pub fn has_debug(&self, message: &str) -> bool {
        self.lines()
            .iter()
            .any(|line| (line.contains("→") || line.contains("[DEBUG]")) && line.contains(message))
    }

    /// Get lines matching a pattern.
    pub fn lines_matching(&self, pattern: &str) -> Vec<String> {
        self.lines()
            .into_iter()
            .filter(|line| line.contains(pattern))
            .collect()
    }

    /// Get the first line containing a pattern.
    pub fn first_line_containing(&self, pattern: &str) -> Option<String> {
        self.lines().into_iter().find(|line| line.contains(pattern))
    }

    /// Get the last line containing a pattern.
    pub fn last_line_containing(&self, pattern: &str) -> Option<String> {
        self.lines()
            .into_iter()
            .rev()
            .find(|line| line.contains(pattern))
    }

    /// Clear the buffer.
    pub fn clear(&self) {
        if let Ok(mut buffer) = self.buffer.lock() {
            buffer.clear();
        }
    }

    /// Get the number of lines.
    pub fn len(&self) -> usize {
        self.buffer.lock().map(|b| b.len()).unwrap_or(0)
    }

    /// Check if the buffer is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Assert that the output contains a string.
    ///
    /// Panics with a helpful message if the assertion fails.
    pub fn assert_contains(&self, needle: &str) {
        assert!(
            self.contains(needle),
            "Expected output to contain '{}', but got:\n{}",
            needle,
            self.output()
        );
    }

    /// Assert that the output does not contain a string.
    ///
    /// Panics with a helpful message if the assertion fails.
    pub fn assert_not_contains(&self, needle: &str) {
        assert!(
            !self.contains(needle),
            "Expected output to NOT contain '{}', but got:\n{}",
            needle,
            self.output()
        );
    }

    /// Assert that there's a success message.
    pub fn assert_success(&self, message: &str) {
        assert!(
            self.has_success(message),
            "Expected success message '{}', but got:\n{}",
            message,
            self.output()
        );
    }

    /// Assert that there's an error message.
    pub fn assert_error(&self, message: &str) {
        assert!(
            self.has_error(message),
            "Expected error message '{}', but got:\n{}",
            message,
            self.output()
        );
    }

    /// Assert that there's a warning message.
    pub fn assert_warning(&self, message: &str) {
        assert!(
            self.has_warning(message),
            "Expected warning message '{}', but got:\n{}",
            message,
            self.output()
        );
    }

    /// Assert that there's an info message.
    pub fn assert_info(&self, message: &str) {
        assert!(
            self.has_info(message),
            "Expected info message '{}', but got:\n{}",
            message,
            self.output()
        );
    }

    /// Assert that the output has exactly N lines.
    pub fn assert_line_count(&self, expected: usize) {
        let actual = self.len();
        assert!(
            actual == expected,
            "Expected {} lines, but got {}:\n{}",
            expected,
            actual,
            self.output()
        );
    }

    /// Assert that the output is empty.
    pub fn assert_empty(&self) {
        assert!(
            self.is_empty(),
            "Expected empty output, but got:\n{}",
            self.output()
        );
    }

    /// Assert that the output is not empty.
    pub fn assert_not_empty(&self) {
        assert!(
            !self.is_empty(),
            "Expected non-empty output, but got nothing"
        );
    }
}

/// A writer that captures output to a TestOutput.
pub struct TestWriter {
    output: TestOutput,
}

impl TestWriter {
    /// Create a new test writer.
    pub fn new(output: TestOutput) -> Self {
        Self { output }
    }

    /// Get the underlying TestOutput.
    pub fn output(&self) -> &TestOutput {
        &self.output
    }
}

impl Write for TestWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        if let Ok(s) = std::str::from_utf8(buf) {
            for line in s.lines() {
                self.output.write(line);
            }
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_output_write() {
        let output = TestOutput::new();
        output.write("line 1");
        output.write("line 2");
        assert_eq!(output.len(), 2);
        assert!(output.contains("line 1"));
    }

    #[test]
    fn test_output_clear() {
        let output = TestOutput::new();
        output.write("test");
        output.clear();
        assert!(output.is_empty());
    }

    #[test]
    fn test_output_has_success() {
        let output = TestOutput::new();
        output.write("✓ Task completed");
        assert!(output.has_success("Task completed"));
    }

    #[test]
    fn test_output_has_error() {
        let output = TestOutput::new();
        output.write("✗ Task failed");
        assert!(output.has_error("Task failed"));
    }

    #[test]
    fn test_output_lines_matching() {
        let output = TestOutput::new();
        output.write("foo bar");
        output.write("baz qux");
        output.write("foo baz");
        let matches = output.lines_matching("foo");
        assert_eq!(matches.len(), 2);
    }

    #[test]
    fn test_output_assert_contains() {
        let output = TestOutput::new();
        output.write("hello world");
        output.assert_contains("hello");
    }

    #[test]
    fn test_test_writer() {
        let output = TestOutput::new();
        let mut writer = TestWriter::new(output.clone());
        write!(writer, "test line").unwrap();
        assert!(output.contains("test line"));
    }
}
