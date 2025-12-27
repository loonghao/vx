//! Team configuration tests
//!
//! Tests for team collaboration configuration parsing.

use rstest::rstest;
use vx_config::parse_config_str;

// ============================================
// Team Config Parsing Tests
// ============================================

#[test]
fn test_parse_team_config_basic() {
    let content = r#"
[team]
extends = "https://example.com/team-preset.toml"
"#;
    let config = parse_config_str(content).unwrap();
    let team = config.team.unwrap();
    assert_eq!(
        team.extends,
        Some("https://example.com/team-preset.toml".to_string())
    );
}

#[test]
fn test_parse_team_code_owners() {
    let content = r#"
[team.code_owners]
default_owners = ["@default-team"]

[team.code_owners.paths]
"*.rs" = ["@rust-team"]
"*.ts" = ["@frontend-team"]
"/docs/" = ["@docs-team", "@tech-writers"]
"#;
    let config = parse_config_str(content).unwrap();
    let team = config.team.unwrap();
    let code_owners = team.code_owners.unwrap();

    assert_eq!(
        code_owners.default_owners,
        vec!["@default-team".to_string()]
    );
    assert_eq!(
        code_owners.paths.get("*.rs"),
        Some(&vec!["@rust-team".to_string()])
    );
    assert_eq!(
        code_owners.paths.get("*.ts"),
        Some(&vec!["@frontend-team".to_string()])
    );
    assert_eq!(
        code_owners.paths.get("/docs/"),
        Some(&vec!["@docs-team".to_string(), "@tech-writers".to_string()])
    );
}

#[test]
fn test_parse_team_review_config() {
    let content = r#"
[team.review]
required_approvals = 2
dismiss_stale = true
require_code_owner = true
"#;
    let config = parse_config_str(content).unwrap();
    let team = config.team.unwrap();
    let review = team.review.unwrap();

    assert_eq!(review.required_approvals, Some(2));
    assert_eq!(review.dismiss_stale, Some(true));
    assert_eq!(review.require_code_owner, Some(true));
}

#[test]
fn test_parse_team_conventions_commit_format() {
    let content = r#"
[team.conventions]
commit_format = "conventional"
commit_pattern = "^(feat|fix|docs|style|refactor|test|chore)(\\(.+\\))?: .+"
"#;
    let config = parse_config_str(content).unwrap();
    let team = config.team.unwrap();
    let conventions = team.conventions.unwrap();

    assert_eq!(conventions.commit_format, Some("conventional".to_string()));
    assert!(conventions.commit_pattern.is_some());
}

#[test]
fn test_parse_team_conventions_branch_pattern() {
    let content = r#"
[team.conventions]
branch_pattern = "^(feature|bugfix|hotfix|release)/[a-z0-9-]+$"
linear_history = true
"#;
    let config = parse_config_str(content).unwrap();
    let team = config.team.unwrap();
    let conventions = team.conventions.unwrap();

    assert_eq!(
        conventions.branch_pattern,
        Some("^(feature|bugfix|hotfix|release)/[a-z0-9-]+$".to_string())
    );
    assert_eq!(conventions.linear_history, Some(true));
}

#[test]
fn test_parse_team_conventions_merge_strategies() {
    let content = r#"
[team.conventions]
merge_strategies = ["squash", "rebase"]
"#;
    let config = parse_config_str(content).unwrap();
    let team = config.team.unwrap();
    let conventions = team.conventions.unwrap();

    assert_eq!(
        conventions.merge_strategies,
        vec!["squash".to_string(), "rebase".to_string()]
    );
}

#[test]
fn test_parse_full_team_config() {
    let content = r#"
[team]
extends = "https://company.com/standards.toml"

[team.code_owners]
default_owners = ["@core-team"]

[team.code_owners.paths]
"*.rs" = ["@rust-team"]
"*.py" = ["@python-team"]

[team.review]
required_approvals = 2
dismiss_stale = true

[team.conventions]
commit_format = "conventional"
branch_pattern = "^(feature|fix)/.*$"
"#;
    let config = parse_config_str(content).unwrap();
    let team = config.team.unwrap();

    assert!(team.extends.is_some());
    assert!(team.code_owners.is_some());
    assert!(team.review.is_some());
    assert!(team.conventions.is_some());
}

#[test]
fn test_team_config_empty() {
    let content = r#"
[team]
"#;
    let config = parse_config_str(content).unwrap();
    let team = config.team.unwrap();

    assert!(team.extends.is_none());
    assert!(team.code_owners.is_none());
    assert!(team.review.is_none());
    assert!(team.conventions.is_none());
}

#[rstest]
#[case(1)]
#[case(2)]
#[case(3)]
#[case(5)]
fn test_parse_review_required_approvals(#[case] approvals: u32) {
    let content = format!(
        r#"
[team.review]
required_approvals = {}
"#,
        approvals
    );
    let config = parse_config_str(&content).unwrap();
    let team = config.team.unwrap();
    let review = team.review.unwrap();

    assert_eq!(review.required_approvals, Some(approvals));
}

#[test]
fn test_parse_code_owners_output_path() {
    let content = r#"
[team.code_owners]
enabled = true
output = ".github/CODEOWNERS"
"#;
    let config = parse_config_str(content).unwrap();
    let team = config.team.unwrap();
    let code_owners = team.code_owners.unwrap();

    assert_eq!(code_owners.enabled, Some(true));
    assert_eq!(code_owners.output, Some(".github/CODEOWNERS".to_string()));
}
