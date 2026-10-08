//! Native subprocess coverage for RuntimeTester output capture and deadlines.

#![cfg(feature = "testing")]

use std::io::{self, Write};
use std::time::{Duration, Instant};

use rstest::rstest;

use vx_runtime::{RuntimeTester, TestCommand, TestConfig};

const PAYLOAD_SIZE: usize = 1024 * 1024;
const STDOUT_MARKER: &str = "runtime stdout marker";
const STDERR_MARKER: &str = "runtime stderr marker";

fn fixture_selected(name: &str) -> bool {
    let args: Vec<_> = std::env::args().collect();
    args.iter().any(|arg| arg == "--exact") && args.iter().any(|arg| arg == name)
}

fn helper_command(name: &str) -> TestCommand {
    TestCommand::new(format!(
        "{{executable}} --exact {name} --ignored --nocapture"
    ))
}

fn run_command(command: TestCommand) -> vx_runtime::RuntimeTestResult {
    RuntimeTester::new("runtime-output-fixture")
        .with_executable(std::env::current_exe().unwrap())
        .with_config(TestConfig {
            functional_commands: vec![command],
            timeout_ms: 5_000,
            ..Default::default()
        })
        .run_all()
}

fn write_large_output(stderr_first: bool) {
    let stdout = || {
        let mut pipe = io::stdout().lock();
        writeln!(pipe, "{STDOUT_MARKER}").unwrap();
        pipe.write_all(&vec![b'o'; PAYLOAD_SIZE]).unwrap();
        pipe.flush().unwrap();
    };
    let stderr = || {
        let mut pipe = io::stderr().lock();
        writeln!(pipe, "{STDERR_MARKER}").unwrap();
        pipe.write_all(&vec![b'e'; PAYLOAD_SIZE]).unwrap();
        pipe.flush().unwrap();
    };
    if stderr_first {
        stderr();
        stdout();
    } else {
        stdout();
        stderr();
    }
    std::process::exit(0);
}

#[test]
#[ignore = "native fixture launched by RuntimeTester output tests"]
fn test_helper_stdout_first() {
    if fixture_selected("test_helper_stdout_first") {
        write_large_output(false);
    }
}

#[test]
#[ignore = "native fixture launched by RuntimeTester output tests"]
fn test_helper_stderr_first() {
    if fixture_selected("test_helper_stderr_first") {
        write_large_output(true);
    }
}

#[test]
#[ignore = "native fixture launched by RuntimeTester output tests"]
fn test_helper_nonzero_exit() {
    if fixture_selected("test_helper_nonzero_exit") {
        writeln!(io::stdout(), "{STDOUT_MARKER}").unwrap();
        writeln!(io::stderr(), "{STDERR_MARKER}").unwrap();
        io::stdout().flush().unwrap();
        io::stderr().flush().unwrap();
        std::process::exit(7);
    }
}

#[test]
#[ignore = "native fixture launched by RuntimeTester output tests"]
fn test_helper_slow_exit() {
    if fixture_selected("test_helper_slow_exit") {
        std::thread::sleep(Duration::from_secs(10));
    }
}

#[rstest]
#[case("test_helper_stdout_first")]
#[case("test_helper_stderr_first")]
fn test_runtime_tester_drains_both_large_output_streams(#[case] helper: &str) {
    let result = run_command(helper_command(helper));
    assert_eq!(result.test_cases.len(), 1);
    let case = &result.test_cases[0];
    assert!(case.passed, "{:?}", case.error);
    assert_eq!(case.exit_code, Some(0));
    let stdout = case.stdout.as_deref().unwrap();
    let stderr = case.stderr.as_deref().unwrap();
    assert!(stdout.contains(STDOUT_MARKER));
    assert!(stderr.contains(STDERR_MARKER));
    assert!(stdout.contains(&"o".repeat(PAYLOAD_SIZE)));
    assert!(stderr.contains(&"e".repeat(PAYLOAD_SIZE)));
    assert!(!stdout.contains(STDERR_MARKER));
    assert!(!stderr.contains(STDOUT_MARKER));
}

#[rstest]
#[case(Some(7), true)]
#[case(None, false)]
fn test_runtime_tester_preserves_nonzero_exit_and_output(
    #[case] expected_exit_code: Option<i32>,
    #[case] expected_passed: bool,
) {
    let mut command = helper_command("test_helper_nonzero_exit");
    command.expected_exit_code = expected_exit_code;
    let result = run_command(command);
    assert_eq!(result.test_cases.len(), 1);
    let case = &result.test_cases[0];
    assert_eq!(case.passed, expected_passed, "{:?}", case.error);
    assert_eq!(case.exit_code, Some(7));
    assert!(case.stdout.as_deref().unwrap().contains(STDOUT_MARKER));
    assert!(case.stderr.as_deref().unwrap().contains(STDERR_MARKER));
    if !expected_passed {
        assert!(case.error.as_deref().unwrap().contains("exit code 7"));
    }
}

#[test]
fn test_runtime_tester_preserves_per_command_timeout() {
    let mut command = helper_command("test_helper_slow_exit");
    command.timeout_ms = Some(250);
    let start = Instant::now();
    let result = run_command(command);
    assert!(start.elapsed() < Duration::from_secs(4));
    assert_eq!(result.test_cases.len(), 1);
    let case = &result.test_cases[0];
    assert!(!case.passed);
    assert!(case.error.as_deref().unwrap().contains("timed out"));
}

#[cfg(windows)]
#[test]
#[ignore = "native fixture launched by RuntimeTester output tests"]
fn test_helper_windows_pipe_flush() {
    if !fixture_selected("test_helper_windows_pipe_flush") {
        return;
    }
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetStdHandle(kind: u32) -> *mut std::ffi::c_void;
        fn FlushFileBuffers(handle: *mut std::ffi::c_void) -> i32;
    }

    // Godot's Windows terminal logger flushes the pipe before exiting, even
    // when --version writes only a few bytes. This waits for a concurrent reader.
    writeln!(io::stdout(), "4.7.2.stable.fixture").unwrap();
    io::stdout().flush().unwrap();
    // SAFETY: GetStdHandle receives STD_OUTPUT_HANDLE and the child has a piped
    // stdout. The borrowed OS handle remains owned by the process.
    let flushed = unsafe { FlushFileBuffers(GetStdHandle(-11_i32 as u32)) };
    assert_ne!(flushed, 0, "{}", io::Error::last_os_error());
    writeln!(io::stdout(), "pipe flush completed").unwrap();
    io::stdout().flush().unwrap();
    std::process::exit(0);
}

#[cfg(windows)]
#[test]
fn test_runtime_tester_reads_while_windows_child_flushes_pipe() {
    let result = run_command(helper_command("test_helper_windows_pipe_flush"));
    assert_eq!(result.test_cases.len(), 1);
    let case = &result.test_cases[0];
    assert!(case.passed, "{:?}", case.error);
    assert_eq!(case.exit_code, Some(0));
    let stdout = case.stdout.as_deref().unwrap();
    assert!(stdout.contains("4.7.2.stable.fixture"));
    assert!(stdout.contains("pipe flush completed"));
}
