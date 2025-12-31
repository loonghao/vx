//! Terminal tests.

use rstest::rstest;
use vx_console::{CiEnvironment, Term, TerminalType};

#[rstest]
fn test_term_detect() {
    let term = Term::detect();
    // Just verify it doesn't panic
    let _ = term.supports_color();
    let _ = term.supports_unicode();
    let _ = term.is_interactive();
    let _ = term.is_tty();
    let _ = term.terminal_type();
}

#[rstest]
fn test_term_minimal() {
    let term = Term::minimal();
    assert!(!term.supports_color());
    assert!(!term.supports_unicode());
    assert!(!term.is_interactive());
    assert!(!term.supports_hyperlinks());
    assert_eq!(term.terminal_type(), TerminalType::Unknown);
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
    assert!(CiEnvironment::Buildkite.supports_color());
    // Generic CI may not support colors
    assert!(!CiEnvironment::Generic.supports_color());
    assert!(!CiEnvironment::Jenkins.supports_color());
}

#[rstest]
fn test_ci_environment_name() {
    assert_eq!(CiEnvironment::GitHubActions.name(), "GitHub Actions");
    assert_eq!(CiEnvironment::GitLabCi.name(), "GitLab CI");
    assert_eq!(CiEnvironment::Jenkins.name(), "Jenkins");
    assert_eq!(CiEnvironment::AzurePipelines.name(), "Azure Pipelines");
    assert_eq!(CiEnvironment::CircleCi.name(), "CircleCI");
    assert_eq!(CiEnvironment::TravisCi.name(), "Travis CI");
    assert_eq!(
        CiEnvironment::BitbucketPipelines.name(),
        "Bitbucket Pipelines"
    );
    assert_eq!(CiEnvironment::TeamCity.name(), "TeamCity");
    assert_eq!(CiEnvironment::Buildkite.name(), "Buildkite");
    assert_eq!(CiEnvironment::Generic.name(), "CI");
}

#[rstest]
fn test_term_is_ci() {
    let term = Term::minimal();
    // Minimal term has no CI environment
    assert!(!term.is_ci());
    assert!(term.ci_environment().is_none());
}

#[rstest]
fn test_terminal_type_detect() {
    let _ = TerminalType::detect();
}

#[rstest]
fn test_terminal_type_supports_hyperlinks() {
    assert!(TerminalType::WindowsTerminal.supports_hyperlinks());
    assert!(TerminalType::ITerm2.supports_hyperlinks());
    assert!(TerminalType::VSCode.supports_hyperlinks());
    assert!(TerminalType::WezTerm.supports_hyperlinks());
    assert!(TerminalType::Kitty.supports_hyperlinks());
    assert!(!TerminalType::WindowsConsole.supports_hyperlinks());
    assert!(!TerminalType::Unix.supports_hyperlinks());
    assert!(!TerminalType::Unknown.supports_hyperlinks());
}

#[rstest]
fn test_terminal_type_supports_unicode() {
    assert!(TerminalType::WindowsTerminal.supports_unicode());
    assert!(TerminalType::ITerm2.supports_unicode());
    assert!(TerminalType::Unix.supports_unicode());
    assert!(!TerminalType::WindowsConsole.supports_unicode());
    assert!(!TerminalType::Unknown.supports_unicode());
}

#[rstest]
fn test_enable_ansi_support() {
    // Just verify it doesn't panic
    let _ = Term::enable_ansi_support();
}
