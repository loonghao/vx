//! Core output filter — per-line processing logic.
//!
//! The `OutputFilter` applies deduplication, blank-run collapsing, and
//! line-budget enforcement to a stream of text lines from a subprocess.

use crate::rules::{is_error_line, strip_ansi};

/// Filter aggressiveness level — controls how much output is suppressed.
///
/// Select a level based on how noisy the tool output typically is:
///
/// | Level       | Dedup threshold | Max lines | Use case                              |
/// |-------------|-----------------|-----------|---------------------------------------|
/// | Light       | disabled        | unlimited | Verbose tools where every line counts |
/// | Normal      | ≥3 identical    | 500       | Default for most tools                |
/// | Aggressive  | ≥2 identical    | 100       | Very noisy tools (e.g. `cargo build`) |
///
/// Set via `VX_FILTER_LEVEL=light|normal|aggressive` or `--filter-level` CLI flag.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FilterLevel {
    /// Light: ANSI stripping and blank-run collapsing only. No dedup, no truncation.
    Light,
    /// Normal: dedup (≥3 identical lines) + 500-line budget. **Default.**
    #[default]
    Normal,
    /// Aggressive: dedup (≥2 identical lines) + 100-line budget.
    Aggressive,
}

impl FilterLevel {
    /// Parse the filter level from the `VX_FILTER_LEVEL` environment variable.
    ///
    /// Falls back to `Normal` if the variable is absent or unrecognised.
    pub fn from_env() -> Self {
        match std::env::var("VX_FILTER_LEVEL").as_deref() {
            Ok("light") | Ok("Light") | Ok("LIGHT") => FilterLevel::Light,
            Ok("aggressive") | Ok("Aggressive") | Ok("AGGRESSIVE") => FilterLevel::Aggressive,
            _ => FilterLevel::Normal,
        }
    }
}

/// Configuration for the output filter.
#[derive(Debug, Clone, Default)]
pub struct OutputFilterConfig {
    /// Collapse ≥N consecutive identical lines into one summary.
    /// Set to `usize::MAX` to disable deduplication (Light level).
    pub dedup_threshold: usize,

    /// Truncate after this many total emitted lines (None = unlimited).
    pub max_lines: Option<usize>,

    /// Collapse runs of multiple blank lines into a single blank line.
    pub strip_empty_runs: bool,
}

impl OutputFilterConfig {
    /// Build config for the given aggressiveness level.
    ///
    /// # Examples
    /// ```
    /// use vx_output_filter::{FilterLevel, OutputFilterConfig};
    /// let cfg = OutputFilterConfig::for_level(FilterLevel::Aggressive);
    /// assert_eq!(cfg.dedup_threshold, 2);
    /// assert_eq!(cfg.max_lines, Some(100));
    /// ```
    pub fn for_level(level: FilterLevel) -> Self {
        match level {
            FilterLevel::Light => Self {
                dedup_threshold: usize::MAX, // disabled
                max_lines: None,             // unlimited
                strip_empty_runs: true,
            },
            FilterLevel::Normal => Self {
                dedup_threshold: 3,
                max_lines: Some(500),
                strip_empty_runs: true,
            },
            FilterLevel::Aggressive => Self {
                dedup_threshold: 2,
                max_lines: Some(100),
                strip_empty_runs: true,
            },
        }
    }

    /// Sensible compact defaults (Normal level). Alias for `for_level(Normal)`.
    pub fn compact_defaults() -> Self {
        Self::for_level(FilterLevel::Normal)
    }

    /// Return `Some(config)` when `VX_OUTPUT=compact` and stdout is **not** a TTY,
    /// otherwise `None`. The level is read from `VX_FILTER_LEVEL` (default: Normal).
    pub fn from_env() -> Option<Self> {
        use std::io::IsTerminal;
        let is_compact = matches!(
            std::env::var("VX_OUTPUT").as_deref(),
            Ok("compact") | Ok("Compact") | Ok("COMPACT")
        );
        if is_compact && !std::io::stdout().is_terminal() {
            Some(Self::for_level(FilterLevel::from_env()))
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
                    // At or above threshold — swallow; will emit summary on next change
                    if self.repeat_count >= self.config.dedup_threshold {
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
        if self.last_line.is_some() && self.repeat_count >= self.config.dedup_threshold {
            let extra = self.repeat_count - self.config.dedup_threshold + 1;
            if extra > 0 {
                out.push(format!("  ... (+{extra} identical lines omitted)"));
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
