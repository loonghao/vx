//! Windows-specific process execution with signal handling

use anyhow::{Context, Result};
use std::process::Command;

use crate::config::ShimConfig;
use crate::platform::PlatformExecutorTrait;

/// Windows-specific process executor
pub struct WindowsExecutor;

impl WindowsExecutor {
    /// Create a new Windows executor
    pub fn new() -> Self {
        Self
    }

    /// Execute using standard Command for simplicity
    fn execute_simple(&self, mut command: Command, _config: &ShimConfig) -> Result<i32> {
        // Set up signal handling to ignore Ctrl+C
        ctrlc::set_handler(|| {
            // Ignore Ctrl+C in the shim, let the child process handle it
        })
        .context("Failed to set Ctrl+C handler")?;

        let mut child = command.spawn().context("Failed to spawn child process")?;
        let exit_status = child.wait().context("Failed to wait for child process")?;

        Ok(exit_status.code().unwrap_or(1))
    }
}

impl PlatformExecutorTrait for WindowsExecutor {
    fn execute(&self, command: Command, config: &ShimConfig) -> Result<i32> {
        self.execute_simple(command, config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_windows_executor_creation() {
        let _executor = WindowsExecutor::new();
        // Test passes if no panic occurs
    }
}
