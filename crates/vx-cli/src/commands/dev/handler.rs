//! Dev command handler

use super::export::handle_export;
use super::info::handle_info;
use super::shell::spawn_dev_shell;
use super::Args;
use super::tools::get_registry;
use crate::commands::common::load_config_view_cwd;
use crate::commands::setup::ConfigView;
use crate::ui::UI;
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::env;
use std::path::Path;
use std::process::Command;
use vx_env::{ToolEnvironment, ToolSpec};
use vx_project_analyzer::{AnalyzerConfig, ProjectAnalyzer};

/// Handle dev command with Args
pub async fn handle(args: &Args) -> Result<()> {
    // Use common configuration loading
    let (config_path, mut config) = load_config_view_cwd()?;

    // Override isolation mode if --inherit-system is specified
    if args.inherit_system {
        config.isolation = false;
    }

    // Merge CLI passenv with config passenv
    if !args.passenv.is_empty() {
        config.passenv.extend(args.passenv.clone());
    }

    if config.tools.is_empty() {
        UI::warn("No tools configured in vx.toml");
        UI::hint("Run 'vx init' to initialize the project configuration");
        return Ok(());
    }

    // Handle --export mode
    if args.export {
        return handle_export(&config, args.format.clone()).await;
    }

    // Handle --info mode
    if args.info {
        return handle_info(&config).await;
    }

    // Check and install missing tools if needed
    if !args.no_install {
        let auto_install = config
            .settings
            .get("auto_install")
            .map(|v| v == "true")
            .unwrap_or(true);

        if auto_install {
            // Reuse sync's tool installation logic to avoid duplication
            let (registry, _) = get_registry()?;
            crate::commands::sync::handle(
                &registry,
                false,   // check: false - we want to install
                false,   // force: false
                false,   // dry_run: false
                args.verbose,
                false,   // no_parallel: false - dev prefers parallel
                false,   // no_auto_install: false
            ).await?;
        }
    }

    // Build the environment
    let env_vars = build_dev_environment(&config, args.verbose).await?;

    // Execute command or spawn shell
    if let Some(cmd) = &args.command {
        execute_command_in_env(cmd, &env_vars)?;
    } else {
        spawn_dev_shell(args.shell.clone(), &env_vars, &config, Some(config_path.clone()))?;
    }

    Ok(())
}

/// Build environment variables for the dev shell
async fn build_dev_environment(config: &ConfigView, verbose: bool) -> Result<HashMap<String, String>> {
    // Merge env from vx.toml with setenv from settings
    let mut env_vars = config.env.clone();
    env_vars.extend(config.setenv.clone());

    // Get registry to query runtime bin directories
    let (registry, context) = get_registry()?;

    // Create ToolSpecs with proper bin directories from runtime providers
    let mut tool_specs = Vec::new();
    for (tool_name, version) in &config.tools {
        // Find the runtime for this tool to get bin directories
        let (bin_dirs, resolved_bin_dir) = if let Some(provider) = registry.providers().iter().find(|p| p.supports(tool_name)) {
            if let Some(runtime) = provider.get_runtime(tool_name) {
                // Call prepare_environment to get runtime-specific environment variables
                if let Ok(runtime_env) = runtime.prepare_environment(version, &context).await {
                    // Merge runtime-specific environment variables (e.g., MSVC's INCLUDE, LIB)
                    for (key, value) in runtime_env {
                        if verbose {
                            UI::info(&format!("  Runtime env: {}={}", key, value));
                        }
                        env_vars.insert(key, value);
                    }
                }
                
                // Try to get the resolved bin directory from the runtime
                let resolved = if let Ok(Some(exe_path)) = runtime.get_executable_path_for_version(version, &context).await {
                    // Get the parent directory of the executable as the bin directory
                    exe_path.parent().map(|p| p.to_path_buf())
                } else {
                    None
                };
                
                let dirs = runtime.possible_bin_dirs().into_iter().map(|s| s.to_string()).collect();
                (dirs, resolved)
            } else {
                (vec!["bin".to_string()], None)
            }
        } else {
            (vec!["bin".to_string()], None)
        };

        let mut spec = ToolSpec::with_bin_dirs(tool_name.clone(), version.clone(), bin_dirs);
        if let Some(bin_dir) = resolved_bin_dir {
            spec = spec.set_resolved_bin_dir(bin_dir);
        }
        tool_specs.push(spec);
    }

    // Use ToolEnvironment from vx-env with isolation settings
    let mut builder = ToolEnvironment::new()
        .tools_from_specs(tool_specs)
        .env_vars(&env_vars)
        .warn_missing(verbose)
        .isolation(config.isolation);

    // Add passenv patterns if in isolation mode
    if config.isolation && !config.passenv.is_empty() {
        builder = builder.passenv(config.passenv.clone());
    }

    let mut env_result = builder.build()?;

    // Set VX_DEV environment variable to indicate we're in a dev shell
    env_result.insert("VX_DEV".to_string(), "1".to_string());

    // Set VX_PROJECT_NAME for prompt customization
    env_result.insert("VX_PROJECT_NAME".to_string(), config.project_name.clone());

    // Set VX_PROJECT_ROOT
    if let Ok(current_dir) = env::current_dir() {
        env_result.insert(
            "VX_PROJECT_ROOT".to_string(),
            current_dir.to_string_lossy().to_string(),
        );
    }

    // Log tool paths if verbose
    if verbose {
        if config.isolation {
            UI::info("Running in isolation mode");
            if !config.passenv.is_empty() {
                UI::info(&format!("  passenv: {}", config.passenv.join(", ")));
            }
        }
        if let Some(path) = env_result.get("PATH") {
            let sep = if cfg!(windows) { ";" } else { ":" };
            for entry in path.split(sep).take(config.tools.len() + 1) {
                UI::info(&format!("  PATH: {}", entry));
            }
        }
    }

    Ok(env_result)
}

/// Build environment variables for script execution
///
/// Uses vx-env's ToolEnvironment for consistent environment building.
pub fn build_script_environment(config: &ConfigView) -> Result<HashMap<String, String>> {
    // Merge env from vx.toml with setenv from settings
    let mut env_vars = config.env.clone();
    env_vars.extend(config.setenv.clone());

    let mut builder = ToolEnvironment::new()
        .tools(&config.tools)
        .env_vars(&env_vars)
        .isolation(config.isolation);

    // Add passenv patterns if in isolation mode
    if config.isolation && !config.passenv.is_empty() {
        builder = builder.passenv(config.passenv.clone());
    }

    builder.build()
}

/// Execute a command in the dev environment
fn execute_command_in_env(cmd: &[String], env_vars: &HashMap<String, String>) -> Result<()> {
    if cmd.is_empty() {
        return Err(anyhow::anyhow!("No command specified"));
    }

    let program = &cmd[0];
    let args = &cmd[1..];

    // Clear inherited environment and set only our variables
    let mut command = Command::new(program);
    command.args(args);
    command.env_clear();

    // Set all environment variables from our isolated/configured environment
    for (key, value) in env_vars {
        command.env(key, value);
    }

    let status = command
        .status()
        .with_context(|| format!("Failed to execute: {}", program))?;

    if !status.success() {
        std::process::exit(vx_resolver::exit_code_from_status(&status));
    }

    Ok(())
}
