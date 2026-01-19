//! Extension executor - runs extension scripts using vx-managed runtimes

use crate::config::CommandConfig;
use crate::error::{ExtensionError, ExtensionResult};
use crate::{Extension, ExtensionConfig};
use std::collections::HashMap;
use std::path::Path;
use std::process::Stdio;
use tokio::process::Command;
use tracing::{debug, info};
use vx_args::{ArgParser, HelpFormatter, Interpolator, ParsedArgs};
use vx_core::exit_code_from_status;

/// Extension executor - executes extension scripts
pub struct ExtensionExecutor {
    /// Environment variables to inject
    env_vars: HashMap<String, String>,
    /// Variable interpolator
    interpolator: Interpolator,
    /// Whether to load .env files
    load_dotenv: bool,
}

impl ExtensionExecutor {
    /// Create a new extension executor
    pub fn new() -> Self {
        Self {
            env_vars: HashMap::new(),
            interpolator: Interpolator::new().allow_missing(true),
            load_dotenv: true,
        }
    }

    /// Disable .env file loading
    pub fn without_dotenv(mut self) -> Self {
        self.load_dotenv = false;
        self
    }

    /// Execute an extension command
    pub async fn execute(
        &self,
        extension: &Extension,
        subcommand: Option<&str>,
        args: &[String],
    ) -> ExtensionResult<i32> {
        let config = &extension.config;

        // Check for help flag
        if args.iter().any(|a| a == "--help" || a == "-h") {
            self.print_help(extension, subcommand)?;
            return Ok(0);
        }

        // Determine which script to run
        let (script_path, script_args, cmd_config) = self.resolve_script(extension, subcommand)?;

        // Verify script exists
        if !script_path.exists() {
            return Err(ExtensionError::script_not_found(
                &extension.name,
                &script_path,
                &extension.path,
            ));
        }

        // Parse arguments if definitions exist
        let parsed_args = self.parse_arguments(extension, subcommand, args)?;

        // Build environment variables
        let env_map = self.build_env_vars(extension, &parsed_args, cmd_config.as_ref())?;

        // Get the runtime to use
        let runtime = config.runtime.runtime_name().unwrap_or("python");

        // Build final arguments
        let final_args = self.build_final_args(&script_args, args, &parsed_args)?;

        // Build the command
        let mut cmd = self.build_command(runtime, &script_path, &final_args)?;

        // Set working directory to extension directory
        cmd.current_dir(&extension.path);

        // Inject environment variables
        self.inject_env_vars(&mut cmd, extension, &env_map);

        info!(
            "Executing extension '{}' with {} runtime",
            extension.name, runtime
        );
        debug!("Script: {:?}", script_path);
        debug!("Args: {:?}", final_args);

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

        let exit_code = exit_code_from_status(&status);
        if !status.success() {
            debug!(
                "Extension '{}' exited with code {}",
                extension.name, exit_code
            );
        }

        Ok(exit_code)
    }

    /// Print help for an extension or subcommand
    fn print_help(&self, extension: &Extension, subcommand: Option<&str>) -> ExtensionResult<()> {
        let config = &extension.config;

        if let Some(subcmd) = subcommand {
            // Help for specific subcommand
            if let Some(cmd_config) = config.commands.get(subcmd) {
                let parser = self.build_parser_for_command(subcmd, cmd_config);
                let formatter = HelpFormatter::new();
                println!("{}", formatter.format(&parser));
            } else {
                println!("Unknown command: {}", subcmd);
                self.print_extension_help(extension)?;
            }
        } else {
            self.print_extension_help(extension)?;
        }

        Ok(())
    }

    /// Print general help for an extension
    fn print_extension_help(&self, extension: &Extension) -> ExtensionResult<()> {
        let config = &extension.config;

        println!("{} v{}", extension.name, config.extension.version);
        if !config.extension.description.is_empty() {
            println!("{}", config.extension.description);
        }
        println!();

        if !config.commands.is_empty() {
            println!("Commands:");
            for (name, cmd) in &config.commands {
                println!("  {:16} {}", name, cmd.description);
            }
            println!();
        }

        if config.entrypoint.main.is_some() && !config.entrypoint.arguments.is_empty() {
            println!("Arguments:");
            for arg in &config.entrypoint.arguments {
                let flag = if arg.positional {
                    format!("<{}>", arg.name)
                } else if let Some(short) = &arg.short {
                    format!("-{}, --{}", short, arg.name.replace('_', "-"))
                } else {
                    format!("    --{}", arg.name.replace('_', "-"))
                };
                let help = arg.help.as_deref().unwrap_or("");
                println!("  {:20} {}", flag, help);
            }
            println!();
        }

        println!(
            "Run '{} --help' for more information on a command.",
            extension.name
        );

        Ok(())
    }

    /// Build argument parser for a command
    fn build_parser_for_command(&self, name: &str, cmd_config: &CommandConfig) -> ArgParser {
        let mut parser = ArgParser::new(name);

        for arg_def in &cmd_config.arguments {
            let arg = arg_def.to_arg_def();
            if arg_def.positional {
                parser.positional(arg);
            } else {
                parser.add_arg(arg);
            }
        }

        parser
    }

    /// Parse arguments according to extension definition
    fn parse_arguments(
        &self,
        extension: &Extension,
        subcommand: Option<&str>,
        args: &[String],
    ) -> ExtensionResult<Option<ParsedArgs>> {
        let config = &extension.config;

        // Get argument definitions
        let arg_defs = if let Some(subcmd) = subcommand {
            config
                .commands
                .get(subcmd)
                .map(|c| &c.arguments)
                .filter(|a| !a.is_empty())
        } else {
            Some(&config.entrypoint.arguments).filter(|a| !a.is_empty())
        };

        let Some(arg_defs) = arg_defs else {
            return Ok(None);
        };

        // Build parser
        let mut parser = ArgParser::new(&extension.name).allow_unknown(true);

        for arg_def in arg_defs {
            let arg = arg_def.to_arg_def();
            if arg_def.positional {
                parser.positional(arg);
            } else {
                parser.add_arg(arg);
            }
        }

        // Parse arguments
        let parsed = parser
            .parse(args)
            .map_err(|e| ExtensionError::ArgumentError {
                extension: extension.name.clone(),
                message: e.to_string(),
            })?;

        Ok(Some(parsed))
    }

    /// Build environment variables map
    fn build_env_vars(
        &self,
        extension: &Extension,
        parsed_args: &Option<ParsedArgs>,
        cmd_config: Option<&CommandConfig>,
    ) -> ExtensionResult<HashMap<String, String>> {
        let mut env_map = HashMap::new();
        let config = &extension.config;

        // Load .env files if enabled
        if self.load_dotenv {
            // Load from extension directory
            let dotenv_path = extension.path.join(".env");
            if dotenv_path.exists() {
                if let Ok(iter) = dotenvy::from_path_iter(&dotenv_path) {
                    for item in iter.flatten() {
                        env_map.insert(item.0, item.1);
                    }
                }
            }

            // Load from current directory
            if let Ok(cwd) = std::env::current_dir() {
                let project_dotenv = cwd.join(".env");
                if project_dotenv.exists() {
                    if let Ok(iter) = dotenvy::from_path_iter(&project_dotenv) {
                        for item in iter.flatten() {
                            env_map.insert(item.0, item.1);
                        }
                    }
                }
            }
        }

        // Add extension-level env vars
        for (key, value) in &config.env {
            let interpolated = self.interpolate_value(value, &env_map)?;
            env_map.insert(key.clone(), interpolated);
        }

        // Add command-level env vars
        if let Some(cmd) = cmd_config {
            for (key, value) in &cmd.env {
                let interpolated = self.interpolate_value(value, &env_map)?;
                env_map.insert(key.clone(), interpolated);
            }
        }

        // Add parsed arguments as env vars
        if let Some(parsed) = parsed_args {
            for (key, value) in parsed.to_env_map() {
                env_map.insert(format!("VX_ARG_{}", key), value);
            }
        }

        // Add custom env vars
        for (key, value) in &self.env_vars {
            env_map.insert(key.clone(), value.clone());
        }

        Ok(env_map)
    }

    /// Interpolate a value using the interpolator
    fn interpolate_value(
        &self,
        value: &str,
        env_map: &HashMap<String, String>,
    ) -> ExtensionResult<String> {
        self.interpolator.interpolate(value, env_map).map_err(|e| {
            ExtensionError::InterpolationError {
                message: e.to_string(),
            }
        })
    }

    /// Build final arguments to pass to the script
    fn build_final_args(
        &self,
        script_args: &[String],
        user_args: &[String],
        parsed_args: &Option<ParsedArgs>,
    ) -> ExtensionResult<Vec<String>> {
        let mut final_args = Vec::new();

        // Add script default args (with interpolation)
        let env_map: HashMap<String, String> = HashMap::new();
        for arg in script_args {
            let interpolated = self.interpolate_value(arg, &env_map)?;
            final_args.push(interpolated);
        }

        // If we have parsed args, use passthrough for remaining args
        if let Some(parsed) = parsed_args {
            final_args.extend(parsed.passthrough().iter().cloned());
        } else {
            // Otherwise, pass all user args directly
            final_args.extend(user_args.iter().cloned());
        }

        Ok(final_args)
    }

    /// Resolve which script to execute
    fn resolve_script(
        &self,
        extension: &Extension,
        subcommand: Option<&str>,
    ) -> ExtensionResult<(std::path::PathBuf, Vec<String>, Option<CommandConfig>)> {
        let config = &extension.config;

        if let Some(subcmd) = subcommand {
            // Look for subcommand script
            if let Some(cmd_config) = config.get_command_script(subcmd) {
                let script_path = extension.path.join(&cmd_config.script);
                return Ok((
                    script_path,
                    cmd_config.args.clone(),
                    Some(cmd_config.clone()),
                ));
            }

            // If no specific command, try main script with subcommand as arg
            if let Some(main) = config.get_main_script() {
                let script_path = extension.path.join(main);
                let mut args = config.entrypoint.args.clone();
                args.insert(0, subcmd.to_string());
                return Ok((script_path, args, None));
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
            return Ok((script_path, config.entrypoint.args.clone(), None));
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
        args: &[String],
    ) -> ExtensionResult<Command> {
        // For now, we'll use the runtime directly
        // In the future, this should integrate with vx-runtime to get the correct path
        let interpreter = self.get_interpreter(runtime);

        let mut cmd = Command::new(&interpreter);

        // Add the script
        cmd.arg(script_path);

        // Add args
        cmd.args(args);

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
    fn inject_env_vars(
        &self,
        cmd: &mut Command,
        extension: &Extension,
        extra_env: &HashMap<String, String>,
    ) {
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

        // Extra environment variables (from .env, config, and parsed args)
        for (key, value) in extra_env {
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
    use crate::config::{EntrypointConfig, ExtensionMetadata, ExtensionType, RuntimeRequirement};

    fn create_test_extension() -> Extension {
        let mut commands = HashMap::new();
        commands.insert(
            "hello".to_string(),
            CommandConfig {
                description: "Say hello".to_string(),
                script: "hello.py".to_string(),
                args: vec![],
                arguments: vec![],
                env: HashMap::new(),
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
                    arguments: vec![],
                },
                commands,
                hooks: HashMap::new(),
                env: HashMap::new(),
                extends: None,
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

        let (script, _args, _cmd) = executor.resolve_script(&ext, Some("hello")).unwrap();
        assert!(script.ends_with("hello.py"));
    }

    #[test]
    fn test_resolve_script_main() {
        let executor = ExtensionExecutor::new();
        let ext = create_test_extension();

        let (script, _args, _cmd) = executor.resolve_script(&ext, None).unwrap();
        assert!(script.ends_with("main.py"));
    }
}
