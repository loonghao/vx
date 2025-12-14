//! CLI Snapshot Tests using trycmd
//!
//! This module uses trycmd for file-based CLI testing.
//! Test cases are defined in `.md` or `.trycmd` files under `tests/cmd/`.
//!
//! To update snapshots when CLI output changes:
//! ```bash
//! TRYCMD=overwrite cargo test --test cli_snapshots
//! ```
//!
//! To dump new snapshots for review:
//! ```bash
//! TRYCMD=dump cargo test --test cli_snapshots
//! ```

#[test]
fn cli_tests() {
    let t = trycmd::TestCases::new();
    t.case("tests/cmd/*.md");
    t.case("tests/cmd/**/*.md");
}

#[test]
fn cli_trycmd_tests() {
    let t = trycmd::TestCases::new();
    t.case("tests/cmd/*.trycmd");
    t.case("tests/cmd/**/*.trycmd");
}
