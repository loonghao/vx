//! Platform-specific execution implementations

use anyhow::Result;
use std::process::Command;

use crate::config::ShimConfig;

#[cfg(unix)]
mod unix;
#[cfg(windows)]
mod windows;

/// Platform-specific process executor
pub struct PlatformExecutor {
    #[cfg(windows)]
    inner: windows::WindowsExecutor,
    #[cfg(unix)]
    inner: unix::UnixExecutor,
}

impl PlatformExecutor {
    /// Create a new platform executor
    pub fn new() -> Self {
        Self {
            #[cfg(windows)]
            inner: windows::WindowsExecutor::new(),
            #[cfg(unix)]
            inner: unix::UnixExecutor::new(),
        }
    }

    /// Execute a command with platform-specific behavior
    pub fn execute(&self, command: Command, config: &ShimConfig) -> Result<i32> {
        self.inner.execute(command, config)
    }
}

impl Default for PlatformExecutor {
    fn default() -> Self {
        Self::new()
    }
}

/// Trait for platform-specific execution
trait PlatformExecutorTrait {
    fn execute(&self, command: Command, config: &ShimConfig) -> Result<i32>;
}
