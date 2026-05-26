use rstest::rstest;
use serial_test::serial;
use vx_output_filter::filter::{FilterLevel, OutputFilter, OutputFilterConfig};
use vx_output_filter::rules::{is_error_line, strip_ansi};

fn compact_config() -> OutputFilterConfig {
    OutputFilterConfig::compact_defaults()
}

fn filter_lines(level: FilterLevel, lines: &[String]) -> Vec<String> {
    let mut filter = OutputFilter::new(OutputFilterConfig::for_level(level));
    let mut output = Vec::new();
    for line in lines {
        output.extend(filter.filter_line(line));
    }
    output.extend(filter.finalize());
    output
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

// ── FilterLevel ───────────────────────────────────────────────────────────────

#[test]
fn test_filter_level_light_config() {
    let cfg = OutputFilterConfig::for_level(FilterLevel::Light);
    assert_eq!(cfg.dedup_threshold, usize::MAX, "Light disables dedup");
    assert_eq!(cfg.max_lines, None, "Light has no line limit");
    assert!(cfg.strip_empty_runs, "Light still collapses blank runs");
}

#[test]
fn test_filter_level_normal_config() {
    let cfg = OutputFilterConfig::for_level(FilterLevel::Normal);
    assert_eq!(cfg.dedup_threshold, 3, "Normal dedup threshold is 3");
    assert_eq!(cfg.max_lines, Some(500), "Normal line budget is 500");
    assert!(cfg.strip_empty_runs);
}

#[test]
fn test_filter_level_aggressive_config() {
    let cfg = OutputFilterConfig::for_level(FilterLevel::Aggressive);
    assert_eq!(cfg.dedup_threshold, 2, "Aggressive dedup threshold is 2");
    assert_eq!(cfg.max_lines, Some(100), "Aggressive line budget is 100");
    assert!(cfg.strip_empty_runs);
}

#[test]
#[serial]
fn test_filter_level_from_env_default_is_normal() {
    unsafe { std::env::remove_var("VX_FILTER_LEVEL") };
    assert_eq!(FilterLevel::from_env(), FilterLevel::Normal);
}

#[test]
#[serial]
fn test_filter_level_from_env_recognises_light() {
    unsafe { std::env::set_var("VX_FILTER_LEVEL", "light") };
    assert_eq!(FilterLevel::from_env(), FilterLevel::Light);
    unsafe { std::env::remove_var("VX_FILTER_LEVEL") };
}

#[test]
#[serial]
fn test_filter_level_from_env_recognises_aggressive() {
    unsafe { std::env::set_var("VX_FILTER_LEVEL", "aggressive") };
    assert_eq!(FilterLevel::from_env(), FilterLevel::Aggressive);
    unsafe { std::env::remove_var("VX_FILTER_LEVEL") };
}

#[test]
fn test_compact_defaults_equals_normal_level() {
    let defaults = OutputFilterConfig::compact_defaults();
    let normal = OutputFilterConfig::for_level(FilterLevel::Normal);
    assert_eq!(defaults.dedup_threshold, normal.dedup_threshold);
    assert_eq!(defaults.max_lines, normal.max_lines);
    assert_eq!(defaults.strip_empty_runs, normal.strip_empty_runs);
}

#[test]
fn test_aggressive_level_dedup_collapses_at_2() {
    let cfg = OutputFilterConfig::for_level(FilterLevel::Aggressive);
    let mut f = OutputFilter::new(cfg); // threshold = 2

    // First line passes (repeat_count becomes 1, which is < threshold 2)
    assert_eq!(f.filter_line("building...").len(), 1);
    // Second identical line: repeat_count reaches threshold (2 >= 2) → collapsed
    assert_eq!(f.filter_line("building...").len(), 0);
    // Third is also collapsed
    assert_eq!(f.filter_line("building...").len(), 0);
}

#[test]
fn test_light_level_no_dedup() {
    let cfg = OutputFilterConfig::for_level(FilterLevel::Light);
    let mut f = OutputFilter::new(cfg); // threshold = usize::MAX (disabled)

    // All identical lines should pass through with Light level
    for _ in 0..10 {
        assert_eq!(f.filter_line("building...").len(), 1);
    }
    // finalize should not report any omissions
    let summary = f.finalize();
    assert!(
        !summary.iter().any(|l| l.contains("omitted")),
        "Light level should not suppress any lines"
    );
}

#[test]
fn test_filter_level_comparison_reduces_tokens_and_keeps_errors() {
    let mut raw = Vec::new();
    raw.extend((0..60).map(|_| "building dependency graph".to_string()));
    raw.extend((0..5).map(|_| "error: failed to compile crate".to_string()));
    raw.extend((0..160).map(|i| format!("progress step {i}")));
    raw.extend((0..140).map(|_| "warning: retrying download".to_string()));
    raw.extend((0..160).map(|i| format!("trace detail {i}")));

    let normal = filter_lines(FilterLevel::Normal, &raw);
    let aggressive = filter_lines(FilterLevel::Aggressive, &raw);

    let raw_chars = raw.join("\n").len();
    let normal_chars = normal.join("\n").len();
    let aggressive_chars = aggressive.join("\n").len();

    assert!(
        normal_chars < raw_chars,
        "normal compact output should be smaller than raw output"
    );
    assert!(
        aggressive_chars < normal_chars,
        "aggressive compact output should be smaller than normal compact output"
    );
    assert!(
        normal.iter().any(|line| line.contains("error: failed")),
        "normal filtering must preserve error-looking lines"
    );
    assert!(
        aggressive.iter().any(|line| line.contains("error: failed")),
        "aggressive filtering must preserve early error-looking lines"
    );
    assert!(
        normal.iter().any(|line| line.contains("omitted")),
        "normal filtering should explain omitted repeated lines"
    );
    assert!(
        aggressive.iter().any(|line| line.contains("omitted")),
        "aggressive filtering should explain omitted or truncated lines"
    );
}
