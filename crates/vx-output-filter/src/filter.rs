//! Core output filter — per-line processing logic.
//!
//! The `OutputFilter` applies deduplication, blank-run collapsing, and
//! line-budget enforcement to a stream of text lines from a subprocess.

use crate::rules::{is_error_line, strip_ansi};

/// Configuration for the output filter.
#[derive(Debug, Clone)]
pub struct OutputFilterConfig {
    /// Collapse ≥N consecutive identical lines into one summary.
    pub dedup_threshold: usize,

    /// Truncate after this many total emitted lines (None = unlimited).
    pub max_lines: Option<usize>,

    /// Collapse runs of multiple blank lines into a single blank line.
    pub strip_empty_runs: bool,
}

impl OutputFilterConfig {
    /// Sensible compact defaults used in AI-agent / CI mode.
    pub fn compact_defaults() -> Self {
        Self {
            dedup_threshold: 3,
            max_lines: Some(200),
            strip_empty_runs: true,
        }
    }

    /// Return `Some(compact_defaults())` when `VX_OUTPUT=compact` and stdout
    /// is **not** a TTY, otherwise `None`.
    pub fn from_env() -> Option<Self> {
        use std::io::IsTerminal;
        let is_compact = matches!(
            std::env::var("VX_OUTPUT").as_deref(),
            Ok("compact") | Ok("Compact") | Ok("COMPACT")
        );
        if is_compact && !std::io::stdout().is_terminal() {
            Some(Self::compact_defaults())
        } else {
            None
        }
    }
}

/// Stateful per-stream filter.
///
/// Call [`filter_line`] for each output line; call [`finalize`] at the end
/// to flush any pending dedup summary.
pub struct OutputFilter {
    config: OutputFilterConfig,
    /// Last line content seen (after ANSI strip)
    last_line: Option<String>,
    /// How many times the last line has repeated
    repeat_count: usize,
    /// Total lines emitted so far
    emitted: usize,
    /// Whether we have hit max_lines
    truncated: bool,
    /// How many lines were dropped due to truncation
    truncated_count: usize,
    /// Whether the previous emitted line was blank
    last_was_blank: bool,
}

impl OutputFilter {
    /// Create a new filter with the given configuration.
    pub fn new(config: OutputFilterConfig) -> Self {
        Self {
            config,
            last_line: None,
            repeat_count: 0,
            emitted: 0,
            truncated: false,
            truncated_count: 0,
            last_was_blank: false,
        }
    }

    /// Process one line. Returns zero, one, or two lines to emit.
    pub fn filter_line(&mut self, raw: &str) -> Vec<String> {
        if self.truncated {
            self.truncated_count += 1;
            return vec![];
        }

        let clean = strip_ansi(raw);
        let is_blank = clean.trim().is_empty();

        // Collapse blank runs
        if is_blank && self.config.strip_empty_runs && self.last_was_blank {
            return vec![];
        }

        // Dedup: error lines bypass dedup and always emit
        let is_err = is_error_line(&clean);

        let mut out: Vec<String> = Vec::new();

        if !is_err {
            if let Some(ref prev) = self.last_line {
                if *prev == clean {
                    self.repeat_count += 1;
                    if self.repeat_count < self.config.dedup_threshold {
                        // Below threshold — emit normally
                    } else {
                        // At or above threshold — swallow; will emit summary later
                        return vec![];
                    }
                } else {
                    // Line changed — flush dedup summary if needed
                    if self.repeat_count >= self.config.dedup_threshold {
                        let extra = self.repeat_count - self.config.dedup_threshold + 1;
                        if extra > 0 {
                            out.push(format!("  ... (+{extra} identical lines omitted)"));
                        }
                    }
                    self.last_line = Some(clean.clone());
                    self.repeat_count = 1;
                }
            } else {
                self.last_line = Some(clean.clone());
                self.repeat_count = 1;
            }
        }

        out.push(clean.clone());
        self.last_was_blank = is_blank;

        // Apply max_lines budget
        self.emit_lines(out)
    }

    fn emit_lines(&mut self, lines: Vec<String>) -> Vec<String> {
        let Some(max) = self.config.max_lines else {
            self.emitted += lines.len();
            return lines;
        };
        let mut result = Vec::new();
        for l in lines {
            if self.emitted < max {
                self.emitted += 1;
                result.push(l);
            } else {
                self.truncated = true;
                self.truncated_count += 1;
            }
        }
        result
    }

    /// Flush any pending state and return final summary lines.
    pub fn finalize(&mut self) -> Vec<String> {
        let mut out = Vec::new();

        // Flush dedup summary for the last run
        if let Some(ref _prev) = self.last_line {
            if self.repeat_count >= self.config.dedup_threshold {
                let extra = self.repeat_count - self.config.dedup_threshold + 1;
                if extra > 0 {
                    out.push(format!("  ... (+{extra} identical lines omitted)"));
                }
            }
        }

        // Truncation summary
        if self.truncated_count > 0 {
            out.push(format!(
                "  ... (+{} lines omitted, use default mode to see all output)",
                self.truncated_count
            ));
        }

        out
    }
}
