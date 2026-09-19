//! Regression tests for the `yazi` provider version pattern.
//!
//! `yazi --version` changed its output shape in 26.9.1:
//!
//! - up to 25.x it printed a single line, `Yazi 25.5.31 (<hash> <date>)`
//! - from 26.9.1 it prints a multi-line banner where the version sits on the
//!   second line behind a `Version:` label:
//!
//! ```text
//! Yazi
//!     Version: 26.9.1 (8dd895c 2026-09-01)
//!     Debug  : false
//!     Triple : x86_64-unknown-linux-gnu (linux-x86_64)
//!     Rustc  : 1.98.0 (88d9e12a 2026-08-18)
//! ```
//!
//! The provider's `version_pattern` is compiled as a regex and matched against
//! stdout/stderr, so it must accept both layouts and must keep requiring the
//! full `X.Y[.Z]` version rather than stopping at the major number.

use regex::Regex;
use vx_starlark::StarlarkProvider;

/// Multi-line banner emitted by `yazi --version` since 26.9.1.
const MULTILINE_OUTPUT: &str = "\
Yazi
    Version: 26.9.1 (8dd895c 2026-09-01)
    Debug  : false
    Triple : x86_64-unknown-linux-gnu (linux-x86_64)
    Rustc  : 1.98.0 (88d9e12a 2026-08-18)
";

/// Same banner with CRLF line endings, as produced on Windows.
const MULTILINE_OUTPUT_CRLF: &str =
    "Yazi\r\n    Version: 26.9.1 (8dd895c 2026-09-01)\r\n    Debug  : false\r\n";

/// Single-line output emitted by yazi 25.x and earlier.
const LEGACY_SINGLE_LINE_OUTPUT: &str = "Yazi 25.5.31 (e7d1a2b 2025-05-31)\n";

/// Load the `yazi` provider and return the `expected_output` pattern of its
/// `version_check` test command.
async fn yazi_version_pattern() -> String {
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let star_path = manifest_dir
        .parent() // crates/
        .expect("vx-starlark must live under crates/")
        .join("vx-providers")
        .join("yazi")
        .join("provider.star");

    let provider = StarlarkProvider::load(&star_path)
        .await
        .expect("yazi provider.star must load");

    let runtime = provider
        .runtimes()
        .iter()
        .find(|rt| rt.name == "yazi")
        .expect("yazi provider must define a `yazi` runtime");

    let command = runtime
        .test_commands
        .iter()
        .find(|cmd| cmd.name.as_deref() == Some("version_check"))
        .expect("yazi runtime must define a `version_check` test command");

    command
        .expected_output
        .clone()
        .expect("version_check must declare an expected_output pattern")
}

#[tokio::test]
async fn test_yazi_version_pattern_matches_multiline_output() {
    let pattern = yazi_version_pattern().await;
    let re = Regex::new(&pattern).expect("version_pattern must be a valid regex");

    assert!(
        re.is_match(MULTILINE_OUTPUT),
        "pattern {:?} did not match the multi-line yazi >= 26.9.1 output",
        pattern
    );
}

#[tokio::test]
async fn test_yazi_version_pattern_matches_multiline_output_crlf() {
    let pattern = yazi_version_pattern().await;
    let re = Regex::new(&pattern).expect("version_pattern must be a valid regex");

    assert!(
        re.is_match(MULTILINE_OUTPUT_CRLF),
        "pattern {:?} did not match the CRLF multi-line output",
        pattern
    );
}

#[tokio::test]
async fn test_yazi_version_pattern_matches_legacy_single_line_output() {
    let pattern = yazi_version_pattern().await;
    let re = Regex::new(&pattern).expect("version_pattern must be a valid regex");

    assert!(
        re.is_match(LEGACY_SINGLE_LINE_OUTPUT),
        "pattern {:?} did not match the legacy single-line output",
        pattern
    );
}

#[tokio::test]
async fn test_yazi_version_pattern_requires_full_version() {
    let pattern = yazi_version_pattern().await;
    let re = Regex::new(&pattern).expect("version_pattern must be a valid regex");

    // A major number alone must not satisfy the pattern: matching it would hide
    // a truncated or missing version string.
    let major_only_single_line = "Yazi 25\n";
    assert!(
        !re.is_match(major_only_single_line),
        "pattern {:?} must not match a bare major version",
        pattern
    );

    let major_only_multiline = "Yazi\n    Version: 25\n";
    assert!(
        !re.is_match(major_only_multiline),
        "pattern {:?} must not match a bare major version in the banner",
        pattern
    );
}

#[tokio::test]
async fn test_yazi_version_pattern_rejects_unrelated_output() {
    let pattern = yazi_version_pattern().await;
    let re = Regex::new(&pattern).expect("version_pattern must be a valid regex");

    assert!(
        !re.is_match("Yazi\n    Debug  : false\n"),
        "pattern {:?} must not match a banner without a version line",
        pattern
    );
    assert!(
        !re.is_match("command not found: yazi\n"),
        "pattern {:?} must not match unrelated output",
        pattern
    );
}
