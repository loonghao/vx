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

use crate::commands::common::load_config_view_cwd;
use crate::commands::dev::build_script_environment;
use crate::commands::setup::ConfigView;
use crate::ui::UI;
use vx_args::Interpolator;
use vx_config::ScriptConfig;
use vx_env::execute_with_env;

/// Handle the run command - execute a script from vx.toml
///
/// This function:
/// 1. Finds and parses vx.toml configuration
/// 2. Handles --list to show available scripts
/// 3. Handles -H/--script-help for script-specific help
/// 4. Separates script args from passthrough args (after --)
/// 5. Interpolates variables in the script command
/// 6. Builds the environment with vx-managed tools in PATH
/// 7. Executes the script
pub async fn handle(
    script_name: Option<&str>,
    list: bool,
    script_help: bool,
    args: &[String],
) -> Result<()> {
    // Use common configuration loading
    let (config_path, config) = load_config_view_cwd()?;

    // Handle --list flag
    if list {
        print_available_scripts(&config)?;
        return Ok(());
    }

    // Handle -H/--script-help flag
    if script_help {
        if let Some(name) = script_name {
            print_script_help(name, &config)?;
        } else {
            print_run_help(&config)?;
        }
        return Ok(());
    }

    // If no script name provided, show usage
    let script_name = match script_name {
        Some(name) => name,
        None => {
            print_run_help(&config)?;
            return Ok(());
        }
    };

    // Split args at -- separator
    let (script_args, passthrough_args) = split_args_at_separator(args);

    // Get the script config
    let script_config = config.scripts.get(script_name).ok_or_else(|| {
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

    // Extract command and details from ScriptConfig
    let (script_cmd, details) = match script_config {
        ScriptConfig::Simple(cmd) => (cmd.clone(), None),
        ScriptConfig::Detailed(d) => (d.command.clone(), Some(d)),
    };

    // -------------------------
    // Execute dependency scripts first (topological order)
    // -------------------------
    if let Some(details) = &details
        && !details.depends.is_empty()
    {
        execute_dependencies(&details.depends, &config, &config_path, args).await?;
    }

    // Build environment with vx-managed tools in PATH
    let mut env_vars = build_script_environment(&config)?;

    // Load .env files
    let current_dir = config_path.parent().unwrap();
    load_dotenv_files(current_dir, &mut env_vars);

    // Add config env vars
    for (key, value) in &config.env {
        env_vars.insert(key.clone(), value.clone());
    }

    // Add script-level env vars (override config-level)
    if let Some(details) = &details {
        for (key, value) in &details.env {
            env_vars.insert(key.clone(), value.clone());
        }
    }

    // Create interpolator with built-in variables
    let interpolator = Interpolator::new().allow_missing(true);

    // Build variable source from env vars and args
    let mut var_source: HashMap<String, String> = env_vars.clone();

    // Add script arguments as variables (before --)
    for (i, arg) in script_args.iter().enumerate() {
        var_source.insert(format!("arg{}", i + 1), arg.clone());
        var_source.insert(i.to_string(), arg.clone());
    }
    var_source.insert("@".to_string(), script_args.join(" "));
    var_source.insert("#".to_string(), script_args.len().to_string());

    // Add passthrough arguments as {{args}} variable (after --)

    // If no -- separator, use all args as passthrough for backward compatibility
    let effective_passthrough = if args.contains(&"--".to_string()) {
        passthrough_args
    } else {
        args.to_vec()
    };
    var_source.insert("args".to_string(), effective_passthrough.join(" "));

    // Interpolate the script command
    let interpolated_cmd = interpolator.interpolate(&script_cmd, &var_source)?;

    // Build the full command
    let full_cmd = if script_cmd.contains("{{args}}") {
        // Command uses {{args}} placeholder - already interpolated
        interpolated_cmd
    } else if !script_args.is_empty() {
        // Legacy behavior: append script args if no {{args}} placeholder
        let uses_placeholders = script_cmd.contains("{{") && script_cmd.contains("}}");
        if uses_placeholders {
            // Arguments already interpolated via other placeholders
            interpolated_cmd
        } else {
            // Append script arguments directly
            format!("{} {}", interpolated_cmd, script_args.join(" "))
        }
    } else {
        interpolated_cmd
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

    // Handle cwd override from script details
    if let Some(details) = &details
        && let Some(ref cwd) = details.cwd
    {
        let target_dir = if Path::new(cwd).is_absolute() {
            std::path::PathBuf::from(cwd)
        } else {
            current_dir.join(cwd)
        };
        std::env::set_current_dir(&target_dir)
            .map_err(|e| anyhow::anyhow!("Failed to change to script cwd '{}': {}", cwd, e))?;
    }

    // Execute the script with the proper environment
    let status = execute_with_env(&full_cmd, &env_vars)?;

    if !status.success() {
        // Use exit_code_from_status to handle Ctrl+C gracefully
        std::process::exit(vx_resolver::exit_code_from_status(&status));
    }

    Ok(())
}

/// Print general run command help
fn print_run_help(config: &ConfigView) -> Result<()> {
    println!("Run a script defined in vx.toml");
    println!();
    println!("Usage: vx run <SCRIPT> [ARGS...]");
    println!("       vx run --list");
    println!("       vx run <SCRIPT> -H");
    println!();
    println!("Options:");
    println!("  -l, --list         List available scripts");
    println!("  -H, --script-help  Show script-specific help");
    println!("  -h, --help         Show this help message");
    println!();
    println!("Arguments after the script name are passed to the script.");
    println!("Use {{{{args}}}} in script command to receive all arguments.");
    println!();
    print_available_scripts(config)?;
    Ok(())
}

/// Print available scripts
fn print_available_scripts(config: &ConfigView) -> Result<()> {
    if config.scripts.is_empty() {
        println!("No scripts defined in vx.toml");
        println!();
        println!("Add scripts to your vx.toml:");
        println!();
        println!("  [scripts]");
        println!("  test = \"cargo test\"");
        println!("  build = \"cargo build --release\"");
    } else {
        println!("Available scripts:");
        for (name, script) in &config.scripts {
            let (cmd, desc) = match script {
                ScriptConfig::Simple(s) => (s.as_str(), None),
                ScriptConfig::Detailed(d) => (d.command.as_str(), d.description.as_deref()),
            };
            // Truncate long commands
            let display_cmd = if cmd.len() > 50 {
                format!("{}...", &cmd[..47])
            } else {
                cmd.to_string()
            };
            if let Some(desc) = desc {
                println!("  {:<15} {} ({})", name, display_cmd, desc);
            } else {
                println!("  {:<15} {}", name, display_cmd);
            }
        }
    }
    Ok(())
}

/// Split arguments at -- separator
/// Returns (script_args, passthrough_args)
fn split_args_at_separator(args: &[String]) -> (Vec<String>, Vec<String>) {
    if let Some(pos) = args.iter().position(|arg| arg == "--") {
        let script_args = args[..pos].to_vec();
        let passthrough_args = args[pos + 1..].to_vec();
        (script_args, passthrough_args)
    } else {
        (args.to_vec(), Vec::new())
    }
}

/// Load .env files from the current directory
fn load_dotenv_files(dir: &Path, env_vars: &mut HashMap<String, String>) {
    // Load .env file
    let dotenv_path = dir.join(".env");
    if dotenv_path.exists()
        && let Ok(iter) = dotenvy::from_path_iter(&dotenv_path)
    {
        for item in iter.flatten() {
            env_vars.insert(item.0, item.1);
        }
    }

    // Load .env.local file (higher priority)
    let dotenv_local = dir.join(".env.local");
    if dotenv_local.exists()
        && let Ok(iter) = dotenvy::from_path_iter(&dotenv_local)
    {
        for item in iter.flatten() {
            env_vars.insert(item.0, item.1);
        }
    }
}

/// Print help for a script
fn print_script_help(script_name: &str, config: &ConfigView) -> Result<()> {
    if let Some(script_config) = config.scripts.get(script_name) {
        let (cmd, details) = match script_config {
            ScriptConfig::Simple(s) => (s.as_str(), None),
            ScriptConfig::Detailed(d) => (d.command.as_str(), Some(d)),
        };

        println!("Script: {}", script_name);
        if let Some(d) = &details
            && let Some(ref desc) = d.description
        {
            println!("Description: {}", desc);
        }
        println!("Command: {}", cmd);

        if let Some(d) = &details {
            if !d.depends.is_empty() {
                println!("Dependencies: {}", d.depends.join(", "));
            }
            if let Some(ref cwd) = d.cwd {
                println!("Working directory: {}", cwd);
            }
            if !d.env.is_empty() {
                println!("Environment:");
                for (k, v) in &d.env {
                    println!("  {} = {}", k, v);
                }
            }
        }

        println!();
        println!(
            "Usage: vx run {} [script-args...] [-- passthrough-args...]",
            script_name
        );

        println!();
        println!("Arguments:");
        println!("  script-args       Arguments for script interpolation");
        println!("  --                Separator for passthrough arguments");
        println!("  passthrough-args  Arguments passed directly to the command");
        println!();
        println!("Variable Interpolation:");
        println!("  {{{{arg1}}}}          First script argument");
        println!("  {{{{arg2}}}}          Second script argument");
        println!("  {{{{@}}}}             All script arguments");
        println!("  {{{{#}}}}             Number of script arguments");
        println!("  {{{{args}}}}          All passthrough arguments (after --)");
        println!("  {{{{env.VAR}}}}       Environment variable VAR");
        println!("  {{{{project.root}}}}  Project root directory");
        println!("  {{{{project.name}}}}  Project name");
        println!("  {{{{os.name}}}}       Operating system");
        println!("  {{{{vx.version}}}}    VX version");
        println!();
        println!("Examples:");
        println!("  vx run {} arg1 arg2", script_name);
        println!("  vx run {} -- --flag value", script_name);
        println!("  vx run {} script-arg -- --tool-flag", script_name);
    } else {
        println!("Script '{}' not found.", script_name);
        println!();
        println!("Run 'vx run --list' to see available scripts.");
    }
    Ok(())
}

/// Execute dependency scripts in topological order
///
/// Handles circular dependency detection and ensures each script runs at most once.
async fn execute_dependencies(
    depends: &[String],
    config: &ConfigView,
    config_path: &Path,
    _parent_args: &[String],
) -> Result<()> {
    let mut visited = std::collections::HashSet::new();
    let mut order = Vec::new();

    // Build topological order with cycle detection
    for dep in depends {
        topological_sort(dep, config, &mut visited, &mut Vec::new(), &mut order)?;
    }

    if order.is_empty() {
        return Ok(());
    }

    // Build shared environment
    let mut env_vars = build_script_environment(config)?;

    // Load .env files
    let current_dir = config_path.parent().unwrap();
    load_dotenv_files(current_dir, &mut env_vars);

    // Add config env vars
    for (key, value) in &config.env {
        env_vars.insert(key.clone(), value.clone());
    }

    // Execute each dependency in order
    for dep_name in &order {
        let script_config = config.scripts.get(dep_name.as_str()).ok_or_else(|| {
            anyhow::anyhow!("Dependency script '{}' not found in vx.toml", dep_name)
        })?;

        let (cmd, details) = match script_config {
            ScriptConfig::Simple(s) => (s.clone(), None),
            ScriptConfig::Detailed(d) => (d.command.clone(), Some(d)),
        };

        // Merge script-level env vars
        let mut dep_env = env_vars.clone();
        if let Some(d) = &details {
            for (k, v) in &d.env {
                dep_env.insert(k.clone(), v.clone());
            }
        }

        UI::info(&format!("Running dependency '{}': {}", dep_name, cmd));

        // Handle cwd for dependency
        let saved_dir = std::env::current_dir().ok();
        if let Some(d) = &details
            && let Some(ref cwd) = d.cwd
        {
            let target_dir = if Path::new(cwd).is_absolute() {
                std::path::PathBuf::from(cwd)
            } else {
                current_dir.join(cwd)
            };
            std::env::set_current_dir(&target_dir).map_err(|e| {
                anyhow::anyhow!("Failed to change to dependency cwd '{}': {}", cwd, e)
            })?;
        }

        let status = execute_with_env(&cmd, &dep_env)?;

        // Restore cwd
        if let Some(dir) = saved_dir {
            let _ = std::env::set_current_dir(dir);
        }

        if !status.success() {
            return Err(anyhow::anyhow!(
                "Dependency script '{}' failed with exit code {}",
                dep_name,
                vx_resolver::exit_code_from_status(&status)
            ));
        }
    }

    Ok(())
}

/// Topological sort with cycle detection using DFS
fn topological_sort(
    name: &str,
    config: &ConfigView,
    visited: &mut std::collections::HashSet<String>,
    stack: &mut Vec<String>,
    order: &mut Vec<String>,
) -> Result<()> {
    if visited.contains(name) {
        return Ok(());
    }

    // Cycle detection
    if stack.contains(&name.to_string()) {
        let cycle_start = stack.iter().position(|s| s == name).unwrap();
        let cycle: Vec<_> = stack[cycle_start..].to_vec();
        return Err(anyhow::anyhow!(
            "Circular dependency detected: {} -> {}",
            cycle.join(" -> "),
            name
        ));
    }

    stack.push(name.to_string());

    // Recurse into this script's dependencies
    if let Some(ScriptConfig::Detailed(d)) = config.scripts.get(name) {
        for dep in &d.depends {
            topological_sort(dep, config, visited, stack, order)?;
        }
    }

    stack.pop();
    visited.insert(name.to_string());
    order.push(name.to_string());

    Ok(())
}

/// List all available scripts in vx.toml
pub async fn handle_list() -> Result<()> {
    let (_config_path, config) = load_config_view_cwd()?;

    if config.scripts.is_empty() {
        UI::info("No scripts defined in vx.toml");
        UI::hint(
            "Add scripts to your vx.toml:\n\n[scripts]\nbuild = \"cargo build\"\ntest = \"cargo test\"",
        );
        return Ok(());
    }

    UI::info("Available scripts:");
    for (name, script) in &config.scripts {
        let cmd = match script {
            ScriptConfig::Simple(s) => s.as_str(),
            ScriptConfig::Detailed(d) => d.command.as_str(),
        };
        println!("  {} = \"{}\"", name, cmd);
    }

    Ok(())
}
