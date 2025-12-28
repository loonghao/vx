//! Extension executor - runs extension scripts using vx-managed runtimes

use crate::error::{ExtensionError, ExtensionResult};
use crate::{Extension, ExtensionConfig};
use std::collections::HashMap;
use std::path::Path;
use std::process::Stdio;
use tokio::process::Command;
use tracing::{debug, info};

/// Extension executor - executes extension scripts
pub struct ExtensionExecutor {
    /// Environment variables to inject
    env_vars: HashMap<String, String>,
}

impl ExtensionExecutor {
    /// Create a new extension executor
    pub fn new() -> Self {
        Self {
            env_vars: HashMap::new(),
        }
    }

    /// Execute an extension command
    pub async fn execute(
        &self,
        extension: &Extension,
        subcommand: Option<&str>,
        args: &[String],
    ) -> ExtensionResult<i32> {
        let config = &extension.config;

        // Determine which script to run
        let (script_path, script_args) = self.resolve_script(extension, subcommand)?;

        // Verify script exists
        if !script_path.exists() {
            return Err(ExtensionError::script_not_found(
                &extension.name,
                &script_path,
                &extension.path,
            ));
        }

        // Get the runtime to use
        let runtime = config.runtime.runtime_name().unwrap_or("python");

        // Build the command
        let mut cmd = self.build_command(runtime, &script_path, &script_args, args)?;

        // Set working directory to extension directory
        cmd.current_dir(&extension.path);

        // Inject environment variables
        self.inject_env_vars(&mut cmd, extension);

        info!(
            "Executing extension '{}' with {} runtime",
            extension.name, runtime
        );
        debug!("Script: {:?}", script_path);
        debug!("Args: {:?}", args);

        // Execute the command
        let status = cmd
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .await
            .map_err(|e| {
                ExtensionError::io(
                    format!("Failed to execute extension '{}': {}", extension.name, e),
                    Some(script_path.clone()),
                    e,
                )
            })?;

        let exit_code = status.code().unwrap_or(1);
        if !status.success() {
            debug!(
                "Extension '{}' exited with code {}",
                extension.name, exit_code
            );
        }

        Ok(exit_code)
    }

    /// Resolve which script to execute
    fn resolve_script(
        &self,
        extension: &Extension,
        subcommand: Option<&str>,
    ) -> ExtensionResult<(std::path::PathBuf, Vec<String>)> {
        let config = &extension.config;

        if let Some(subcmd) = subcommand {
            // Look for subcommand script
            if let Some(cmd_config) = config.get_command_script(subcmd) {
                let script_path = extension.path.join(&cmd_config.script);
                return Ok((script_path, cmd_config.args.clone()));
            }

            // If no specific command, try main script with subcommand as arg
            if let Some(main) = config.get_main_script() {
                let script_path = extension.path.join(main);
                let mut args = config.entrypoint.args.clone();
                args.insert(0, subcmd.to_string());
                return Ok((script_path, args));
            }

            return Err(ExtensionError::subcommand_not_found(
                &extension.name,
                subcmd,
                get_available_commands(config),
            ));
        }

        // No subcommand - use main entrypoint
        if let Some(main) = config.get_main_script() {
            let script_path = extension.path.join(main);
            return Ok((script_path, config.entrypoint.args.clone()));
        }

        Err(ExtensionError::no_entrypoint(
            &extension.name,
            get_available_commands(config),
        ))
    }

    /// Build the execution command
    fn build_command(
        &self,
        runtime: &str,
        script_path: &Path,
        script_args: &[String],
        user_args: &[String],
    ) -> ExtensionResult<Command> {
        // For now, we'll use the runtime directly
        // In the future, this should integrate with vx-runtime to get the correct path
        let interpreter = self.get_interpreter(runtime);

        let mut cmd = Command::new(&interpreter);

        // Add the script
        cmd.arg(script_path);

        // Add script default args
        cmd.args(script_args);

        // Add user args
        cmd.args(user_args);

        Ok(cmd)
    }

    /// Get the interpreter for a runtime
    fn get_interpreter(&self, runtime: &str) -> String {
        // Map runtime names to interpreter commands
        // In the future, this should use vx-runtime to get the actual path
        let interpreter = match runtime {
            "python" | "python3" => "python",
            "node" | "nodejs" => "node",
            "deno" => "deno run",
            "bun" => "bun run",
            "ruby" => "ruby",
            "perl" => "perl",
            "bash" | "sh" => {
                if cfg!(windows) {
                    "bash"
                } else {
                    "sh"
                }
            }
            "powershell" | "pwsh" => {
                if cfg!(windows) {
                    "powershell"
                } else {
                    "pwsh"
                }
            }
            other => other,
        };

        interpreter.to_string()
    }

    /// Inject environment variables into the command
    fn inject_env_vars(&self, cmd: &mut Command, extension: &Extension) {
        // VX version
        cmd.env("VX_VERSION", env!("CARGO_PKG_VERSION"));

        // Extension directory
        cmd.env("VX_EXTENSION_DIR", &extension.path);

        // Extension name
        cmd.env("VX_EXTENSION_NAME", &extension.name);

        // Project directory (current working directory)
        if let Ok(cwd) = std::env::current_dir() {
            cmd.env("VX_PROJECT_DIR", cwd);
        }

        // Runtimes directory
        if let Ok(vx_paths) = vx_paths::VxPaths::new() {
            cmd.env("VX_RUNTIMES_DIR", vx_paths.store_dir);
            cmd.env("VX_HOME", vx_paths.base_dir);
        }

        // Custom environment variables
        for (key, value) in &self.env_vars {
            cmd.env(key, value);
        }
    }

    /// Add a custom environment variable
    pub fn with_env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env_vars.insert(key.into(), value.into());
        self
    }
}

impl Default for ExtensionExecutor {
    fn default() -> Self {
        Self::new()
    }
}

/// Get available subcommands for an extension
pub fn get_available_commands(config: &ExtensionConfig) -> Vec<String> {
    let mut commands: Vec<String> = config.commands.keys().cloned().collect();
    commands.sort();
    commands
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{
        CommandConfig, EntrypointConfig, ExtensionMetadata, ExtensionType, RuntimeRequirement,
    };

    fn create_test_extension() -> Extension {
        let mut commands = HashMap::new();
        commands.insert(
            "hello".to_string(),
            CommandConfig {
                description: "Say hello".to_string(),
                script: "hello.py".to_string(),
                args: vec![],
            },
        );

        Extension {
            name: "test-ext".to_string(),
            config: ExtensionConfig {
                extension: ExtensionMetadata {
                    name: "test-ext".to_string(),
                    version: "1.0.0".to_string(),
                    description: "Test extension".to_string(),
                    extension_type: ExtensionType::Command,
                    authors: vec![],
                    license: None,
                },
                runtime: RuntimeRequirement {
                    requires: Some("python >= 3.10".to_string()),
                    dependencies: vec![],
                },
                entrypoint: EntrypointConfig {
                    main: Some("main.py".to_string()),
                    args: vec![],
                },
                commands,
                hooks: HashMap::new(),
            },
            path: std::path::PathBuf::from("/tmp/test-ext"),
            source: crate::ExtensionSource::User,
        }
    }

    #[test]
    fn test_get_available_commands() {
        let ext = create_test_extension();
        let commands = get_available_commands(&ext.config);
        assert!(commands.contains(&"hello".to_string()));
    }

    #[test]
    fn test_resolve_script_with_subcommand() {
        let executor = ExtensionExecutor::new();
        let ext = create_test_extension();

        let (script, _args) = executor.resolve_script(&ext, Some("hello")).unwrap();
        assert!(script.ends_with("hello.py"));
    }

    #[test]
    fn test_resolve_script_main() {
        let executor = ExtensionExecutor::new();
        let ext = create_test_extension();

        let (script, _args) = executor.resolve_script(&ext, None).unwrap();
        assert!(script.ends_with("main.py"));
    }
}
