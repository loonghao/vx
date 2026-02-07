//! Execution report model
//!
//! Defines the JSON structure written to `~/.vx/metrics/`.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::exporter::SpanRecord;

/// Top-level metrics report for a single vx command execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandMetrics {
    /// Schema version
    pub version: String,
    /// ISO 8601 timestamp when the command started
    pub timestamp: String,
    /// The command that was executed (e.g., "vx node --version")
    pub command: String,
    /// Process exit code (None if still running)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exit_code: Option<i32>,
    /// Total wall-clock duration in milliseconds
    pub total_duration_ms: f64,
    /// Per-stage timing breakdown
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub stages: HashMap<String, StageMetrics>,
    /// All collected OpenTelemetry spans
    pub spans: Vec<SpanRecord>,
}

/// Metrics for a single pipeline stage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageMetrics {
    /// Duration in milliseconds
    pub duration_ms: f64,
    /// Whether the stage was successful
    pub success: bool,
    /// Optional error message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl CommandMetrics {
    /// Create a new metrics report for a command.
    pub fn new(command: String) -> Self {
        Self {
            version: "1".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            command,
            exit_code: None,
            total_duration_ms: 0.0,
            stages: HashMap::new(),
            spans: Vec::new(),
        }
    }

    /// Extract stage timings from collected spans.
    ///
    /// Looks for spans named "resolve", "ensure", "prepare", "execute"
    /// (the four pipeline stages) and extracts their durations.
    pub fn extract_stages_from_spans(&mut self) {
        let stage_names = ["resolve", "ensure", "prepare", "execute"];

        for span in &self.spans {
            let name_lower = span.name.to_lowercase();
            for stage_name in &stage_names {
                if name_lower.contains(stage_name) && !self.stages.contains_key(*stage_name) {
                    let success = !span.status.starts_with("error");
                    let error = if span.status.starts_with("error") {
                        Some(span.status.clone())
                    } else {
                        None
                    };
                    self.stages.insert(
                        stage_name.to_string(),
                        StageMetrics {
                            duration_ms: span.duration_ms,
                            success,
                            error,
                        },
                    );
                    break;
                }
            }
        }
    }

    /// Compute total duration from the root span, or sum of stages.
    pub fn compute_total_duration(&mut self) {
        // Look for a root span (parent_span_id is all zeros)
        let root_span = self
            .spans
            .iter()
            .find(|s| s.parent_span_id == "0000000000000000");

        if let Some(root) = root_span {
            self.total_duration_ms = root.duration_ms;
        } else {
            // Sum all stages
            self.total_duration_ms = self.stages.values().map(|s| s.duration_ms).sum();
        }
    }
}
