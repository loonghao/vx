mod common;

use std::io::ErrorKind;
use std::process::Command;
use std::time::{Duration, Instant};

#[test]
fn run_command_with_timeout_fails_fast() {
    let start = Instant::now();
    let result = common::run_command_with_timeout(sleep_command(), Duration::from_millis(200));

    let err = result.expect_err("sleep command should time out");
    assert_eq!(err.kind(), ErrorKind::TimedOut);
    assert!(
        start.elapsed() < Duration::from_secs(8),
        "timeout helper should fail fast"
    );
    assert!(
        err.to_string().contains("timed out"),
        "timeout error should explain the failure: {err}"
    );
}

#[cfg(windows)]
fn sleep_command() -> Command {
    let mut cmd = Command::new("powershell");
    cmd.args(["-NoProfile", "-Command", "Start-Sleep -Seconds 10"]);
    cmd
}

#[cfg(not(windows))]
fn sleep_command() -> Command {
    let mut cmd = Command::new("sh");
    cmd.args(["-c", "sleep 10"]);
    cmd
}
