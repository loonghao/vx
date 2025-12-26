//! Environment management commands
//!
//! This module provides commands for managing vx environments:
//! - create: Create a new environment
//! - use: Activate an environment
//! - list: List all environments
//! - delete: Remove an environment
//! - show: Show current environment details
//! - export: Export environment variables for shell activation

use crate::commands::dev::{generate_env_export, ExportFormat};
use crate::commands::setup::parse_vx_config;
use crate::ui::UI;
use anyhow::{Context, Result};
use clap::Subcommand;
use std::env;
use std::path::{Path, PathBuf};
use vx_paths::{link, LinkStrategy, PathManager};

/// Environment subcommands
#[derive(Subcommand, Clone)]
pub enum EnvCommand {
    /// Create a new environment
    Create {
        /// Environment name
        name: String,
        /// Clone from an existing environment
        #[arg(long)]
        from: Option<String>,
        /// Set as default environment after creation
        #[arg(long)]
        set_default: bool,
    },

    /// Activate an environment
    Use {
        /// Environment name
        name: String,
        /// Set as the global default
        #[arg(long)]
        global: bool,
    },

    /// List all environments
    #[command(alias = "ls")]
    List {
        /// Show detailed information
        #[arg(long)]
        detailed: bool,
    },

    /// Delete an environment
    #[command(alias = "rm")]
    Delete {
        /// Environment name
        name: String,
        /// Force deletion without confirmation
        #[arg(long)]
        force: bool,
    },

    /// Show current environment details
    Show {
        /// Environment name (defaults to current)
        name: Option<String>,
    },

    /// Add a runtime to an environment
    Add {
        /// Runtime and version (e.g., node@20.0.0)
        runtime_version: String,
        /// Environment name (defaults to current)
        #[arg(long)]
        env: Option<String>,
    },

    /// Remove a runtime from an environment
    Remove {
        /// Runtime name
        runtime: String,
        /// Environment name (defaults to current)
        #[arg(long)]
        env: Option<String>,
    },

    /// Export environment variables for shell activation
    ///
    /// Usage:
    ///   Bash/Zsh: eval "$(vx env export)"
    ///   PowerShell: Invoke-Expression (vx env export --format powershell)
    ///   GitHub Actions: vx env export --format github | source /dev/stdin
    #[command(alias = "activate")]
    Export {
        /// Output format: shell, powershell, batch, github (auto-detected if not specified)
        #[arg(long, short)]
        format: Option<String>,
    },
}

/// Handle environment commands
pub async fn handle(command: EnvCommand) -> Result<()> {
    match command {
        EnvCommand::Create {
            name,
            from,
            set_default,
        } => create_env(&name, from.as_deref(), set_default).await,
        EnvCommand::Use { name, global } => use_env(&name, global).await,
        EnvCommand::List { detailed } => list_envs(detailed).await,
        EnvCommand::Delete { name, force } => delete_env(&name, force).await,
        EnvCommand::Show { name } => show_env(name.as_deref()).await,
        EnvCommand::Add {
            runtime_version,
            env,
        } => add_runtime(&runtime_version, env.as_deref()).await,
        EnvCommand::Remove { runtime, env } => remove_runtime(&runtime, env.as_deref()).await,
        EnvCommand::Export { format } => export_env(format).await,
    }
}

/// Export environment variables for shell activation
async fn export_env(format: Option<String>) -> Result<()> {
    let current_dir = env::current_dir().context("Failed to get current directory")?;
    let config_path = current_dir.join(".vx.toml");

    if !config_path.exists() {
        anyhow::bail!("No .vx.toml found in current directory. Run 'vx init' first.");
    }

    let config = parse_vx_config(&config_path)?;

    let export_format = match format {
        Some(f) => ExportFormat::from_str(&f).ok_or_else(|| {
            anyhow::anyhow!(
                "Unknown format: {}. Use: shell, powershell, batch, or github",
                f
            )
        })?,
        None => ExportFormat::detect(),
    };

    let output = generate_env_export(&config, export_format)?;
    print!("{}", output);

    Ok(())
}

/// Create a new environment
async fn create_env(name: &str, from: Option<&str>, set_default: bool) -> Result<()> {
    let path_manager = PathManager::new()?;

    // Check if environment already exists
    if path_manager.env_exists(name) {
        anyhow::bail!("Environment '{}' already exists", name);
    }

    // Create the environment directory
    let env_dir = path_manager.create_env(name)?;
    UI::success(&format!(
        "Created environment '{}' at {}",
        name,
        env_dir.display()
    ));

    // Clone from existing environment if specified
    if let Some(source) = from {
        if !path_manager.env_exists(source) {
            anyhow::bail!("Source environment '{}' does not exist", source);
        }

        let source_dir = path_manager.env_dir(source);
        clone_env_contents(&source_dir, &env_dir)?;
        UI::info(&format!("Cloned from environment '{}'", source));
    }

    // Set as default if requested
    if set_default {
        set_default_env(name)?;
        UI::info(&format!("Set '{}' as default environment", name));
    }

    Ok(())
}

/// Clone environment contents
fn clone_env_contents(source: &Path, target: &Path) -> Result<()> {
    if !source.exists() {
        return Ok(());
    }

    for entry in std::fs::read_dir(source)? {
        let entry = entry?;
        let source_path = entry.path();
        let target_path = target.join(entry.file_name());

        if source_path.is_symlink() {
            // Recreate symlink pointing to the same target
            let link_target = std::fs::read_link(&source_path)?;
            link::create_link(&link_target, &target_path, LinkStrategy::SymLink)
                .context("Failed to create symlink")?;
        } else if source_path.is_file() {
            std::fs::copy(&source_path, &target_path)?;
        } else if source_path.is_dir() {
            std::fs::create_dir_all(&target_path)?;
            clone_env_contents(&source_path, &target_path)?;
        }
    }

    Ok(())
}

/// Set the default environment
fn set_default_env(name: &str) -> Result<()> {
    let path_manager = PathManager::new()?;
    let config_dir = path_manager.config_dir();
    let default_file = config_dir.join("default-env");

    std::fs::create_dir_all(config_dir)?;
    std::fs::write(&default_file, name)?;

    Ok(())
}

/// Get the current default environment
fn get_default_env() -> Result<String> {
    let path_manager = PathManager::new()?;
    let config_dir = path_manager.config_dir();
    let default_file = config_dir.join("default-env");

    if default_file.exists() {
        let name = std::fs::read_to_string(&default_file)?;
        Ok(name.trim().to_string())
    } else {
        Ok("default".to_string())
    }
}

/// Activate an environment
async fn use_env(name: &str, global: bool) -> Result<()> {
    let path_manager = PathManager::new()?;

    // Check if environment exists
    if !path_manager.env_exists(name) {
        anyhow::bail!(
            "Environment '{}' does not exist. Create it with 'vx env create {}'",
            name,
            name
        );
    }

    if global {
        set_default_env(name)?;
        UI::success(&format!("Set '{}' as the default environment", name));
    } else {
        // Print shell activation instructions
        let env_dir = path_manager.env_dir(name);
        UI::info(&format!(
            "To activate environment '{}' in current shell:",
            name
        ));

        #[cfg(windows)]
        {
            println!("  $env:VX_ENV = \"{}\"", name);
            println!("  $env:PATH = \"{};$env:PATH\"", env_dir.display());
        }

        #[cfg(not(windows))]
        {
            println!("  export VX_ENV=\"{}\"", name);
            println!("  export PATH=\"{}:$PATH\"", env_dir.display());
        }

        println!();
        UI::hint(
            "Or add 'eval \"$(vx shell init)\"' to your shell profile for automatic activation",
        );
    }

    Ok(())
}

/// List all environments
async fn list_envs(detailed: bool) -> Result<()> {
    let path_manager = PathManager::new()?;
    let envs = path_manager.list_envs()?;
    let current_env = get_default_env().unwrap_or_else(|_| "default".to_string());

    if envs.is_empty() {
        UI::info("No environments found. Create one with 'vx env create <name>'");
        return Ok(());
    }

    println!("Environments:");
    println!();

    for env_name in &envs {
        let is_current = env_name == &current_env;
        let marker = if is_current { " (active)" } else { "" };

        if detailed {
            let env_dir = path_manager.env_dir(env_name);
            let runtimes = list_env_runtimes(&env_dir)?;

            println!("  {}{}", env_name, marker);
            println!("    Path: {}", env_dir.display());

            if runtimes.is_empty() {
                println!("    Runtimes: (none)");
            } else {
                println!("    Runtimes:");
                for runtime in runtimes {
                    println!("      - {}", runtime);
                }
            }
            println!();
        } else {
            let prefix = if is_current { "* " } else { "  " };
            println!("{}{}{}", prefix, env_name, marker);
        }
    }

    Ok(())
}

/// List runtimes in an environment
fn list_env_runtimes(env_dir: &PathBuf) -> Result<Vec<String>> {
    let mut runtimes = Vec::new();

    if !env_dir.exists() {
        return Ok(runtimes);
    }

    for entry in std::fs::read_dir(env_dir)? {
        let entry = entry?;
        if let Some(name) = entry.file_name().to_str() {
            runtimes.push(name.to_string());
        }
    }

    runtimes.sort();
    Ok(runtimes)
}

/// Delete an environment
async fn delete_env(name: &str, force: bool) -> Result<()> {
    let path_manager = PathManager::new()?;

    // Prevent deleting the default environment
    if name == "default" {
        anyhow::bail!("Cannot delete the 'default' environment");
    }

    // Check if environment exists
    if !path_manager.env_exists(name) {
        anyhow::bail!("Environment '{}' does not exist", name);
    }

    // Confirm deletion
    if !force {
        UI::warning(&format!(
            "This will delete environment '{}' and all its contents.",
            name
        ));
        print!("Are you sure? [y/N] ");
        use std::io::Write;
        std::io::stdout().flush()?;

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;

        if !input.trim().eq_ignore_ascii_case("y") {
            UI::info("Cancelled");
            return Ok(());
        }
    }

    path_manager.remove_env(name)?;
    UI::success(&format!("Deleted environment '{}'", name));

    Ok(())
}

/// Show environment details
async fn show_env(name: Option<&str>) -> Result<()> {
    let path_manager = PathManager::new()?;
    let env_name = name
        .map(String::from)
        .unwrap_or_else(|| get_default_env().unwrap_or_else(|_| "default".to_string()));

    if !path_manager.env_exists(&env_name) {
        anyhow::bail!("Environment '{}' does not exist", env_name);
    }

    let env_dir = path_manager.env_dir(&env_name);
    let runtimes = list_env_runtimes(&env_dir)?;
    let is_active = get_default_env().map(|e| e == env_name).unwrap_or(false);

    println!("Environment: {}", env_name);
    println!("Path: {}", env_dir.display());
    println!("Active: {}", if is_active { "yes" } else { "no" });
    println!();

    if runtimes.is_empty() {
        println!("Runtimes: (none)");
        UI::hint("Add runtimes with 'vx env add <runtime>@<version>'");
    } else {
        println!("Runtimes:");
        for runtime in runtimes {
            // Try to get version info
            let runtime_path = env_dir.join(&runtime);
            if runtime_path.is_symlink() {
                if let Ok(target) = std::fs::read_link(&runtime_path) {
                    println!("  {} -> {}", runtime, target.display());
                } else {
                    println!("  {}", runtime);
                }
            } else {
                println!("  {}", runtime);
            }
        }
    }

    Ok(())
}

/// Add a runtime to an environment
async fn add_runtime(runtime_version: &str, env_name: Option<&str>) -> Result<()> {
    let path_manager = PathManager::new()?;
    let env = env_name
        .map(String::from)
        .unwrap_or_else(|| get_default_env().unwrap_or_else(|_| "default".to_string()));

    // Parse runtime@version
    let (runtime, version) = parse_runtime_version(runtime_version)?;

    // Check if environment exists
    if !path_manager.env_exists(&env) {
        anyhow::bail!("Environment '{}' does not exist", env);
    }

    // Check if runtime version is installed in store
    if !path_manager.is_version_in_store(&runtime, &version) {
        anyhow::bail!(
            "Runtime '{}@{}' is not installed. Install it first with 'vx install {}@{}'",
            runtime,
            version,
            runtime,
            version
        );
    }

    // Create link from environment to store
    let store_dir = path_manager.version_store_dir(&runtime, &version);
    let env_runtime_path = path_manager.env_runtime_path(&env, &runtime);

    // Remove existing link if present
    if env_runtime_path.exists() || env_runtime_path.is_symlink() {
        std::fs::remove_file(&env_runtime_path)
            .or_else(|_| std::fs::remove_dir_all(&env_runtime_path))
            .context("Failed to remove existing runtime link")?;
    }

    // Create symlink using vx-paths link module
    link::create_link(&store_dir, &env_runtime_path, LinkStrategy::SymLink)
        .context("Failed to create symlink to runtime")?;

    UI::success(&format!(
        "Added {}@{} to environment '{}'",
        runtime, version, env
    ));

    Ok(())
}

/// Remove a runtime from an environment
async fn remove_runtime(runtime: &str, env_name: Option<&str>) -> Result<()> {
    let path_manager = PathManager::new()?;
    let env = env_name
        .map(String::from)
        .unwrap_or_else(|| get_default_env().unwrap_or_else(|_| "default".to_string()));

    // Check if environment exists
    if !path_manager.env_exists(&env) {
        anyhow::bail!("Environment '{}' does not exist", env);
    }

    let env_runtime_path = path_manager.env_runtime_path(&env, runtime);

    if !env_runtime_path.exists() && !env_runtime_path.is_symlink() {
        anyhow::bail!("Runtime '{}' is not in environment '{}'", runtime, env);
    }

    std::fs::remove_file(&env_runtime_path)
        .or_else(|_| std::fs::remove_dir_all(&env_runtime_path))
        .context("Failed to remove runtime from environment")?;

    UI::success(&format!("Removed {} from environment '{}'", runtime, env));

    Ok(())
}

/// Parse runtime@version string
fn parse_runtime_version(s: &str) -> Result<(String, String)> {
    let parts: Vec<&str> = s.splitn(2, '@').collect();
    if parts.len() != 2 {
        anyhow::bail!(
            "Invalid format '{}'. Expected '<runtime>@<version>' (e.g., node@20.0.0)",
            s
        );
    }
    Ok((parts[0].to_string(), parts[1].to_string()))
}
