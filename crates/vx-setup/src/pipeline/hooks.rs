//! Hook execution for setup pipeline

use crate::types::HookCommand;
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::Path;
use std::process::{Command, Stdio};

/// Hook execution result
#[derive(Debug, Clone)]
pub struct HookResult {
    /// Hook name
    pub name: String,
    /// Whether the hook succeeded
    pub success: bool,
    /// Exit code (if available)
    pub exit_code: Option<i32>,
    /// Error message (if failed)
    pub error: Option<String>,
    /// Output (if captured)
    pub output: Option<String>,
}

/// Hook executor for setup pipeline
pub struct HookExecutor {
    /// Working directory for hook execution
    working_dir: std::path::PathBuf,
    /// Whether to show output
    verbose: bool,
    /// Shell to use
    shell: String,
    /// Environment variables to set
    env_vars: HashMap<String, String>,
}

impl HookExecutor {
    /// Create a new hook executor
    pub fn new(working_dir: impl AsRef<Path>) -> Self {
        let shell = if cfg!(windows) {
            "powershell".to_string()
        } else {
            std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string())
        };

        Self {
            working_dir: working_dir.as_ref().to_path_buf(),
            verbose: false,
            shell,
            env_vars: HashMap::new(),
        }
    }

    /// Set verbose mode
    pub fn verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    /// Set shell
    #[allow(dead_code)]
    pub fn shell(mut self, shell: impl Into<String>) -> Self {
        self.shell = shell.into();
        self
    }

    /// Add environment variable
    #[allow(dead_code)]
    pub fn env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env_vars.insert(key.into(), value.into());
        self
    }

    /// Add multiple environment variables
    pub fn envs(mut self, vars: HashMap<String, String>) -> Self {
        self.env_vars.extend(vars);
        self
    }

    /// Execute a hook command
    pub fn execute(&self, name: &str, hook: &HookCommand) -> Result<HookResult> {
        let commands = hook.as_vec();
        let mut combined_output = String::new();

        for cmd in &commands {
            if cmd.trim().is_empty() {
                continue;
            }

            let result = self.run_command(name, cmd)?;
            if let Some(output) = &result.output {
                combined_output.push_str(output);
                combined_output.push('\n');
            }
            if !result.success {
                return Ok(HookResult {
                    output: Some(combined_output),
                    ..result
                });
            }
        }

        Ok(HookResult {
            name: name.to_string(),
            success: true,
            exit_code: Some(0),
            error: None,
            output: if combined_output.is_empty() {
                None
            } else {
                Some(combined_output)
            },
        })
    }

    /// Run a single command
    fn run_command(&self, name: &str, cmd: &str) -> Result<HookResult> {
        let (shell_cmd, shell_arg) = if cfg!(windows) {
            if self.shell.contains("powershell") || self.shell.contains("pwsh") {
                (&self.shell as &str, "-Command")
            } else {
                ("cmd", "/C")
            }
        } else {
            (&self.shell as &str, "-c")
        };

        let mut command = Command::new(shell_cmd);
        command.arg(shell_arg).arg(cmd);
        command.current_dir(&self.working_dir);

        // Set environment variables
        for (key, value) in &self.env_vars {
            command.env(key, value);
        }

        if self.verbose {
            command.stdout(Stdio::inherit());
            command.stderr(Stdio::inherit());
        } else {
            command.stdout(Stdio::piped());
            command.stderr(Stdio::piped());
        }

        let output = command
            .output()
            .with_context(|| format!("Failed to execute hook '{}': {}", name, cmd))?;

        let exit_code = output.status.code();
        let success = output.status.success();

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        let error = if !success {
            if stderr.is_empty() {
                Some(format!(
                    "Hook '{}' failed with exit code {:?}",
                    name, exit_code
                ))
            } else {
                Some(stderr.clone())
            }
        } else {
            None
        };

        Ok(HookResult {
            name: name.to_string(),
            success,
            exit_code,
            error,
            output: if stdout.is_empty() && stderr.is_empty() {
                None
            } else {
                Some(format!("{}{}", stdout, stderr))
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_hook_executor_single_command() {
        let executor = HookExecutor::new(env::current_dir().unwrap());
        let hook = HookCommand::Single("echo hello".to_string());
        let result = executor.execute("test", &hook).unwrap();
        assert!(result.success);
        assert_eq!(result.exit_code, Some(0));
    }

    #[test]
    fn test_hook_executor_multiple_commands() {
        let executor = HookExecutor::new(env::current_dir().unwrap());
        let hook = HookCommand::Multiple(vec!["echo first".to_string(), "echo second".to_string()]);
        let result = executor.execute("test", &hook).unwrap();
        assert!(result.success);
    }

    #[test]
    fn test_hook_executor_empty_command() {
        let executor = HookExecutor::new(env::current_dir().unwrap());
        let hook = HookCommand::Single("".to_string());
        let result = executor.execute("test", &hook).unwrap();
        assert!(result.success);
    }

    #[test]
    fn test_hook_executor_failing_command() {
        let executor = HookExecutor::new(env::current_dir().unwrap());
        let hook = HookCommand::Single("exit 1".to_string());
        let result = executor.execute("test", &hook).unwrap();
        assert!(!result.success);
        assert!(result.error.is_some());
    }
}
