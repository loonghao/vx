//! Offline regressions for the E2E subprocess timeout helper.

mod common;

use rstest::rstest;
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

#[cfg(unix)]
use std::os::unix::process::CommandExt;

const PAYLOAD_SIZE: usize = 1024 * 1024;
const CHILD_MODE: &str = "VX_COMMAND_TIMEOUT_TEST_CHILD";

fn write_payload(stream: &mut impl Write, label: &str, byte: u8, size: usize) {
    write!(stream, "<{label}-begin>").expect("write payload marker");
    stream.write_all(&vec![byte; size]).expect("write payload");
    write!(stream, "<{label}-end>").expect("write payload marker");
    stream.flush().expect("flush payload");
}

fn child_command(mode: &str) -> Command {
    let mut command = Command::new(std::env::current_exe().expect("test executable"));
    command
        .args([
            "--exact",
            thread::current().name().expect("current test name"),
            "--nocapture",
        ])
        .env(CHILD_MODE, mode);
    command
}

fn run_child_if_requested() {
    let Ok(mode) = std::env::var(CHILD_MODE) else {
        return;
    };
    match mode.as_str() {
        "stdout" | "stderr" | "both" => {
            if mode != "stderr" {
                write_payload(&mut std::io::stdout(), "stdout", b'a', PAYLOAD_SIZE);
            }
            if mode != "stdout" {
                write_payload(&mut std::io::stderr(), "stderr", b'b', PAYLOAD_SIZE);
            }
        }
        "small" | "nonzero" => {
            write_payload(&mut std::io::stdout(), "stdout", b'a', 128);
            write_payload(&mut std::io::stderr(), "stderr", b'b', 128);
            if mode == "nonzero" {
                std::process::exit(7);
            }
        }
        "timeout" => {
            write_payload(&mut std::io::stdout(), "stdout", b'a', 16 * 1024);
            write_payload(&mut std::io::stderr(), "stderr", b'b', 16 * 1024);
            thread::sleep(Duration::from_secs(30));
        }
        "parent" => {
            let mut command = child_command("descendant");
            command.stdout(Stdio::inherit()).stderr(Stdio::inherit());
            // Keep this writer outside the direct child's cleanup group on Unix.
            #[cfg(unix)]
            command.process_group(0);
            #[expect(
                clippy::zombie_processes,
                reason = "The fixture parent must exit first; the outer test signals and observes writer cleanup"
            )]
            let descendant = command.spawn().expect("spawn inherited-pipe writer");
            std::fs::write(
                std::env::var_os("VX_TIMEOUT_DESCENDANT_PID").expect("PID file"),
                descendant.id().to_string(),
            )
            .expect("record descendant PID");
        }
        "descendant" => {
            let stop = PathBuf::from(std::env::var_os("VX_TIMEOUT_STOP").expect("stop file"));
            let done = PathBuf::from(std::env::var_os("VX_TIMEOUT_DONE").expect("done file"));
            write_payload(&mut std::io::stdout(), "stdout", b'a', 128);
            write_payload(&mut std::io::stderr(), "stderr", b'b', 128);
            let start = Instant::now();
            while !stop.exists() && start.elapsed() < Duration::from_secs(8) {
                thread::sleep(Duration::from_millis(20));
            }
            std::fs::write(done, b"writer closed").expect("record writer cleanup");
        }
        _ => panic!("unknown child mode: {mode}"),
    }
    std::process::exit(0);
}

fn assert_payload(bytes: &[u8], label: &str, byte: u8, size: usize) {
    let prefix = format!("<{label}-begin>");
    let suffix = format!("<{label}-end>");
    let start = bytes
        .windows(prefix.len())
        .position(|part| part == prefix.as_bytes())
        .expect("opening payload marker")
        + prefix.len();
    assert_eq!(&bytes[start..start + size], vec![byte; size]);
    assert_eq!(
        &bytes[start + size..start + size + suffix.len()],
        suffix.as_bytes()
    );
}

#[rstest]
#[case::stdout("stdout")]
#[case::stderr("stderr")]
#[case::both("both")]
fn preserves_large_output(#[case] mode: &str) {
    run_child_if_requested();
    let output = common::run_command_with_timeout(child_command(mode), Duration::from_secs(3))
        .expect("large output must drain before the deadline");
    assert!(output.status.success());
    if mode != "stderr" {
        assert_payload(&output.stdout, "stdout", b'a', PAYLOAD_SIZE);
    }
    if mode != "stdout" {
        assert_payload(&output.stderr, "stderr", b'b', PAYLOAD_SIZE);
    }
}

#[rstest]
#[case::success("small", 0)]
#[case::nonzero("nonzero", 7)]
fn preserves_output_and_exit_status(#[case] mode: &str, #[case] exit: i32) {
    run_child_if_requested();
    let output = common::run_command_with_timeout(child_command(mode), Duration::from_secs(3))
        .expect("normal child exit");
    assert_eq!(output.status.code(), Some(exit));
    assert_payload(&output.stdout, "stdout", b'a', 128);
    assert_payload(&output.stderr, "stderr", b'b', 128);
}

#[test]
fn timeout_preserves_bounded_partial_diagnostics() {
    run_child_if_requested();
    let start = Instant::now();
    let error = common::run_command_with_timeout(child_command("timeout"), Duration::from_secs(3))
        .expect_err("sleeping child must time out");
    assert_eq!(error.kind(), ErrorKind::TimedOut);
    let message = error.to_string();
    assert!(message.contains("<stdout-begin>"), "{message}");
    assert!(message.contains("<stderr-begin>"), "{message}");
    assert!(message.contains("truncated"), "{message}");
    assert!(message.len() < 12 * 1024, "diagnostics must stay bounded");
    assert!(start.elapsed() < Duration::from_secs(12));
}

struct DescendantCleanup {
    stop: PathBuf,
    done: PathBuf,
}

impl DescendantCleanup {
    fn stop_and_wait(&self) -> bool {
        let _ = std::fs::write(&self.stop, b"stop");
        let start = Instant::now();
        while !self.done.exists() && start.elapsed() < Duration::from_secs(3) {
            thread::sleep(Duration::from_millis(20));
        }
        self.done.exists()
    }
}

impl Drop for DescendantCleanup {
    fn drop(&mut self) {
        self.stop_and_wait();
    }
}

#[test]
fn descendant_held_pipes_cannot_bypass_deadline() {
    run_child_if_requested();
    let directory = tempfile::tempdir().expect("fixture directory");
    let cleanup = DescendantCleanup {
        stop: directory.path().join("stop"),
        done: directory.path().join("done"),
    };
    let pid = directory.path().join("descendant.pid");
    let mut command = child_command("parent");
    command
        .env("VX_TIMEOUT_STOP", &cleanup.stop)
        .env("VX_TIMEOUT_DONE", &cleanup.done)
        .env("VX_TIMEOUT_DESCENDANT_PID", &pid);
    let start = Instant::now();
    let result = common::run_command_with_timeout(command, Duration::from_secs(3));
    // Always release the escaped writer before assertions, including on regression.
    let cleaned = cleanup.stop_and_wait();
    assert!(Path::new(&pid).exists(), "descendant was actually spawned");
    assert!(cleaned, "escaped writer acknowledged cleanup");
    assert_eq!(
        result
            .expect_err("inherited pipes must obey the deadline")
            .kind(),
        ErrorKind::TimedOut
    );
    assert!(start.elapsed() < Duration::from_secs(12));
}

#[test]
fn spawn_error_keeps_its_error_kind() {
    let directory = tempfile::tempdir().expect("fixture directory");
    let command = Command::new(directory.path().join("missing-command-timeout-fixture"));
    let error = common::run_command_with_timeout(command, Duration::from_secs(1))
        .expect_err("missing executable");
    assert_eq!(error.kind(), ErrorKind::NotFound);
}
