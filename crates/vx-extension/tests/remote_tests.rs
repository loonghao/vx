//! Tests for remote extension installation

use rstest::rstest;
use vx_extension::RemoteSource;

#[rstest]
#[case("github:user/repo", "user", "repo", None)]
#[case("github:user/repo@v1.0.0", "user", "repo", Some("v1.0.0"))]
#[case("github:org/my-extension@main", "org", "my-extension", Some("main"))]
fn test_parse_github_shorthand(
    #[case] input: &str,
    #[case] expected_owner: &str,
    #[case] expected_repo: &str,
    #[case] expected_version: Option<&str>,
) {
    let source = RemoteSource::parse(input).unwrap();
    match source {
        RemoteSource::GitHub {
            owner,
            repo,
            version,
            ..
        } => {
            assert_eq!(owner, expected_owner);
            assert_eq!(repo, expected_repo);
            assert_eq!(version.as_deref(), expected_version);
        }
        _ => panic!("Expected GitHub source"),
    }
}

#[rstest]
#[case("https://github.com/user/repo", "user", "repo", None)]
#[case("https://github.com/user/repo@v2.0.0", "user", "repo", Some("v2.0.0"))]
#[case("https://github.com/org/project.git", "org", "project", None)]
fn test_parse_github_https_url(
    #[case] input: &str,
    #[case] expected_owner: &str,
    #[case] expected_repo: &str,
    #[case] expected_version: Option<&str>,
) {
    let source = RemoteSource::parse(input).unwrap();
    match source {
        RemoteSource::GitHub {
            owner,
            repo,
            version,
            ..
        } => {
            assert_eq!(owner, expected_owner);
            assert_eq!(repo, expected_repo);
            assert_eq!(version.as_deref(), expected_version);
        }
        _ => panic!("Expected GitHub source"),
    }
}

#[rstest]
#[case("git@github.com:user/repo.git", "user", "repo", None)]
#[case("git@github.com:org/project.git@v1.0", "org", "project", Some("v1.0"))]
fn test_parse_github_ssh_url(
    #[case] input: &str,
    #[case] expected_owner: &str,
    #[case] expected_repo: &str,
    #[case] expected_version: Option<&str>,
) {
    let source = RemoteSource::parse(input).unwrap();
    match source {
        RemoteSource::GitHub {
            owner,
            repo,
            version,
            ..
        } => {
            assert_eq!(owner, expected_owner);
            assert_eq!(repo, expected_repo);
            assert_eq!(version.as_deref(), expected_version);
        }
        _ => panic!("Expected GitHub source"),
    }
}

#[rstest]
#[case("invalid-source")]
#[case("ftp://example.com/repo")]
#[case("file:///path/to/repo")]
fn test_parse_invalid_source(#[case] input: &str) {
    let result = RemoteSource::parse(input);
    assert!(result.is_err());
}

#[test]
fn test_clone_url_github() {
    let source = RemoteSource::GitHub {
        owner: "user".to_string(),
        repo: "repo".to_string(),
        version: None,
        subdir: None,
    };
    assert_eq!(source.clone_url(), "https://github.com/user/repo.git");
}

#[test]
fn test_clone_url_git() {
    let source = RemoteSource::GitUrl {
        url: "https://gitlab.com/user/repo.git".to_string(),
        version: None,
    };
    assert_eq!(source.clone_url(), "https://gitlab.com/user/repo.git");
}

#[test]
fn test_version() {
    let source = RemoteSource::GitHub {
        owner: "user".to_string(),
        repo: "repo".to_string(),
        version: Some("v1.0.0".to_string()),
        subdir: None,
    };
    assert_eq!(source.version(), Some("v1.0.0"));

    let source_no_version = RemoteSource::GitHub {
        owner: "user".to_string(),
        repo: "repo".to_string(),
        version: None,
        subdir: None,
    };
    assert_eq!(source_no_version.version(), None);
}

#[test]
fn test_display_name() {
    let source = RemoteSource::GitHub {
        owner: "user".to_string(),
        repo: "repo".to_string(),
        version: None,
        subdir: None,
    };
    assert_eq!(source.display_name(), "user/repo");

    let source_git = RemoteSource::GitUrl {
        url: "https://gitlab.com/user/repo.git".to_string(),
        version: None,
    };
    assert_eq!(
        source_git.display_name(),
        "https://gitlab.com/user/repo.git"
    );
}
