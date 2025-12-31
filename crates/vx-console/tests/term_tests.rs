//! Terminal tests.

use rstest::rstest;
use vx_console::{CiEnvironment, Term};

#[rstest]
fn test_term_detect() {
    let term = Term::detect();
    // Just verify it doesn't panic
    let _ = term.supports_color();
    let _ = term.supports_unicode();
    let _ = term.is_interactive();
    let _ = term.is_tty();
}

#[rstest]
fn test_term_minimal() {
    let term = Term::minimal();
    assert!(!term.supports_color());
    assert!(!term.supports_unicode());
    assert!(!term.is_interactive());
    assert!(!term.supports_hyperlinks());
}

#[rstest]
fn test_term_size() {
    let term = Term::detect();
    // Size may or may not be available
    let _ = term.size();
    let _ = term.width();
    let _ = term.height();
}

#[rstest]
fn test_term_capabilities() {
    let term = Term::detect();
    let caps = term.capabilities();
    // Just verify we can access capabilities
    let _ = caps.color;
    let _ = caps.unicode;
    let _ = caps.interactive;
    let _ = caps.hyperlinks;
}

#[rstest]
fn test_term_hyperlink_no_support() {
    let term = Term::minimal();
    let result = term.hyperlink("https://example.com", "Example");
    assert_eq!(result, "Example");
}

#[rstest]
fn test_ci_environment_supports_color() {
    assert!(CiEnvironment::GitHubActions.supports_color());
    assert!(CiEnvironment::GitLabCi.supports_color());
    assert!(CiEnvironment::AzurePipelines.supports_color());
    assert!(CiEnvironment::CircleCi.supports_color());
    assert!(CiEnvironment::TravisCi.supports_color());
    // Generic CI may not support colors
    assert!(!CiEnvironment::Generic.supports_color());
    assert!(!CiEnvironment::Jenkins.supports_color());
}

#[rstest]
fn test_term_is_ci() {
    let term = Term::minimal();
    // Minimal term has no CI environment
    assert!(!term.is_ci());
    assert!(term.ci_environment().is_none());
}
