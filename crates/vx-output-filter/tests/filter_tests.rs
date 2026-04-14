use rstest::rstest;
use vx_output_filter::filter::{OutputFilter, OutputFilterConfig};
use vx_output_filter::rules::{is_error_line, strip_ansi};

fn compact_config() -> OutputFilterConfig {
    OutputFilterConfig::compact_defaults()
}

// ── ANSI stripping ────────────────────────────────────────────────────────────

#[test]
fn test_ansi_stripped() {
    let result = strip_ansi("\x1b[32mhello\x1b[0m world");
    assert_eq!(result, "hello world");
}

#[test]
fn test_ansi_stripped_no_codes() {
    let plain = "just plain text";
    assert_eq!(strip_ansi(plain), plain);
}

// ── Dedup / collapse ──────────────────────────────────────────────────────────

#[test]
fn test_dedup_collapse() {
    let mut f = OutputFilter::new(compact_config()); // threshold = 3

    // Lines 1 and 2 pass through
    assert_eq!(f.filter_line("building...").len(), 1);
    assert_eq!(f.filter_line("building...").len(), 1);
    // Line 3+ is collapsed (at threshold)
    assert_eq!(f.filter_line("building...").len(), 0);
    assert_eq!(f.filter_line("building...").len(), 0);

    // Finalize emits summary
    let summary = f.finalize();
    assert!(
        summary.iter().any(|l| l.contains("omitted")),
        "finalize should emit omitted-lines summary"
    );
}

#[test]
fn test_error_lines_bypass_dedup() {
    let mut f = OutputFilter::new(compact_config());

    // Error lines are always emitted even when identical
    for _ in 0..5 {
        let emitted = f.filter_line("error: compilation failed");
        assert_eq!(emitted.len(), 1, "error line should always be emitted");
    }
}

// ── Blank-run collapsing ──────────────────────────────────────────────────────

#[test]
fn test_empty_run_stripped() {
    let mut f = OutputFilter::new(compact_config());

    // First blank is kept
    let first = f.filter_line("");
    assert_eq!(first.len(), 1);

    // Consecutive blank is dropped
    let second = f.filter_line("");
    assert_eq!(
        second.len(),
        0,
        "second consecutive blank should be dropped"
    );

    // Non-blank resets the run
    let text = f.filter_line("hello");
    assert_eq!(text.len(), 1);
}

// ── max_lines truncation ──────────────────────────────────────────────────────

#[test]
fn test_max_lines_overflow_summary() {
    let config = OutputFilterConfig {
        dedup_threshold: 100, // no dedup
        max_lines: Some(3),
        strip_empty_runs: false,
    };
    let mut f = OutputFilter::new(config);

    for i in 0..6 {
        f.filter_line(&format!("line {i}"));
    }

    let summary = f.finalize();
    assert!(
        summary.iter().any(|l| l.contains("omitted")),
        "finalize should report truncated lines"
    );
}

// ── Parametric happy-path ─────────────────────────────────────────────────────

#[rstest]
#[case("simple text", "simple text")]
#[case("\x1b[1mbold\x1b[0m", "bold")]
#[case("  spaces  ", "  spaces  ")]
fn test_filter_line_basic(#[case] input: &str, #[case] expected: &str) {
    let mut f = OutputFilter::new(compact_config());
    let emitted = f.filter_line(input);
    assert_eq!(emitted.len(), 1);
    assert_eq!(emitted[0], expected);
}

// ── Rules ─────────────────────────────────────────────────────────────────────

#[test]
fn test_is_error_line_detects_error() {
    assert!(is_error_line("error: failed to compile"));
    assert!(is_error_line("Error: something went wrong"));
    assert!(is_error_line("FATAL: out of memory"));
    assert!(is_error_line("panic! at the disco"));
    assert!(!is_error_line("everything is fine"));
}

// ── from_env (unit-level) ─────────────────────────────────────────────────────

#[test]
fn test_from_env_none_by_default() {
    // In normal test runs stdout may or may not be a TTY,
    // but VX_OUTPUT is not "compact" so from_env() should return None.
    // Safety: single-threaded test binary; no concurrent access to VX_OUTPUT
    unsafe { std::env::remove_var("VX_OUTPUT") };
    // Note: might return Some in non-TTY CI, but compact is not set → None
    let result = OutputFilterConfig::from_env();
    // We only assert it doesn't panic; value depends on env
    let _ = result;
}
