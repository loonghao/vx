//! Common test utilities for vx-cli E2E tests

#![allow(dead_code)]

use std::io::{self, ErrorKind, Read};
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, Once, mpsc};
use std::thread;
use std::time::{Duration, Instant};

#[cfg(unix)]
use std::os::unix::process::CommandExt;

use vx_runtime::{ProviderRegistry, RuntimeContext, mock_context};

static INIT: Once = Once::new();
const DEFAULT_E2E_TIMEOUT_SECS: u64 = 300;

// ============================================================================
// Environment Setup
// ============================================================================

/// Initialize test environment (called once per test run)
pub fn init_test_env() {
    INIT.call_once(|| unsafe {
        std::env::set_var("VX_TEST_MODE", "1");
    });
}

/// Clean up test environment
pub fn cleanup_test_env() {
    // Clean up any test artifacts
}

// ============================================================================
// Registry Helpers
// ============================================================================

/// Create a test provider registry with all providers registered
pub fn create_test_registry() -> ProviderRegistry {
    vx_cli::create_registry()
}

/// Create a full registry with all available providers (async version for compatibility)
pub async fn create_full_registry() -> ProviderRegistry {
    vx_cli::create_registry()
}

/// Create a test runtime context
pub fn create_test_context() -> RuntimeContext {
    mock_context()
}

// ============================================================================
// Binary Helpers
// ============================================================================

/// Get the vx binary name for current platform
pub fn binary_name() -> &'static str {
    if cfg!(windows) { "vx.exe" } else { "vx" }
}

/// Get the vx binary path
pub fn vx_binary() -> PathBuf {
    // Check VX_BINARY environment variable first (for CI artifact-based testing)
    if let Ok(path) = std::env::var("VX_BINARY") {
        let p = PathBuf::from(&path);
        if p.exists() {
            return p;
        }
    }

    let cargo_target = std::env::var("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("target"));

    // Check release build first (CI uses release)
    let release_binary = cargo_target.join("release").join(binary_name());
    if release_binary.exists() {
        return release_binary;
    }

    // Check debug build
    let debug_binary = cargo_target.join("debug").join(binary_name());
    if debug_binary.exists() {
        return debug_binary;
    }

    // Fall back to system PATH
    PathBuf::from(binary_name())
}

/// Check if vx binary is available
pub fn vx_available() -> bool {
    vx_binary().exists() || Command::new("vx").arg("--version").output().is_ok()
}

// ============================================================================
// Command Execution Helpers
// ============================================================================

/// Run vx with given arguments
pub fn run_vx(args: &[&str]) -> std::io::Result<Output> {
    let mut cmd = Command::new(vx_binary());
    cmd.args(args);
    run_command_with_timeout(cmd, e2e_timeout())
}

/// Run vx in a specific directory
pub fn run_vx_in_dir(dir: &Path, args: &[&str]) -> std::io::Result<Output> {
    let mut cmd = Command::new(vx_binary());
    cmd.args(args).current_dir(dir);
    run_command_with_timeout(cmd, e2e_timeout())
}

/// Run vx with environment variables
pub fn run_vx_with_env(args: &[&str], env: &[(&str, &str)]) -> std::io::Result<Output> {
    let mut cmd = Command::new(vx_binary());
    cmd.args(args);
    for (key, value) in env {
        cmd.env(key, value);
    }
    run_command_with_timeout(cmd, e2e_timeout())
}

/// Run a command with a timeout so external tools cannot hang the whole E2E suite.
pub fn run_command_with_timeout(mut cmd: Command, timeout: Duration) -> io::Result<Output> {
    let command_debug = format!("{cmd:?}");
    cmd.stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    configure_timeout_child(&mut cmd);

    let mut child = cmd.spawn().map_err(|error| {
        io::Error::new(
            error.kind(),
            format!("failed to spawn {command_debug}: {error}"),
        )
    })?;
    let child_id = child.id();
    let start = Instant::now();
    let mut stdout = None;
    let mut stderr = None;

    let result = (|| {
        stdout = Some(OutputCapture::spawn(
            child
                .stdout
                .take()
                .ok_or_else(|| io::Error::other("missing child stdout pipe"))?,
            "stdout",
        )?);
        stderr = Some(OutputCapture::spawn(
            child
                .stderr
                .take()
                .ok_or_else(|| io::Error::other("missing child stderr pipe"))?,
            "stderr",
        )?);
        let stdout = stdout.as_mut().expect("stdout capture was initialized");
        let stderr = stderr.as_mut().expect("stderr capture was initialized");

        loop {
            let status = child.try_wait()?;
            let stdout_complete = stdout.poll()?;
            let stderr_complete = stderr.poll()?;
            if let Some(status) = status
                && stdout_complete
                && stderr_complete
            {
                return Ok(Output {
                    status,
                    stdout: stdout.take_bytes(),
                    stderr: stderr.take_bytes(),
                });
            }

            // EOF can lag direct-child exit when a descendant inherits a pipe.
            // Keep that wait under the same deadline as the command itself.
            if start.elapsed() >= timeout {
                return Err(io::Error::new(
                    ErrorKind::TimedOut,
                    format!(
                        "command timed out after {:.1}s: {command_debug}",
                        timeout.as_secs_f64()
                    ),
                ));
            }

            thread::sleep(Duration::from_millis(100));
        }
    })();

    match result {
        Ok(output) => Ok(output),
        Err(error) => {
            // Cancel before cleanup/snapshotting so escaped writers cannot keep
            // growing a detached capture buffer after this function returns.
            for capture in [&stdout, &stderr].into_iter().flatten() {
                capture.cancel();
            }
            terminate_process_tree(child_id, &mut child);
            let mut message = format!("{error}\ncommand: {command_debug}");
            if let Err(error) = wait_after_timeout(&mut child, Duration::from_secs(2)) {
                message.push_str(&format!("\nfailed to reap child after failure: {error}"));
            }
            for capture in [&stdout, &stderr].into_iter().flatten() {
                message.push_str(&capture.diagnostic());
            }
            Err(io::Error::new(error.kind(), message))
        }
    }
}

/// Drain one pipe without tying the caller's deadline to a blocking reader join.
struct OutputCapture {
    name: &'static str,
    bytes: Arc<Mutex<Vec<u8>>>,
    cancelled: Arc<AtomicBool>,
    completion: mpsc::Receiver<io::Result<()>>,
    complete: bool,
}

impl OutputCapture {
    fn spawn(reader: impl Read + Send + 'static, name: &'static str) -> io::Result<Self> {
        let bytes = Arc::new(Mutex::new(Vec::new()));
        let cancelled = Arc::new(AtomicBool::new(false));
        let (sender, completion) = mpsc::sync_channel(1);
        let worker_bytes = Arc::clone(&bytes);
        let worker_cancelled = Arc::clone(&cancelled);
        // Portable std cannot interrupt an already-blocked pipe read. Never
        // join this worker: process-tree cleanup normally closes the pipe; an
        // escaped writer may keep the thread until its next write/EOF. The
        // cancellation flag prevents further buffering while the caller exits.
        thread::Builder::new()
            .name(format!("vx-e2e-{name}"))
            .spawn(move || {
                let result = Self::read(reader, &worker_bytes, &worker_cancelled);
                let _ = sender.send(result);
            })?;
        Ok(Self {
            name,
            bytes,
            cancelled,
            completion,
            complete: false,
        })
    }

    fn read(
        mut reader: impl Read,
        bytes: &Mutex<Vec<u8>>,
        cancelled: &AtomicBool,
    ) -> io::Result<()> {
        let mut chunk = [0; 8192];
        while !cancelled.load(Ordering::Acquire) {
            match reader.read(&mut chunk) {
                Ok(0) => break,
                Ok(count) => {
                    let mut bytes = bytes.lock().unwrap_or_else(|error| error.into_inner());
                    if !cancelled.load(Ordering::Acquire) {
                        bytes.extend_from_slice(&chunk[..count]);
                    }
                }
                Err(error) if error.kind() == ErrorKind::Interrupted => continue,
                Err(error) => return Err(error),
            }
        }
        Ok(())
    }

    fn poll(&mut self) -> io::Result<bool> {
        if !self.complete {
            match self.completion.try_recv() {
                Ok(result) => {
                    self.complete = true;
                    result.map_err(|error| {
                        io::Error::new(error.kind(), format!("{} capture: {error}", self.name))
                    })?;
                }
                Err(mpsc::TryRecvError::Empty) => {}
                Err(mpsc::TryRecvError::Disconnected) => {
                    return Err(io::Error::other(format!(
                        "{} capture reader ended without a result",
                        self.name
                    )));
                }
            }
        }
        Ok(self.complete)
    }

    fn take_bytes(&self) -> Vec<u8> {
        std::mem::take(&mut *self.bytes.lock().unwrap_or_else(|error| error.into_inner()))
    }

    fn cancel(&self) {
        self.cancelled.store(true, Ordering::Release);
    }

    fn diagnostic(&self) -> String {
        const MAX_DIAGNOSTIC_BYTES: usize = 4096;
        let bytes = self.bytes.lock().unwrap_or_else(|error| error.into_inner());
        let count = bytes.len().min(MAX_DIAGNOSTIC_BYTES);
        let truncated = if count < bytes.len() {
            ", truncated"
        } else {
            ""
        };
        format!(
            "\n{} ({} bytes{truncated}):\n{}",
            self.name,
            bytes.len(),
            String::from_utf8_lossy(&bytes[..count])
        )
    }
}

impl Drop for OutputCapture {
    fn drop(&mut self) {
        self.cancel();
    }
}

fn e2e_timeout() -> Duration {
    std::env::var("VX_E2E_TIMEOUT_SECS")
        .ok()
        .and_then(|raw| raw.parse::<u64>().ok())
        .filter(|seconds| *seconds > 0)
        .map(Duration::from_secs)
        .unwrap_or_else(|| Duration::from_secs(DEFAULT_E2E_TIMEOUT_SECS))
}

fn terminate_process_tree(_pid: u32, child: &mut std::process::Child) {
    #[cfg(windows)]
    {
        run_cleanup_command(Command::new("taskkill").args(["/PID", &_pid.to_string(), "/T", "/F"]));
        let _ = child.kill();
    }

    #[cfg(not(windows))]
    {
        let process_group = format!("-{_pid}");
        run_cleanup_command(Command::new("kill").args(["-TERM", "--", &process_group]));
        thread::sleep(Duration::from_millis(100));
        run_cleanup_command(Command::new("kill").args(["-KILL", "--", &process_group]));
        let _ = child.kill();
    }
}

fn run_cleanup_command(command: &mut Command) {
    let Ok(mut child) = command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
    else {
        return;
    };
    if wait_after_timeout(&mut child, Duration::from_secs(2)).is_err() {
        let _ = child.kill();
        let _ = wait_after_timeout(&mut child, Duration::from_millis(500));
    }
}

fn wait_after_timeout(child: &mut std::process::Child, timeout: Duration) -> io::Result<()> {
    let start = Instant::now();
    loop {
        if child.try_wait()?.is_some() {
            return Ok(());
        }

        if start.elapsed() >= timeout {
            return Err(io::Error::new(
                ErrorKind::TimedOut,
                "child did not exit after timeout termination",
            ));
        }

        thread::sleep(Duration::from_millis(50));
    }
}

#[cfg(unix)]
fn configure_timeout_child(cmd: &mut Command) {
    cmd.process_group(0);
}

#[cfg(not(unix))]
fn configure_timeout_child(_cmd: &mut Command) {}

// ============================================================================
// Output Helpers
// ============================================================================

/// Check if output indicates success
pub fn is_success(output: &Output) -> bool {
    output.status.success()
}

/// Get stdout as string
pub fn stdout_str(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).to_string()
}

/// Get stderr as string
pub fn stderr_str(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).to_string()
}

/// Combined output for debugging
pub fn combined_output(output: &Output) -> String {
    format!(
        "stdout:\n{}\nstderr:\n{}",
        stdout_str(output),
        stderr_str(output)
    )
}

/// Get exit code
pub fn exit_code(output: &Output) -> Option<i32> {
    output.status.code()
}

// ============================================================================
// Assertion Helpers
// ============================================================================

/// Assert command succeeded
pub fn assert_success(output: &Output, context: &str) {
    assert!(
        is_success(output),
        "{} should succeed: {}",
        context,
        combined_output(output)
    );
}

/// Assert command failed
pub fn assert_failure(output: &Output, context: &str) {
    assert!(
        !is_success(output),
        "{} should fail: {}",
        context,
        combined_output(output)
    );
}

/// Assert stdout contains text
pub fn assert_stdout_contains(output: &Output, text: &str, context: &str) {
    let stdout = stdout_str(output);
    assert!(
        stdout.contains(text),
        "{}: stdout should contain '{}'\nActual: {}",
        context,
        text,
        combined_output(output)
    );
}

/// Assert stderr contains text
pub fn assert_stderr_contains(output: &Output, text: &str, context: &str) {
    let stderr = stderr_str(output);
    assert!(
        stderr.contains(text),
        "{}: stderr should contain '{}'\nActual: {}",
        context,
        text,
        combined_output(output)
    );
}

/// Assert output (stdout or stderr) contains text
pub fn assert_output_contains(output: &Output, text: &str, context: &str) {
    let combined = combined_output(output);
    assert!(
        combined.contains(text),
        "{}: output should contain '{}'\nActual: {}",
        context,
        text,
        combined
    );
}

// ============================================================================
// Skip Helpers
// ============================================================================

/// Skip test if vx is not available
#[macro_export]
macro_rules! skip_if_no_vx {
    () => {
        if !$crate::common::vx_available() {
            eprintln!("Skipping: vx binary not found");
            return;
        }
    };
}

/// Skip test if network tests are disabled
/// Network tests run automatically in CI (when GITHUB_TOKEN is set) or when VX_NETWORK_TESTS=1
#[macro_export]
macro_rules! skip_if_no_network {
    () => {
        if !$crate::common::network_tests_enabled() {
            eprintln!("Skipping: network tests disabled (set VX_NETWORK_TESTS=1 or GITHUB_TOKEN to enable)");
            return;
        }
    };
}

/// Check if network tests should run
/// Returns true if:
/// - VX_NETWORK_TESTS=1 is set explicitly, OR
/// - GITHUB_TOKEN or GH_TOKEN is set (CI environment), OR
/// - CI=true is set (GitHub Actions, etc.)
pub fn network_tests_enabled() -> bool {
    // Explicit opt-in
    if std::env::var("VX_NETWORK_TESTS")
        .map(|v| v == "1")
        .unwrap_or(false)
    {
        return true;
    }

    // CI environment with GitHub token (avoids rate limits)
    let has_token = std::env::var("GITHUB_TOKEN").is_ok() || std::env::var("GH_TOKEN").is_ok();
    let is_ci = std::env::var("CI").map(|v| v == "true").unwrap_or(false);

    has_token || is_ci
}

/// Skip test if tool is not installed (check via vx which)
pub fn tool_installed(tool: &str) -> bool {
    if !vx_available() {
        return false;
    }
    run_vx(&["which", tool])
        .map(|o| is_success(&o))
        .unwrap_or(false)
}

// ============================================================================
// Constants
// ============================================================================

/// Supported tools for testing (tools registered via Runtime, not package managers)
pub const SUPPORTED_TOOLS: &[&str] = &["node", "go", "cargo", "uv", "bun"];

/// Supported package managers for testing
pub const SUPPORTED_PACKAGE_MANAGERS: &[&str] = &["npm", "pnpm", "yarn"];

/// Get all registered runtime names from the registry
pub fn get_registered_runtimes(registry: &ProviderRegistry) -> Vec<String> {
    registry.runtime_names()
}
