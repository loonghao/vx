//! Cross-platform process execution with signal handling

use anyhow::Result;
use std::process::{Command, Stdio};

use crate::config::ShimConfig;
use crate::platform::PlatformExecutor;

/// Cross-platform process executor
pub struct Executor {
    config: ShimConfig,
    platform: PlatformExecutor,
}

impl Executor {
    /// Create a new executor with the given configuration
    pub fn new(config: ShimConfig) -> Self {
        Self {
            config,
            platform: PlatformExecutor::new(),
        }
    }

    /// Execute the target program with the given arguments
    pub fn execute(&self, args: &[String]) -> Result<i32> {
        let target_path = self.config.resolved_path();
        let mut cmd_args = self.config.resolved_args();
        cmd_args.extend_from_slice(args);

        // Create the command
        let mut command = Command::new(&target_path);
        command.args(&cmd_args);

        // Set working directory if specified
        if let Some(working_dir) = self.config.resolved_working_dir() {
            command.current_dir(working_dir);
        }

        // Set environment variables
        for (key, value) in self.config.resolved_env() {
            command.env(key, value);
        }

        // Configure stdio
        command
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit());

        // Platform-specific execution
        self.platform.execute(command, &self.config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ShimConfig;
    use std::collections::HashMap;

    #[test]
    fn test_executor_creation() {
        let config = ShimConfig {
            path: "/bin/echo".to_string(),
            args: Some(vec!["hello".to_string()]),
            working_dir: None,
            env: None,
            hide_console: None,
            run_as_admin: None,
            signal_handling: None,
        };

        let executor = Executor::new(config);
        assert_eq!(executor.config.path, "/bin/echo");
    }

    #[test]
    fn test_executor_with_env() {
        let mut env = HashMap::new();
        env.insert("TEST_VAR".to_string(), "test_value".to_string());

        let config = ShimConfig {
            path: "/bin/echo".to_string(),
            args: None,
            working_dir: None,
            env: Some(env),
            hide_console: None,
            run_as_admin: None,
            signal_handling: None,
        };

        let executor = Executor::new(config);
        let resolved_env = executor.config.resolved_env();
        assert_eq!(
            resolved_env.get("TEST_VAR"),
            Some(&"test_value".to_string())
        );
    }
}
