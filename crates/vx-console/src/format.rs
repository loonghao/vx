//! Output format definitions.
//!
//! This module provides different output modes (Standard, JSON, CI)
//! for different use cases.

use crate::shell::Verbosity;
use crate::term::CiEnvironment;
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

    /// Convert to Verbosity.
    pub fn to_verbosity(&self) -> Verbosity {
        match self {
            OutputMode::Quiet => Verbosity::Quiet,
            OutputMode::Verbose => Verbosity::Verbose,
            _ => Verbosity::Normal,
        }
    }

    /// Detect the best output mode based on environment.
    pub fn detect() -> Self {
        // RFC 0031: Check unified VX_OUTPUT env var first
        match std::env::var("VX_OUTPUT").as_deref() {
            Ok("json") => return OutputMode::Json,
            Ok("quiet") => return OutputMode::Quiet,
            Ok("verbose") => return OutputMode::Verbose,
            _ => {}
        }

        // Check for JSON mode (legacy)
        if std::env::var("VX_OUTPUT_JSON").is_ok() {
            return OutputMode::Json;
        }

        // Check for quiet mode
        if std::env::var("VX_QUIET").is_ok() {
            return OutputMode::Quiet;
        }

        // Check for verbose mode
        if std::env::var("VX_VERBOSE").is_ok() {
            return OutputMode::Verbose;
        }

        // Check for CI environment
        if CiEnvironment::detect().is_some() {
            return OutputMode::Ci;
        }

        OutputMode::Standard
    }
}

impl From<Verbosity> for OutputMode {
    fn from(v: Verbosity) -> Self {
        match v {
            Verbosity::Quiet => OutputMode::Quiet,
            Verbosity::Normal => OutputMode::Standard,
            Verbosity::Verbose => OutputMode::Verbose,
        }
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
    // ========== GitHub Actions ==========

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

    /// Set a GitHub Actions output variable (new format using GITHUB_OUTPUT).
    pub fn github_set_output(name: &str, value: &str) -> String {
        format!("{}={}", name, value)
    }

    /// Mask a value in GitHub Actions logs.
    pub fn github_mask(value: &str) -> String {
        format!("::add-mask::{}", value)
    }

    // ========== GitLab CI ==========

    /// Format a GitLab CI collapsible section start.
    pub fn gitlab_section_start(name: &str, header: &str) -> String {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        format!(
            "\x1b[0Ksection_start:{}:{}[collapsed=true]\r\x1b[0K{}",
            timestamp, name, header
        )
    }

    /// Format a GitLab CI collapsible section end.
    pub fn gitlab_section_end(name: &str) -> String {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        format!("\x1b[0Ksection_end:{}:{}\r\x1b[0K", timestamp, name)
    }

    // ========== Azure Pipelines ==========

    /// Format an Azure Pipelines task command.
    pub fn azure_task(command: &str, properties: &[(&str, &str)], message: &str) -> String {
        let props = if properties.is_empty() {
            String::new()
        } else {
            let prop_str: Vec<String> = properties
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect();
            format!(" {}", prop_str.join(";"))
        };
        format!("##vso[task.{}{}]{}", command, props, message)
    }

    /// Format an Azure Pipelines error.
    pub fn azure_error(message: &str, file: Option<&str>, line: Option<u32>) -> String {
        match (file, line) {
            (Some(f), Some(l)) => {
                format!(
                    "##vso[task.logissue type=error;sourcepath={};linenumber={}]{}",
                    f, l, message
                )
            }
            (Some(f), None) => {
                format!(
                    "##vso[task.logissue type=error;sourcepath={}]{}",
                    f, message
                )
            }
            _ => Self::azure_task("logissue", &[("type", "error")], message),
        }
    }

    /// Format an Azure Pipelines warning.
    pub fn azure_warning(message: &str) -> String {
        Self::azure_task("logissue", &[("type", "warning")], message)
    }

    /// Format an Azure Pipelines group start.
    pub fn azure_group_start(name: &str) -> String {
        format!("##[group]{}", name)
    }

    /// Format an Azure Pipelines group end.
    pub fn azure_group_end() -> String {
        "##[endgroup]".to_string()
    }

    // ========== Generic CI ==========

    /// Format output for any CI environment.
    pub fn format_for_ci(ci: CiEnvironment, level: &str, message: &str) -> String {
        match ci {
            CiEnvironment::GitHubActions => match level {
                "error" => Self::github_error(message, None, None),
                "warning" | "warn" => Self::github_warning(message, None, None),
                "debug" => Self::github_debug(message),
                _ => message.to_string(),
            },
            CiEnvironment::AzurePipelines => match level {
                "error" => Self::azure_error(message, None, None),
                "warning" | "warn" => Self::azure_warning(message),
                _ => message.to_string(),
            },
            _ => message.to_string(),
        }
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
