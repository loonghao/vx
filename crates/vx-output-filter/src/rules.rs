//! Line classification rules for output filtering.
//!
//! Provides ANSI stripping and error-line detection used by the output filter.

use regex::Regex;
use std::sync::OnceLock;

static ERROR_PATTERN: OnceLock<Regex> = OnceLock::new();
static ANSI_PATTERN: OnceLock<Regex> = OnceLock::new();

/// Returns `true` if the line looks like an error/fatal/panic message.
///
/// Error lines are always emitted by the filter regardless of dedup settings.
pub fn is_error_line(line: &str) -> bool {
    let re =
        ERROR_PATTERN.get_or_init(|| Regex::new(r"(?i)(error|fatal|panic|FAILED|Error:)").unwrap());
    re.is_match(line)
}

/// Strip ANSI escape sequences from a string.
pub fn strip_ansi(s: &str) -> String {
    let re = ANSI_PATTERN.get_or_init(|| Regex::new(r"\x1b\[[0-9;]*[mGKHF]").unwrap());
    re.replace_all(s, "").to_string()
}

/// Marker type — future expansion point for pluggable rules.
pub struct FilterRules;

#[cfg(test)]
mod inline_tests {
    use super::*;

    #[test]
    fn test_strip_ansi_removes_codes() {
        assert_eq!(strip_ansi("\x1b[32mhello\x1b[0m"), "hello");
    }

    #[test]
    fn test_is_error_line_detects_error() {
        assert!(is_error_line("error: failed to compile"));
        assert!(is_error_line("Error: something went wrong"));
        assert!(is_error_line("FATAL: out of memory"));
        assert!(is_error_line("panic! at the disco"));
        assert!(!is_error_line("everything is fine"));
    }
}
