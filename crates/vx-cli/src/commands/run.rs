//! Run command implementation
//!
//! Executes scripts defined in vx.toml configuration file.
//! Scripts are executed with the proper vx-managed tool environment.
//!
//! ## Features
//!
//! - **Dynamic Arguments**: Scripts can define arguments with types, defaults, and validation
//! - **Variable Interpolation**: Use `{{var}}` syntax for dynamic values
//! - **Environment Variables**: Automatic loading from `.env` files and config
//! - **Passthrough Arguments**: Arguments after `--` are passed directly to the script

use anyhow::Result;
use std::collections::HashMap;
use std::path::Path;

use crate::commands::dev::build_script_environment;
use crate::commands::setup::{parse_vx_config, ConfigView};
use crate::ui::UI;
use vx_args::Interpolator;
use vx_env::execute_with_env;
use vx_paths::find_vx_config;

/// Handle the run command - execute a script from vx.toml
///
/// This function:
/// 1. Finds and parses vx.toml configuration
/// 2. Parses script arguments if defined
/// 3. Interpolates variables in the script command
/// 4. Builds the environment with vx-managed tools in PATH
/// 5. Executes the script
pub async fn handle(script_name: &str, args: &[String]) -> Result<()> {
    let current_dir = std::env::current_dir()?;

    // Find vx.toml (search current and parent directories)
    let config_path = find_vx_config(&current_dir).map_err(|e| anyhow::anyhow!("{}", e))?;
    let config = parse_vx_config(&config_path)?;

    // Check for help flag
    if args.iter().any(|a| a == "--help" || a == "-h") {
        print_script_help(script_name, &config)?;
        return Ok(());
    }

    // Get the script command
    let script_cmd = config.scripts.get(script_name).ok_or_else(|| {
        let available: Vec<_> = config.scripts.keys().collect();
        if available.is_empty() {
            anyhow::anyhow!("No scripts defined in vx.toml")
        } else {
            anyhow::anyhow!(
                "Script '{}' not found. Available scripts: {}",
                script_name,
                available
                    .iter()
                    .map(|s| s.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        }
    })?;

    // Build environment with vx-managed tools in PATH
    let mut env_vars = build_script_environment(&config)?;

    // Load .env files
    load_dotenv_files(&current_dir, &mut env_vars);

    // Add config env vars
    for (key, value) in &config.env {
        env_vars.insert(key.clone(), value.clone());
    }

    // Create interpolator with built-in variables
    let interpolator = Interpolator::new().allow_missing(true);

    // Build variable source from env vars and args
    let mut var_source: HashMap<String, String> = env_vars.clone();

    // Add arguments as variables
    for (i, arg) in args.iter().enumerate() {
        var_source.insert(format!("arg{}", i + 1), arg.clone());
        var_source.insert(i.to_string(), arg.clone());
    }
    var_source.insert("@".to_string(), args.join(" "));
    var_source.insert("#".to_string(), args.len().to_string());

    // Interpolate the script command
    let interpolated_cmd = interpolator.interpolate(script_cmd, &var_source)?;

    // Build the full command with remaining arguments
    let full_cmd = if args.is_empty() {
        interpolated_cmd
    } else {
        // Check if script uses argument placeholders
        let uses_placeholders = script_cmd.contains("{{") && script_cmd.contains("}}");
        if uses_placeholders {
            // Arguments already interpolated
            interpolated_cmd
        } else {
            // Append arguments directly
            format!("{} {}", interpolated_cmd, args.join(" "))
        }
    };

    UI::info(&format!("Running script '{}': {}", script_name, full_cmd));

    // Add parsed args as env vars (VX_ARG_*)
    for (key, value) in &var_source {
        if !key.starts_with("VX_")
            && !key.contains('.')
            && key.chars().all(|c| c.is_alphanumeric() || c == '_')
        {
            env_vars.insert(format!("VX_ARG_{}", key.to_uppercase()), value.clone());
        }
    }

    // Execute the script with the proper environment
    let status = execute_with_env(&full_cmd, &env_vars)?;

    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }

    Ok(())
}

/// Load .env files from the current directory
fn load_dotenv_files(dir: &Path, env_vars: &mut HashMap<String, String>) {
    // Load .env file
    let dotenv_path = dir.join(".env");
    if dotenv_path.exists() {
        if let Ok(iter) = dotenvy::from_path_iter(&dotenv_path) {
            for item in iter.flatten() {
                env_vars.insert(item.0, item.1);
            }
        }
    }

    // Load .env.local file (higher priority)
    let dotenv_local = dir.join(".env.local");
    if dotenv_local.exists() {
        if let Ok(iter) = dotenvy::from_path_iter(&dotenv_local) {
            for item in iter.flatten() {
                env_vars.insert(item.0, item.1);
            }
        }
    }
}

/// Print help for a script
fn print_script_help(script_name: &str, config: &ConfigView) -> Result<()> {
    if let Some(script_cmd) = config.scripts.get(script_name) {
        println!("Script: {}", script_name);
        println!("Command: {}", script_cmd);
        println!();
        println!("Usage: vx run {} [args...]", script_name);
        println!();
        println!("Arguments are passed directly to the script.");
        println!();
        println!("Variable Interpolation:");
        println!("  {{{{arg1}}}}          First argument");
        println!("  {{{{arg2}}}}          Second argument");
        println!("  {{{{@}}}}             All arguments");
        println!("  {{{{#}}}}             Number of arguments");
        println!("  {{{{env.VAR}}}}       Environment variable VAR");
        println!("  {{{{project.root}}}}  Project root directory");
        println!("  {{{{project.name}}}}  Project name");
        println!("  {{{{os.name}}}}       Operating system");
        println!("  {{{{vx.version}}}}    VX version");
        println!();
        println!("Examples:");
        println!("  vx run {} arg1 arg2", script_name);
        println!("  vx run {} -- --flag value", script_name);
    } else {
        println!("Script '{}' not found.", script_name);
        println!();
        println!("Run 'vx run --list' to see available scripts.");
    }
    Ok(())
}

/// List all available scripts in vx.toml
pub async fn handle_list() -> Result<()> {
    let current_dir = std::env::current_dir()?;
    let config_path = find_vx_config(&current_dir).map_err(|e| anyhow::anyhow!("{}", e))?;
    let config = parse_vx_config(&config_path)?;

    if config.scripts.is_empty() {
        UI::info("No scripts defined in vx.toml");
        UI::hint("Add scripts to your vx.toml:\n\n[scripts]\nbuild = \"cargo build\"\ntest = \"cargo test\"");
        return Ok(());
    }

    UI::info("Available scripts:");
    for (name, cmd) in &config.scripts {
        println!("  {} = \"{}\"", name, cmd);
    }

    Ok(())
}
