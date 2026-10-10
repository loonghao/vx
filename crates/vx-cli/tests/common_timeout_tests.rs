mod common;

use std::io::{ErrorKind, Write};
use std::process::Command;
use std::time::{Duration, Instant};

use rstest::rstest;

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

#[rstest]
fn test_run_command_with_timeout_captures_large_output() {
    let mut command = Command::new(std::env::current_exe().unwrap());
    command.args([
        "--ignored",
        "--exact",
        "test_child_produces_large_output",
        "--nocapture",
    ]);
    let output = common::run_command_with_timeout(command, Duration::from_secs(10)).unwrap();
    assert!(output.status.success());
    assert_eq!(
        output.stdout.iter().filter(|&&byte| byte == b'X').count(),
        1_048_576
    );
    assert_eq!(
        output.stderr.iter().filter(|&&byte| byte == b'Y').count(),
        1_048_576
    );
}

#[rstest]
#[ignore = "executed by the large-output timeout regression"]
fn test_child_produces_large_output() {
    let mut stdout = std::io::stdout().lock();
    let mut stderr = std::io::stderr().lock();
    for _ in 0..64 {
        stdout.write_all(&[b'X'; 16_384]).unwrap();
        stderr.write_all(&[b'Y'; 16_384]).unwrap();
    }
    stdout.flush().unwrap();
    stderr.flush().unwrap();
    if std::env::var_os("VX_TIMEOUT_PROBE_SLEEP").is_some() {
        std::thread::sleep(Duration::from_secs(30));
    }
}

#[rstest]
fn test_vx_binary_is_absolute_from_foreign_directory() {
    let directory = tempfile::tempdir().unwrap();
    let mut command = Command::new(std::env::current_exe().unwrap());
    command
        .args([
            "--ignored",
            "--exact",
            "test_child_locates_profile_candidate",
            "--nocapture",
        ])
        .env_remove("VX_BINARY")
        .env("CARGO_TARGET_DIR", "target")
        .current_dir(directory.path());
    let output = common::run_command_with_timeout(command, Duration::from_secs(15)).unwrap();
    assert!(output.status.success(), "{output:?}");
}

#[rstest]
#[ignore = "executed by the foreign-directory binary regression"]
fn test_child_locates_profile_candidate() {
    let binary = common::vx_binary();
    assert!(binary.is_absolute());
    assert_eq!(binary.file_name().unwrap(), common::binary_name());
}

#[rstest]
fn test_run_command_with_timeout_preserves_bounded_partial_output() {
    let mut command = Command::new(std::env::current_exe().unwrap());
    command
        .args([
            "--ignored",
            "--exact",
            "test_child_produces_large_output",
            "--nocapture",
        ])
        .env("VX_TIMEOUT_PROBE_SLEEP", "1");
    let error = common::run_command_with_timeout(command, Duration::from_secs(2)).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::TimedOut);
    let message = error.to_string();
    assert!(message.contains("stdout (first 16 KiB):"));
    assert!(message.contains("stderr (first 16 KiB):"));
    assert!(message.contains(&"Y".repeat(1024)));
    assert!(message.len() < 40 * 1024);
}
