//! Execute Stage - command execution
//!
//! The final stage of the execution pipeline. Takes a `PreparedExecution` and
//! spawns the process, returning the exit code.
//!
//! This stage wraps the existing `build_command` and `run_command` functions
//! from the `command` module.

use async_trait::async_trait;
use tracing::debug;

use crate::ResolutionResult;
use crate::executor::command::{build_command, run_command};
use crate::executor::pipeline::error::ExecuteError;
use crate::executor::pipeline::stage::Stage;
use vx_core::exit_code_from_status;

use super::prepare::PreparedExecution;

/// The Execute stage: `PreparedExecution` → `i32` (exit code)
///
/// Builds and spawns the command process, then waits for completion.
pub struct ExecuteStage {
    /// Optional execution timeout in seconds
    pub timeout: Option<std::time::Duration>,
}

impl ExecuteStage {
    /// Create a new ExecuteStage
    pub fn new() -> Self {
        Self { timeout: None }
    }

    /// Set execution timeout
    pub fn with_timeout(mut self, timeout: std::time::Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }
}

impl Default for ExecuteStage {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Stage<PreparedExecution, i32> for ExecuteStage {
    type Error = ExecuteError;

    async fn execute(&self, prepared: PreparedExecution) -> Result<i32, ExecuteError> {
        debug!(
            "[ExecuteStage] Executing: {} {:?}",
            prepared.executable.display(),
            prepared.args
        );

        // Verify executable exists (if absolute path)
        if prepared.executable.is_absolute() && !prepared.executable.exists() {
            return Err(ExecuteError::SpawnFailed {
                executable: prepared.executable.clone(),
                reason: format!(
                    "Executable not found at '{}'. Try running 'vx install {}'.",
                    prepared.executable.display(),
                    prepared.plan.primary.name
                ),
            });
        }

        // Build a ResolutionResult to pass to the existing build_command function.
        // This is a compatibility bridge — in Phase 2+ we'll refactor build_command
        // to take PreparedExecution directly.
        let resolution = ResolutionResult {
            runtime: prepared.plan.primary.name.clone(),
            executable: prepared.executable.clone(),
            command_prefix: prepared.command_prefix.clone(),
            missing_dependencies: vec![],
            install_order: vec![],
            runtime_needs_install: false,
            incompatible_dependencies: vec![],
            unsupported_platform_runtimes: vec![],
        };

        let mut cmd = build_command(
            &resolution,
            &prepared.args,
            &prepared.env,
            prepared.inherit_vx_path,
            prepared.vx_tools_path.clone(),
        )
        .map_err(|e| ExecuteError::SpawnFailed {
            executable: prepared.executable.clone(),
            reason: e.to_string(),
        })?;

        debug!(
            "[ExecuteStage] cmd: {} {:?}",
            prepared.executable.display(),
            prepared.args
        );

        let status = run_command(&mut cmd, self.timeout).await.map_err(|e| {
            let msg = e.to_string();
            if msg.contains("timed out") {
                ExecuteError::Timeout {
                    seconds: self.timeout.map(|d| d.as_secs()).unwrap_or(0),
                }
            } else {
                ExecuteError::SpawnFailed {
                    executable: prepared.executable.clone(),
                    reason: msg,
                }
            }
        })?;

        let code = exit_code_from_status(&status);
        debug!("[ExecuteStage] exit={}", code);

        Ok(code)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execute_stage_creation() {
        let stage = ExecuteStage::new();
        assert!(stage.timeout.is_none());
    }

    #[test]
    fn test_execute_stage_with_timeout() {
        let stage = ExecuteStage::new().with_timeout(std::time::Duration::from_secs(30));
        assert_eq!(stage.timeout, Some(std::time::Duration::from_secs(30)));
    }

    #[test]
    fn test_execute_stage_default() {
        let stage = ExecuteStage::default();
        assert!(stage.timeout.is_none());
    }
}
