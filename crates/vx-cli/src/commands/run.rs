//! Run command implementation
//!
//! Executes scripts defined in .vx.toml configuration file.
//! Scripts are executed with the proper vx-managed tool environment.

use anyhow::Result;
use std::path::Path;

use crate::commands::dev::build_script_environment;
use crate::commands::setup::parse_vx_config;
use crate::ui::UI;
use vx_env::execute_with_env;

/// Handle the run command - execute a script from .vx.toml
///
/// This function:
/// 1. Finds and parses .vx.toml configuration
/// 2. Builds the environment with vx-managed tools in PATH
/// 3. Executes the script using the script generator
pub async fn handle(script_name: &str, args: &[String]) -> Result<()> {
    let current_dir = std::env::current_dir()?;

    // Find .vx.toml (search current and parent directories)
    let config_path = find_vx_config(&current_dir)?;
    let config = parse_vx_config(&config_path)?;

    // Get the script command
    let script_cmd = config.scripts.get(script_name).ok_or_else(|| {
        let available: Vec<_> = config.scripts.keys().collect();
        if available.is_empty() {
            anyhow::anyhow!("No scripts defined in .vx.toml")
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

    UI::info(&format!("Running script '{}': {}", script_name, script_cmd));

    // Build the full command with arguments
    let full_cmd = if args.is_empty() {
        script_cmd.clone()
    } else {
        format!("{} {}", script_cmd, args.join(" "))
    };

    // Build environment with vx-managed tools in PATH
    let env_vars = build_script_environment(&config)?;

    // Execute the script with the proper environment
    let status = execute_with_env(&full_cmd, &env_vars)?;

    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }

    Ok(())
}

/// Find .vx.toml in current directory or parent directories
///
/// If VX_PROJECT_ROOT is set, only search in the current directory
/// (used for test isolation).
fn find_vx_config(start_dir: &Path) -> Result<std::path::PathBuf> {
    // Check if VX_PROJECT_ROOT is set (test isolation mode)
    if std::env::var("VX_PROJECT_ROOT").is_ok() {
        let config_path = start_dir.join(".vx.toml");
        if config_path.exists() {
            return Ok(config_path);
        }
        return Err(anyhow::anyhow!(
            "No .vx.toml found in current directory.\n\
             Run 'vx init' to create one."
        ));
    }

    // Normal mode: search up the directory tree
    let mut current = start_dir.to_path_buf();

    loop {
        let config_path = current.join(".vx.toml");
        if config_path.exists() {
            return Ok(config_path);
        }

        if !current.pop() {
            break;
        }
    }

    Err(anyhow::anyhow!(
        "No .vx.toml found in current directory or parent directories.\n\
         Run 'vx init' to create one."
    ))
}

/// List all available scripts in .vx.toml
pub async fn handle_list() -> Result<()> {
    let current_dir = std::env::current_dir()?;
    let config_path = find_vx_config(&current_dir)?;
    let config = parse_vx_config(&config_path)?;

    if config.scripts.is_empty() {
        UI::info("No scripts defined in .vx.toml");
        UI::hint("Add scripts to your .vx.toml:\n\n[scripts]\nbuild = \"cargo build\"\ntest = \"cargo test\"");
        return Ok(());
    }

    UI::info("Available scripts:");
    for (name, cmd) in &config.scripts {
        println!("  {} = \"{}\"", name, cmd);
    }

    Ok(())
}
