//! Tests for the nested `vx dev` warning (RFC: dev shell nesting).
//!
//! `vx dev` spawns one interactive shell per invocation, so an N-deep stack of
//! `vx dev` calls needs N `exit`s to unwind. The fix is deliberately
//! informational only: nesting stays allowed because re-opening a dev shell for
//! a different directory or `vx.toml` is a legitimate workflow. These tests pin
//! that contract — warn, never refuse, never silently reuse the session.

use rstest::*;
use std::collections::HashMap;
use vx_cli::commands::dev::{
    DEV_SHELL_DEPTH_VAR, DEV_SHELL_VAR, is_inside_dev_shell, nested_dev_shell_warning,
    next_dev_shell_depth,
};

/// Build an environment map from the given dev-shell variables.
fn env_with(entries: &[(&str, &str)]) -> HashMap<String, String> {
    entries
        .iter()
        .map(|(key, value)| (key.to_string(), value.to_string()))
        .collect()
}

// ============================================================================
// `is_inside_dev_shell`
// ============================================================================

#[rstest]
#[case(&[], false)]
#[case(&[(DEV_SHELL_VAR, "1")], true)]
#[case(&[(DEV_SHELL_VAR, "")], false)]
#[case(&[(DEV_SHELL_DEPTH_VAR, "3")], false)]
#[case(&[(DEV_SHELL_VAR, "1"), (DEV_SHELL_DEPTH_VAR, "3")], true)]
fn detects_whether_we_are_inside_a_dev_shell(
    #[case] entries: &[(&str, &str)],
    #[case] expected: bool,
) {
    assert_eq!(is_inside_dev_shell(&env_with(entries)), expected);
}

// ============================================================================
// `next_dev_shell_depth`
// ============================================================================

#[rstest]
// Not inside a dev shell at all -> the shell we spawn is the outermost one.
#[case(&[], 1)]
// Inside a shell that predates the depth counter -> treat it as depth 1.
#[case(&[(DEV_SHELL_VAR, "1")], 2)]
#[case(&[(DEV_SHELL_VAR, "1"), (DEV_SHELL_DEPTH_VAR, "1")], 2)]
#[case(&[(DEV_SHELL_VAR, "1"), (DEV_SHELL_DEPTH_VAR, "2")], 3)]
#[case(&[(DEV_SHELL_VAR, "1"), (DEV_SHELL_DEPTH_VAR, "7")], 8)]
// Depth counter without VX_DEV is meaningless -> outermost shell.
#[case(&[(DEV_SHELL_DEPTH_VAR, "4")], 1)]
fn computes_the_depth_of_the_shell_being_spawned(
    #[case] entries: &[(&str, &str)],
    #[case] expected: u32,
) {
    assert_eq!(next_dev_shell_depth(&env_with(entries)), expected);
}

#[rstest]
// Corrupt or nonsensical values must not panic or wrap to zero.
#[case("0", 2)]
#[case("-1", 2)]
#[case("not-a-number", 2)]
#[case("", 2)]
#[case("999999999999999999999", 2)]
#[case("4294967295", 4294967295)]
fn survives_corrupt_depth_values(#[case] depth: &str, #[case] expected: u32) {
    let env = env_with(&[(DEV_SHELL_VAR, "1"), (DEV_SHELL_DEPTH_VAR, depth)]);
    assert_eq!(next_dev_shell_depth(&env), expected);
}

/// A saturated depth stays at `u32::MAX` instead of wrapping to `0`, which
/// would report the shell as depth 1 and hide the nesting.
#[test]
fn depth_saturates_instead_of_wrapping() {
    let env = env_with(&[
        (DEV_SHELL_VAR, "1"),
        (DEV_SHELL_DEPTH_VAR, &u32::MAX.to_string()),
    ]);
    assert_eq!(next_dev_shell_depth(&env), u32::MAX);
}

// ============================================================================
// Warning text
// ============================================================================

#[rstest]
#[case(
    1,
    "vx dev: already inside a dev shell (depth 1); each nested shell needs its own 'exit'."
)]
#[case(
    2,
    "vx dev: already inside a dev shell (depth 2); each nested shell needs its own 'exit'."
)]
#[case(
    9,
    "vx dev: already inside a dev shell (depth 9); each nested shell needs its own 'exit'."
)]
fn warning_states_the_depth(#[case] depth: u32, #[case] expected: &str) {
    assert_eq!(nested_dev_shell_warning(depth), expected);
}

/// The warning tells the user how to get out — the whole point of the message.
#[test]
fn warning_mentions_exit() {
    let warning = nested_dev_shell_warning(3);
    assert!(
        warning.contains("exit"),
        "warning should explain how to leave the shell, got: {warning}"
    );
    assert!(
        warning.contains("depth 3"),
        "warning should carry the nesting depth, got: {warning}"
    );
}

// ============================================================================
// Contract: warn, but never refuse and never silently reuse the session
// ============================================================================

/// The nesting guard is a pure predicate over the environment: it does not
/// itself spawn, exit, or mutate anything. Nesting therefore remains allowed
/// (option "refuse" and option "reuse session" were both explicitly rejected),
/// and the caller decides to warn and then spawn exactly as before.
#[test]
fn nesting_helpers_have_no_side_effects() {
    let env = env_with(&[(DEV_SHELL_VAR, "1"), (DEV_SHELL_DEPTH_VAR, "2")]);

    let before = env.clone();
    let inside = is_inside_dev_shell(&env);
    let depth = next_dev_shell_depth(&env);

    assert!(inside, "a nested dev shell should be detected");
    assert_eq!(depth, 3);
    assert_eq!(env, before, "helpers must not mutate the environment");
}

/// Repeated nesting keeps counting up, so the depth the user sees always matches
/// the number of `exit`s needed to unwind the stack.
#[test]
fn depth_tracks_repeated_nesting() {
    let mut env: HashMap<String, String> = HashMap::new();

    for expected in 1..=5 {
        let depth = next_dev_shell_depth(&env);
        assert_eq!(
            depth, expected,
            "spawn #{expected} should be depth {expected}"
        );

        // Simulate entering the shell that was just spawned.
        env.insert(DEV_SHELL_VAR.to_string(), "1".to_string());
        env.insert(DEV_SHELL_DEPTH_VAR.to_string(), depth.to_string());
    }

    // After five nested shells the user needs five `exit`s.
    assert_eq!(next_dev_shell_depth(&env), 6);
}

/// No dev shell in the environment means depth 1 and no reason to warn — the
/// outermost `vx dev` must behave exactly as it did before this change.
#[test]
fn outermost_shell_is_silent() {
    let env: HashMap<String, String> = HashMap::new();

    assert!(
        !is_inside_dev_shell(&env),
        "a clean environment must not warn"
    );
    assert_eq!(next_dev_shell_depth(&env), 1);
}
