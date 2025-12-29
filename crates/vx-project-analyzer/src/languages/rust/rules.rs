//! Rust script detection rules
//!
//! Defines rules for detecting common Rust scripts based on file presence.
//!
//! Note: `just` rules have been moved to the common module since it's
//! a language-agnostic tool.

use crate::languages::rules::ScriptRule;

/// All Rust script detection rules
///
/// Rules are evaluated by priority (highest first).
/// For each script name, only the highest priority matching rule is used.
pub const RUST_RULES: &[ScriptRule] = &[
    // =========================================================================
    // Task runners (Rust-specific)
    // =========================================================================
    ScriptRule::new("make", "cargo make", "Run cargo-make tasks")
        .triggers(&["Makefile.toml"])
        .priority(90),
    // =========================================================================
    // Build
    // =========================================================================
    ScriptRule::new("build", "cargo build", "Build the project")
        .triggers(&["Cargo.toml"])
        .priority(50),
    ScriptRule::new(
        "build-release",
        "cargo build --release",
        "Build release version",
    )
    .triggers(&["Cargo.toml"])
    .priority(50),
    // =========================================================================
    // Test
    // =========================================================================
    ScriptRule::new("test", "cargo nextest run", "Run tests with nextest")
        .triggers(&[".config/nextest.toml"])
        .priority(100),
    ScriptRule::new("test", "cargo test", "Run tests")
        .triggers(&["Cargo.toml"])
        .excludes(&[".config/nextest.toml"])
        .priority(50),
    // =========================================================================
    // Linting & Formatting
    // =========================================================================
    ScriptRule::new("lint", "cargo clippy", "Run clippy linter")
        .triggers(&["Cargo.toml"])
        .priority(50),
    ScriptRule::new("format", "cargo fmt", "Format code")
        .triggers(&["Cargo.toml"])
        .priority(50),
    ScriptRule::new("check", "cargo check", "Check compilation")
        .triggers(&["Cargo.toml"])
        .priority(50),
    // =========================================================================
    // Documentation
    // =========================================================================
    ScriptRule::new("doc", "cargo doc", "Generate documentation")
        .triggers(&["Cargo.toml"])
        .priority(50),
    // =========================================================================
    // Benchmarks
    // =========================================================================
    ScriptRule::new("bench", "cargo bench", "Run benchmarks")
        .triggers(&["benches"])
        .priority(50),
];
