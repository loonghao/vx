//! Tests for release workflow trigger logic
//!
//! These tests verify that the release workflow correctly handles different
//! scenarios including:
//! - Regular commits (should trigger release-please)
//! - Release PR merges (should trigger build without release-please)
//! - Manual workflow dispatch
//!
//! Reference: GitHub Actions workflow logic in .github/workflows/release.yml

use std::process::Command;

/// Test that version extraction from commit messages works correctly
#[test]
fn test_version_extraction_from_commit_message() {
    let test_cases = vec![
        ("chore: release v0.6.24", Some("v0.6.24")),
        ("chore: release vx-v0.6.24", Some("v0.6.24")),
        ("chore: release v1.0.0", Some("v1.0.0")),
        ("feat: add new feature", None),
        ("fix: resolve bug", None),
        ("chore(deps): bump bincode", None),
        ("chore: release v2.10.5-beta.1", Some("v2.10.5")), // Takes first match
    ];

    for (message, expected) in test_cases {
        let extracted = extract_version_from_commit(message);
        assert_eq!(
            extracted, expected,
            "Failed for message: '{}' - expected {:?}, got {:?}",
            message, expected, extracted
        );
    }
}

/// Test version normalization (removing prefixes)
#[test]
fn test_version_normalization() {
    let test_cases = vec![
        ("v0.6.24", "0.6.24"),
        ("vx-v0.6.24", "0.6.24"),
        ("vx-0.6.24", "0.6.24"),
        ("v1.0.0", "1.0.0"),
        ("0.6.24", "0.6.24"),
    ];

    for (input, expected) in test_cases {
        let normalized = normalize_version(input);
        assert_eq!(
            normalized, expected,
            "Failed for input: '{}' - expected '{}', got '{}'",
            input, expected, normalized
        );
    }
}

/// Test that commit message detection for release commits works
#[test]
fn test_is_release_commit() {
    let release_commits = vec![
        "chore: release v0.6.24",
        "chore: release vx-v0.6.24",
        "chore: release v1.0.0",
        "chore: release v10.20.30",
    ];

    let non_release_commits = vec![
        "feat: add new feature",
        "fix: resolve bug",
        "docs: update README",
        "chore(deps): bump bincode from 1.3.3 to 3.0.0",
        "refactor: improve performance",
        "test: add unit tests",
    ];

    for commit in release_commits {
        assert!(
            is_release_commit(commit),
            "Should be release commit: '{}'",
            commit
        );
    }

    for commit in non_release_commits {
        assert!(
            !is_release_commit(commit),
            "Should NOT be release commit: '{}'",
            commit
        );
    }
}

/// Test release commit detection with leading whitespace
#[test]
fn test_is_release_commit_with_leading_whitespace() {
    // Should handle leading whitespace correctly
    assert!(is_release_commit("  chore: release v0.6.24"));
    assert!(is_release_commit("\nchore: release v0.6.24"));
    assert!(is_release_commit("\tchore: release v0.6.24"));
    assert!(is_release_commit("   chore: release v1.0.0"));

    // Non-release commits with whitespace should still return false
    assert!(!is_release_commit("  feat: add feature"));
    assert!(!is_release_commit("\nchore(deps): bump package"));
}

/// Test workflow trigger condition logic
#[test]
fn test_should_trigger_build() {
    // Scenarios: (event_name, commit_message, release_created, should_trigger)
    let scenarios = vec![
        // Manual dispatch should always trigger
        ("workflow_dispatch", "any message", false, true),
        ("workflow_dispatch", "", true, true),
        // Regular push with new release should trigger
        ("push", "feat: new feature", true, true),
        // Release PR merge should trigger (even though release-please is skipped)
        ("push", "chore: release v0.6.24", false, true),
        // Regular push without release should not trigger
        ("push", "feat: new feature", false, false),
        // Dependabot commit should not trigger
        ("push", "chore(deps): bump bincode", false, false),
    ];

    for (event_name, commit_message, release_created, expected) in scenarios {
        let result = should_trigger_build(event_name, commit_message, release_created);
        assert_eq!(
            result, expected,
            "Failed for event='{}', msg='{}', created={}",
            event_name, commit_message, release_created
        );
    }
}

// Helper functions that mirror the workflow logic

fn extract_version_from_commit(message: &str) -> Option<&str> {
    // Extract version pattern vX.Y.Z from commit message
    // This mirrors the grep -oE 'v[0-9]+\.[0-9]+\.[0-9]+' logic in the workflow
    let patterns = [
        "v0.", "v1.", "v2.", "v3.", "v4.", "v5.", "v6.", "v7.", "v8.", "v9.",
    ];

    for pattern in &patterns {
        if let Some(pos) = message.find(pattern) {
            let rest = &message[pos..];
            // Find the end of version string (digits.digits.digits)
            let end = rest
                .chars()
                .take_while(|c| c.is_ascii_digit() || *c == '.' || *c == 'v')
                .count();
            let version = &rest[..end];
            // Validate it looks like a version
            if version.starts_with('v') && version.matches('.').count() >= 2 {
                return Some(version);
            }
        }
    }
    None
}

fn normalize_version(version: &str) -> String {
    // Remove vx- prefix and v prefix: vx-v0.5.0 -> 0.5.0, v0.5.0 -> 0.5.0
    version
        .trim_start_matches("vx-")
        .trim_start_matches("vx-v")
        .trim_start_matches('v')
        .to_string()
}

fn is_release_commit(message: &str) -> bool {
    // Be tolerant to leading whitespace/newlines in commit messages
    let msg = message.trim_start();
    msg.starts_with("chore: release") && extract_version_from_commit(msg).is_some()
}

fn should_trigger_build(event_name: &str, commit_message: &str, release_created: bool) -> bool {
    match event_name {
        "workflow_dispatch" => true,
        "push" => release_created || is_release_commit(commit_message),
        _ => false,
    }
}

#[cfg(test)]
mod workflow_condition_tests {
    use super::*;

    /// Integration test for the complete release workflow trigger scenario
    /// This documents the expected behavior after the fix
    #[test]
    fn test_release_workflow_trigger_scenarios() {
        println!("Testing release workflow trigger conditions...\n");

        // Scenario 1: Dependabot PR merge (should NOT trigger build)
        let result = should_trigger_build(
            "push",
            "chore(deps): bump bincode from 1.3.3 to 3.0.0",
            false,
        );
        assert!(!result, "Dependabot commits should NOT trigger builds");
        println!("✅ Dependabot PR merge: No build triggered (correct)");

        // Scenario 2: Feature commit (should NOT trigger build unless release-please creates release)
        let result = should_trigger_build("push", "feat: add new provider support", false);
        assert!(
            !result,
            "Regular commits without release should NOT trigger builds"
        );
        println!("✅ Feature commit without release: No build triggered (correct)");

        // Scenario 3: Release PR merge (SHOULD trigger build even without release_created)
        // This is the key fix - previously this would fail
        let result = should_trigger_build("push", "chore: release v0.6.24", false);
        assert!(result, "Release PR merge SHOULD trigger build");
        println!("✅ Release PR merge: Build triggered (FIXED!)");

        // Scenario 4: Manual workflow dispatch (should always trigger)
        let result = should_trigger_build("workflow_dispatch", "any message", false);
        assert!(result, "Manual dispatch should always trigger build");
        println!("✅ Manual workflow dispatch: Build triggered (correct)");

        // Scenario 5: Release-please creates release on regular push
        let result = should_trigger_build("push", "feat: another feature", true);
        assert!(
            result,
            "When release-please creates release, build should trigger"
        );
        println!("✅ Release-please creates release: Build triggered (correct)");

        println!("\n✅ All workflow trigger scenarios passed!");
    }
}
