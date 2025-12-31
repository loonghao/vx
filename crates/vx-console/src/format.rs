//! Output format definitions.
//!
//! This module provides different output modes (Standard, JSON, CI)
//! for different use cases.

use serde::{Deserialize, Serialize};

/// Output mode for the console.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OutputMode {
    /// Standard interactive output with colors and animations.
    #[default]
    Standard,
    /// Quiet mode - only errors.
    Quiet,
    /// Verbose mode - additional debug information.
    Verbose,
    /// JSON output for programmatic consumption.
    Json,
    /// CI mode - simplified output with CI annotations.
    Ci,
}

impl OutputMode {
    /// Check if this mode should show progress animations.
    pub fn show_progress(&self) -> bool {
        matches!(self, OutputMode::Standard | OutputMode::Verbose)
    }

    /// Check if this mode should show colors.
    pub fn show_colors(&self) -> bool {
        !matches!(self, OutputMode::Json | OutputMode::Quiet)
    }

    /// Check if this mode should show debug messages.
    pub fn show_debug(&self) -> bool {
        matches!(self, OutputMode::Verbose)
    }
}

/// JSON output format for structured logging.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonOutput {
    /// Log level.
    pub level: String,
    /// Message content.
    pub message: String,
    /// Timestamp (ISO 8601).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,
    /// Additional context.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<serde_json::Value>,
}

impl JsonOutput {
    /// Create a new JSON output.
    pub fn new(level: &str, message: &str) -> Self {
        Self {
            level: level.to_string(),
            message: message.to_string(),
            timestamp: Some(chrono_now()),
            context: None,
        }
    }

    /// Create an info message.
    pub fn info(message: &str) -> Self {
        Self::new("info", message)
    }

    /// Create a success message.
    pub fn success(message: &str) -> Self {
        Self::new("success", message)
    }

    /// Create a warning message.
    pub fn warn(message: &str) -> Self {
        Self::new("warn", message)
    }

    /// Create an error message.
    pub fn error(message: &str) -> Self {
        Self::new("error", message)
    }

    /// Create a debug message.
    pub fn debug(message: &str) -> Self {
        Self::new("debug", message)
    }

    /// Add context to the output.
    pub fn with_context(mut self, context: serde_json::Value) -> Self {
        self.context = Some(context);
        self
    }

    /// Convert to JSON string.
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| {
            format!(
                r#"{{"level":"{}","message":"{}"}}"#,
                self.level, self.message
            )
        })
    }
}

/// Get current timestamp in ISO 8601 format.
fn chrono_now() -> String {
    // Simple timestamp without chrono dependency
    use std::time::{SystemTime, UNIX_EPOCH};
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}.{:03}Z", duration.as_secs(), duration.subsec_millis())
}

/// CI-specific output formatting.
pub struct CiOutput;

impl CiOutput {
    /// Format a GitHub Actions group start.
    pub fn github_group_start(name: &str) -> String {
        format!("::group::{}", name)
    }

    /// Format a GitHub Actions group end.
    pub fn github_group_end() -> String {
        "::endgroup::".to_string()
    }

    /// Format a GitHub Actions error annotation.
    pub fn github_error(message: &str, file: Option<&str>, line: Option<u32>) -> String {
        let mut annotation = "::error".to_string();
        if let Some(f) = file {
            annotation.push_str(&format!(" file={}", f));
            if let Some(l) = line {
                annotation.push_str(&format!(",line={}", l));
            }
        }
        format!("{}::{}", annotation, message)
    }

    /// Format a GitHub Actions warning annotation.
    pub fn github_warning(message: &str, file: Option<&str>, line: Option<u32>) -> String {
        let mut annotation = "::warning".to_string();
        if let Some(f) = file {
            annotation.push_str(&format!(" file={}", f));
            if let Some(l) = line {
                annotation.push_str(&format!(",line={}", l));
            }
        }
        format!("{}::{}", annotation, message)
    }

    /// Format a GitHub Actions notice annotation.
    pub fn github_notice(message: &str) -> String {
        format!("::notice::{}", message)
    }

    /// Format a GitHub Actions debug message.
    pub fn github_debug(message: &str) -> String {
        format!("::debug::{}", message)
    }

    /// Set a GitHub Actions output variable.
    pub fn github_set_output(name: &str, value: &str) -> String {
        format!("::set-output name={}::{}", name, value)
    }

    /// Mask a value in GitHub Actions logs.
    pub fn github_mask(value: &str) -> String {
        format!("::add-mask::{}", value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_output_mode_show_progress() {
        assert!(OutputMode::Standard.show_progress());
        assert!(OutputMode::Verbose.show_progress());
        assert!(!OutputMode::Quiet.show_progress());
        assert!(!OutputMode::Json.show_progress());
        assert!(!OutputMode::Ci.show_progress());
    }

    #[test]
    fn test_json_output() {
        let output = JsonOutput::info("test message");
        let json = output.to_json();
        assert!(json.contains("info"));
        assert!(json.contains("test message"));
    }

    #[test]
    fn test_ci_output_github() {
        assert_eq!(CiOutput::github_group_start("test"), "::group::test");
        assert_eq!(CiOutput::github_group_end(), "::endgroup::");
        assert!(CiOutput::github_error("error", Some("file.rs"), Some(10)).contains("file=file.rs"));
    }
}
