//! Testing utilities for vx-console.

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
}
