//! Unix-specific process execution with signal handling

use anyhow::{Context, Result};
use nix::sys::signal::{self, Signal};
use nix::sys::wait::{waitpid, WaitStatus};
use nix::unistd::{fork, ForkResult, Pid};
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::config::ShimConfig;
use crate::platform::PlatformExecutorTrait;

/// Unix-specific process executor
pub struct UnixExecutor {
    signal_setup: Arc<AtomicBool>,
}

impl UnixExecutor {
    /// Create a new Unix executor
    pub fn new() -> Self {
        Self {
            signal_setup: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Set up signal handling
    fn setup_signal_handling(&self, config: &ShimConfig) -> Result<()> {
        if self.signal_setup.load(Ordering::Relaxed) {
            return Ok(());
        }

        let signal_config = config.signal_handling.as_ref();
        let ignore_sigint = signal_config.and_then(|s| s.ignore_sigint).unwrap_or(true);

        if ignore_sigint {
            // Ignore SIGINT so it gets passed to the child process
            unsafe {
                signal::signal(Signal::SIGINT, signal::SigHandler::SigIgn)
                    .context("Failed to set SIGINT handler")?;
            }
        }

        // Set up SIGTERM handler to clean up child processes
        unsafe {
            signal::signal(Signal::SIGTERM, signal::SigHandler::SigIgn)
                .context("Failed to set SIGTERM handler")?;
        }

        self.signal_setup.store(true, Ordering::Relaxed);
        Ok(())
    }

    /// Execute using fork/exec for better signal handling
    fn execute_with_fork(&self, mut command: Command, config: &ShimConfig) -> Result<i32> {
        self.setup_signal_handling(config)?;

        match unsafe { fork() }.context("Failed to fork process")? {
            ForkResult::Parent { child } => {
                // Parent process - wait for child
                self.wait_for_child(child, config)
            }
            ForkResult::Child => {
                // Child process - exec the target program
                let error = CommandExt::exec(&mut command);
                // If we reach here, exec failed
                eprintln!("Failed to exec: {}", error);
                std::process::exit(1);
            }
        }
    }

    /// Wait for child process and handle signals
    fn wait_for_child(&self, child_pid: Pid, config: &ShimConfig) -> Result<i32> {
        let signal_config = config.signal_handling.as_ref();
        let kill_on_exit = signal_config.and_then(|s| s.kill_on_exit).unwrap_or(true);

        // Set up cleanup on exit
        if kill_on_exit {
            let child_pid_copy = child_pid;
            ctrlc::set_handler(move || {
                // Kill child process group
                let _ = signal::killpg(Pid::from_raw(child_pid_copy.as_raw()), Signal::SIGTERM);
                std::process::exit(130); // 128 + SIGINT
            })
            .context("Failed to set Ctrl+C handler")?;
        }

        // Wait for child process
        loop {
            match waitpid(child_pid, None) {
                Ok(WaitStatus::Exited(_, exit_code)) => {
                    return Ok(exit_code);
                }
                Ok(WaitStatus::Signaled(_, signal, _)) => {
                    // Child was killed by signal
                    return Ok(128 + signal as i32);
                }
                Ok(WaitStatus::Stopped(_, _)) => {
                    // Child was stopped, continue waiting
                    continue;
                }
                Ok(WaitStatus::Continued(_)) => {
                    // Child was continued, continue waiting
                    continue;
                }
                Ok(_) => {
                    // Other status, continue waiting
                    continue;
                }
                Err(nix::errno::Errno::EINTR) => {
                    // Interrupted by signal, continue waiting
                    continue;
                }
                Err(e) => {
                    return Err(anyhow::anyhow!("Failed to wait for child: {}", e));
                }
            }
        }
    }

    /// Execute using standard Command for simpler cases
    fn execute_simple(&self, mut command: Command, config: &ShimConfig) -> Result<i32> {
        self.setup_signal_handling(config)?;

        let mut child = command.spawn().context("Failed to spawn child process")?;
        let exit_status = child.wait().context("Failed to wait for child process")?;

        Ok(exit_status.code().unwrap_or(1))
    }
}

impl PlatformExecutorTrait for UnixExecutor {
    fn execute(&self, command: Command, config: &ShimConfig) -> Result<i32> {
        let signal_config = config.signal_handling.as_ref();
        let use_fork = signal_config
            .and_then(|s| s.forward_signals)
            .unwrap_or(true);

        if use_fork {
            self.execute_with_fork(command, config)
        } else {
            self.execute_simple(command, config)
        }
    }
}

// Extension trait for Command::exec
trait CommandExt {
    fn exec(&mut self) -> std::io::Error;
}

impl CommandExt for Command {
    fn exec(&mut self) -> std::io::Error {
        use std::os::unix::process::CommandExt as StdCommandExt;
        // exec() either succeeds (never returns) or fails (returns Error)
        StdCommandExt::exec(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unix_executor_creation() {
        let executor = UnixExecutor::new();
        assert!(!executor.signal_setup.load(Ordering::Relaxed));
    }

    #[test]
    fn test_signal_setup() {
        let executor = UnixExecutor::new();
        let config = ShimConfig {
            path: "/bin/echo".to_string(),
            args: None,
            working_dir: None,
            env: None,
            hide_console: None,
            run_as_admin: None,
            signal_handling: None,
        };

        // This should not fail
        let result = executor.setup_signal_handling(&config);
        assert!(result.is_ok());
        assert!(executor.signal_setup.load(Ordering::Relaxed));
    }
}
