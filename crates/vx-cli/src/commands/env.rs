//! Environment management commands
//!
//! This module provides commands for managing vx environments:
//! - create: Create a new environment (project-local or global)
//! - use: Activate an environment
//! - list: List all environments
//! - delete: Remove an environment
//! - show: Show current environment details
//! - shell: Enter an interactive shell with environment tools
//!
//! ## Environment Types
//!
//! - **Project Environment**: Created in `.vx/env/` under the project directory (default when vx.toml exists)
//! - **Global Environment**: Created in `~/.vx/envs/` for cross-project use
//!
//! ## Storage Model
//!
//! All tools are stored globally in `~/.vx/store/` (content-addressable).
//! Environments contain symlinks to the global store, saving disk space.

use crate::commands::setup::parse_vx_config;
use crate::ui::UI;
use anyhow::{Context, Result};
use clap::Subcommand;
use std::collections::HashMap;
use std::env;
use std::path::{Path, PathBuf};
use vx_env::{ExportFormat, SessionContext, ShellSpawner};
use vx_paths::{
    find_config_file, link, project_env_dir, LinkStrategy, PathManager, PROJECT_ENV_DIR,
};

/// Environment subcommands
#[derive(Subcommand, Clone)]
pub enum EnvCommand {
    /// Create a new environment
    ///
    /// By default, creates a project-local environment in `.vx/env/` if `vx.toml` exists.
    /// Use `--global` to create a named global environment in `~/.vx/envs/`.
    Create {
        /// Environment name (optional for project environments)
        name: Option<String>,
        /// Create a global environment instead of project-local
        #[arg(long, short)]
        global: bool,
        /// Clone from an existing environment
        #[arg(long)]
        from: Option<String>,
        /// Set as default environment after creation
        #[arg(long)]
        set_default: bool,
    },

    /// Activate an environment
    Use {
        /// Environment name (optional, uses project env if available)
        name: Option<String>,
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
        /// Show only global environments
        #[arg(long)]
        global: bool,
    },

    /// Delete an environment
    #[command(alias = "rm")]
    Delete {
        /// Environment name (optional for project environment)
        name: Option<String>,
        /// Force deletion without confirmation
        #[arg(long)]
        force: bool,
        /// Delete global environment
        #[arg(long, short)]
        global: bool,
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
        /// Target global environment name
        #[arg(long)]
        env: Option<String>,
        /// Add to global environment instead of project
        #[arg(long, short)]
        global: bool,
    },

    /// Remove a runtime from an environment
    Remove {
        /// Runtime name
        runtime: String,
        /// Target global environment name
        #[arg(long)]
        env: Option<String>,
        /// Remove from global environment
        #[arg(long, short)]
        global: bool,
    },

    /// Sync project environment from vx.toml
    ///
    /// Creates symlinks in `.vx/env/` for all tools defined in `vx.toml`
    Sync,

    /// Enter an environment shell
    ///
    /// Spawns an interactive shell with the environment's tools available in PATH.
    /// Similar to `vx dev` but uses the environment directory instead of vx.toml.
    Shell {
        /// Environment name (defaults to project env or global default)
        name: Option<String>,
        /// Use global environment
        #[arg(long, short)]
        global: bool,
        /// Shell to use (defaults to auto-detect)
        #[arg(long)]
        shell: Option<String>,
        /// Command to execute instead of interactive shell
        #[arg(last = true)]
        command: Option<Vec<String>>,
        /// Export environment variables instead of spawning shell
        #[arg(long)]
        export: bool,
        /// Export format (shell, powershell, batch, github)
        #[arg(long)]
        format: Option<String>,
    },
}

/// Handle environment commands
pub async fn handle(command: EnvCommand) -> Result<()> {
    match command {
        EnvCommand::Create {
            name,
            global,
            from,
            set_default,
        } => create_env(name.as_deref(), global, from.as_deref(), set_default).await,
        EnvCommand::Use { name, global } => use_env(name.as_deref(), global).await,
        EnvCommand::List { detailed, global } => list_envs(detailed, global).await,
        EnvCommand::Delete {
            name,
            force,
            global,
        } => delete_env(name.as_deref(), force, global).await,
        EnvCommand::Show { name } => show_env(name.as_deref()).await,
        EnvCommand::Add {
            runtime_version,
            env,
            global,
        } => add_runtime(&runtime_version, env.as_deref(), global).await,
        EnvCommand::Remove {
            runtime,
            env,
            global,
        } => remove_runtime(&runtime, env.as_deref(), global).await,
        EnvCommand::Sync => sync_env().await,
        EnvCommand::Shell {
            name,
            global,
            shell,
            command,
            export,
            format,
        } => env_shell(name.as_deref(), global, shell, command, export, format).await,
    }
}

// ========== Helper Functions ==========

/// Get the project environment directory if in a project with vx.toml
fn get_project_env_dir() -> Option<PathBuf> {
    let current_dir = env::current_dir().ok()?;

    if find_config_file(&current_dir).is_some() {
        Some(project_env_dir(&current_dir))
    } else {
        None
    }
}

/// Set the default global environment
fn set_default_env(name: &str) -> Result<()> {
    let path_manager = PathManager::new()?;
    let config_dir = path_manager.config_dir();
    let default_file = config_dir.join("default-env");

    std::fs::create_dir_all(config_dir)?;
    std::fs::write(&default_file, name)?;

    Ok(())
}

/// Get the current default global environment
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

/// List runtimes in an environment directory
fn list_env_runtimes(env_dir: &Path) -> Result<Vec<String>> {
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

/// Clone environment contents (symlinks)
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

// ========== Command Implementations ==========

/// Create a new environment
async fn create_env(
    name: Option<&str>,
    global: bool,
    from: Option<&str>,
    set_default: bool,
) -> Result<()> {
    let path_manager = PathManager::new()?;

    if global {
        // Create global environment
        let env_name = name.ok_or_else(|| {
            anyhow::anyhow!("Environment name is required for global environments")
        })?;

        if path_manager.env_exists(env_name) {
            anyhow::bail!("Global environment '{}' already exists", env_name);
        }

        let env_dir = path_manager.create_env(env_name)?;
        UI::success(&format!(
            "Created global environment '{}' at {}",
            env_name,
            env_dir.display()
        ));

        if let Some(source) = from {
            if !path_manager.env_exists(source) {
                anyhow::bail!("Source environment '{}' does not exist", source);
            }
            let source_dir = path_manager.env_dir(source);
            clone_env_contents(&source_dir, &env_dir)?;
            UI::info(&format!("Cloned from environment '{}'", source));
        }

        if set_default {
            set_default_env(env_name)?;
            UI::info(&format!("Set '{}' as default global environment", env_name));
        }
    } else {
        // Create project environment
        let current_dir = env::current_dir().context("Failed to get current directory")?;

        if find_config_file(&current_dir).is_none() {
            anyhow::bail!(
                "No vx.toml found. Create one with 'vx init' or use '--global' for a global environment"
            );
        }

        let env_dir = current_dir.join(PROJECT_ENV_DIR);

        if env_dir.exists() {
            anyhow::bail!(
                "Project environment already exists at {}",
                env_dir.display()
            );
        }

        std::fs::create_dir_all(&env_dir)?;
        UI::success(&format!(
            "Created project environment at {}",
            env_dir.display()
        ));

        // If from is specified, clone from a global environment
        if let Some(source) = from {
            if !path_manager.env_exists(source) {
                anyhow::bail!("Source global environment '{}' does not exist", source);
            }
            let source_dir = path_manager.env_dir(source);
            clone_env_contents(&source_dir, &env_dir)?;
            UI::info(&format!("Cloned from global environment '{}'", source));
        }

        UI::hint("Run 'vx env sync' to populate from vx.toml, or 'vx env add <tool>@<version>' to add tools");
    }

    Ok(())
}

/// Activate an environment
async fn use_env(name: Option<&str>, global: bool) -> Result<()> {
    let path_manager = PathManager::new()?;

    let (env_dir, env_display_name) = if global {
        let env_name =
            name.ok_or_else(|| anyhow::anyhow!("Environment name is required with --global"))?;

        if !path_manager.env_exists(env_name) {
            anyhow::bail!(
                "Global environment '{}' does not exist. Create it with 'vx env create --global {}'",
                env_name,
                env_name
            );
        }

        (
            path_manager.env_dir(env_name),
            format!("global:{}", env_name),
        )
    } else if let Some(env_name) = name {
        // Check global environment
        if !path_manager.env_exists(env_name) {
            anyhow::bail!("Environment '{}' does not exist", env_name);
        }
        (path_manager.env_dir(env_name), env_name.to_string())
    } else {
        // Use project environment if available
        if let Some(project_env) = get_project_env_dir() {
            if project_env.exists() {
                (project_env, "project".to_string())
            } else {
                anyhow::bail!("No project environment found. Create one with 'vx env create'");
            }
        } else {
            anyhow::bail!("No vx.toml found. Use 'vx env use <name>' for a global environment");
        }
    };

    UI::info(&format!(
        "To activate environment '{}' in current shell:",
        env_display_name
    ));

    #[cfg(windows)]
    {
        println!("  $env:VX_ENV = \"{}\"", env_display_name);
        println!("  $env:PATH = \"{};$env:PATH\"", env_dir.display());
    }

    #[cfg(not(windows))]
    {
        println!("  export VX_ENV=\"{}\"", env_display_name);
        println!("  export PATH=\"{}:$PATH\"", env_dir.display());
    }

    println!();
    UI::hint("Or use 'eval \"$(vx dev --export)\"' if you have vx.toml");

    Ok(())
}

/// List all environments
async fn list_envs(detailed: bool, global_only: bool) -> Result<()> {
    let path_manager = PathManager::new()?;
    let current_env = get_default_env().unwrap_or_else(|_| "default".to_string());

    // Show project environment first (unless global_only)
    if !global_only {
        if let Some(project_env) = get_project_env_dir() {
            if project_env.exists() {
                println!("Project Environment:");
                println!();

                if detailed {
                    let runtimes = list_env_runtimes(&project_env)?;
                    println!("  project (active)");
                    println!("    Path: {}", project_env.display());

                    if runtimes.is_empty() {
                        println!("    Tools: (none)");
                    } else {
                        println!("    Tools:");
                        for runtime in runtimes {
                            let runtime_path = project_env.join(&runtime);
                            if runtime_path.is_symlink() {
                                if let Ok(target) = std::fs::read_link(&runtime_path) {
                                    println!("      - {} -> {}", runtime, target.display());
                                } else {
                                    println!("      - {}", runtime);
                                }
                            } else {
                                println!("      - {}", runtime);
                            }
                        }
                    }
                } else {
                    println!("* project (active)");
                }
                println!();
            }
        }
    }

    // Show global environments
    let envs = path_manager.list_envs()?;

    if envs.is_empty() && global_only {
        UI::info("No global environments found. Create one with 'vx env create --global <name>'");
        return Ok(());
    }

    if !envs.is_empty() {
        println!("Global Environments:");
        println!();

        for env_name in &envs {
            let is_default = env_name == &current_env;
            let marker = if is_default { " (default)" } else { "" };

            if detailed {
                let env_dir = path_manager.env_dir(env_name);
                let runtimes = list_env_runtimes(&env_dir)?;

                println!("  {}{}", env_name, marker);
                println!("    Path: {}", env_dir.display());

                if runtimes.is_empty() {
                    println!("    Tools: (none)");
                } else {
                    println!("    Tools:");
                    for runtime in runtimes {
                        let runtime_path = env_dir.join(&runtime);
                        if runtime_path.is_symlink() {
                            if let Ok(target) = std::fs::read_link(&runtime_path) {
                                println!("      - {} -> {}", runtime, target.display());
                            } else {
                                println!("      - {}", runtime);
                            }
                        } else {
                            println!("      - {}", runtime);
                        }
                    }
                }
                println!();
            } else {
                let prefix = if is_default { "* " } else { "  " };
                println!("{}{}{}", prefix, env_name, marker);
            }
        }
    }

    Ok(())
}

/// Delete an environment
async fn delete_env(name: Option<&str>, force: bool, global: bool) -> Result<()> {
    let path_manager = PathManager::new()?;

    let (env_dir, env_display) = if global {
        let env_name =
            name.ok_or_else(|| anyhow::anyhow!("Environment name is required with --global"))?;

        if env_name == "default" {
            anyhow::bail!("Cannot delete the 'default' global environment");
        }

        if !path_manager.env_exists(env_name) {
            anyhow::bail!("Global environment '{}' does not exist", env_name);
        }

        (
            path_manager.env_dir(env_name),
            format!("global:{}", env_name),
        )
    } else {
        // Delete project environment
        let project_env = get_project_env_dir().ok_or_else(|| {
            anyhow::anyhow!(
                "No vx.toml found. Use '--global <name>' to delete a global environment"
            )
        })?;

        if !project_env.exists() {
            anyhow::bail!("Project environment does not exist");
        }

        (project_env, "project".to_string())
    };

    // Confirm deletion
    if !force {
        UI::warning(&format!(
            "This will delete environment '{}' and all its contents.",
            env_display
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

    if global {
        let env_name = name.unwrap();
        path_manager.remove_env(env_name)?;
    } else {
        std::fs::remove_dir_all(&env_dir)?;
    }

    UI::success(&format!("Deleted environment '{}'", env_display));

    Ok(())
}

/// Show environment details
async fn show_env(name: Option<&str>) -> Result<()> {
    let path_manager = PathManager::new()?;

    let (env_dir, env_name, env_type) = if let Some(n) = name {
        // Show specific global environment
        if !path_manager.env_exists(n) {
            anyhow::bail!("Global environment '{}' does not exist", n);
        }
        (path_manager.env_dir(n), n.to_string(), "global")
    } else {
        // Show project environment if available, otherwise default global
        if let Some(project_env) = get_project_env_dir() {
            if project_env.exists() {
                (project_env, "project".to_string(), "project")
            } else {
                let default_env = get_default_env()?;
                (path_manager.env_dir(&default_env), default_env, "global")
            }
        } else {
            let default_env = get_default_env()?;
            (path_manager.env_dir(&default_env), default_env, "global")
        }
    };

    let runtimes = list_env_runtimes(&env_dir)?;

    println!("Environment: {}", env_name);
    println!("Type: {}", env_type);
    println!("Path: {}", env_dir.display());
    println!();

    if runtimes.is_empty() {
        println!("Tools: (none)");
        if env_type == "project" {
            UI::hint("Run 'vx env sync' to populate from vx.toml");
        } else {
            UI::hint("Add tools with 'vx env add <tool>@<version>'");
        }
    } else {
        println!("Tools:");
        for runtime in runtimes {
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
async fn add_runtime(runtime_version: &str, env_name: Option<&str>, global: bool) -> Result<()> {
    let path_manager = PathManager::new()?;
    let (runtime, version) = parse_runtime_version(runtime_version)?;

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

    let (env_dir, env_display) = if global {
        let name = env_name
            .ok_or_else(|| anyhow::anyhow!("Environment name is required with --global"))?;

        if !path_manager.env_exists(name) {
            anyhow::bail!("Global environment '{}' does not exist", name);
        }

        (path_manager.env_dir(name), format!("global:{}", name))
    } else if let Some(name) = env_name {
        // Add to specified global environment
        if !path_manager.env_exists(name) {
            anyhow::bail!("Global environment '{}' does not exist", name);
        }
        (path_manager.env_dir(name), name.to_string())
    } else {
        // Add to project environment (create if needed)
        let project_env = get_project_env_dir().ok_or_else(|| {
            anyhow::anyhow!(
                "No vx.toml found. Use '--global' or '--env <name>' for global environments"
            )
        })?;

        if !project_env.exists() {
            std::fs::create_dir_all(&project_env)?;
            UI::info(&format!(
                "Created project environment at {}",
                project_env.display()
            ));
        }

        (project_env, "project".to_string())
    };

    // Create link from environment to store
    let store_dir = path_manager.version_store_dir(&runtime, &version);
    let env_runtime_path = env_dir.join(&runtime);

    // Remove existing link if present
    if env_runtime_path.exists() || env_runtime_path.is_symlink() {
        std::fs::remove_file(&env_runtime_path)
            .or_else(|_| std::fs::remove_dir_all(&env_runtime_path))
            .context("Failed to remove existing runtime link")?;
    }

    // Create symlink
    link::create_link(&store_dir, &env_runtime_path, LinkStrategy::SymLink)
        .context("Failed to create symlink to runtime")?;

    UI::success(&format!(
        "Added {}@{} to environment '{}'",
        runtime, version, env_display
    ));

    Ok(())
}

/// Remove a runtime from an environment
async fn remove_runtime(runtime: &str, env_name: Option<&str>, global: bool) -> Result<()> {
    let path_manager = PathManager::new()?;

    let (env_dir, env_display) = if global {
        let name = env_name
            .ok_or_else(|| anyhow::anyhow!("Environment name is required with --global"))?;

        if !path_manager.env_exists(name) {
            anyhow::bail!("Global environment '{}' does not exist", name);
        }

        (path_manager.env_dir(name), format!("global:{}", name))
    } else if let Some(name) = env_name {
        if !path_manager.env_exists(name) {
            anyhow::bail!("Global environment '{}' does not exist", name);
        }
        (path_manager.env_dir(name), name.to_string())
    } else {
        let project_env = get_project_env_dir().ok_or_else(|| {
            anyhow::anyhow!(
                "No vx.toml found. Use '--global' or '--env <name>' for global environments"
            )
        })?;

        if !project_env.exists() {
            anyhow::bail!("Project environment does not exist");
        }

        (project_env, "project".to_string())
    };

    let env_runtime_path = env_dir.join(runtime);

    if !env_runtime_path.exists() && !env_runtime_path.is_symlink() {
        anyhow::bail!(
            "Runtime '{}' is not in environment '{}'",
            runtime,
            env_display
        );
    }

    std::fs::remove_file(&env_runtime_path)
        .or_else(|_| std::fs::remove_dir_all(&env_runtime_path))
        .context("Failed to remove runtime from environment")?;

    UI::success(&format!(
        "Removed {} from environment '{}'",
        runtime, env_display
    ));

    Ok(())
}

/// Sync project environment from vx.toml
async fn sync_env() -> Result<()> {
    let current_dir = env::current_dir().context("Failed to get current directory")?;
    let config_path = find_config_file(&current_dir)
        .ok_or_else(|| anyhow::anyhow!("No vx.toml found in current directory"))?;

    let config = parse_vx_config(&config_path)?;
    let path_manager = PathManager::new()?;
    let env_dir = current_dir.join(PROJECT_ENV_DIR);

    // Create project environment directory if needed
    if !env_dir.exists() {
        std::fs::create_dir_all(&env_dir)?;
        UI::info(&format!(
            "Created project environment at {}",
            env_dir.display()
        ));
    }

    let mut synced = 0;
    let mut missing = Vec::new();

    for (tool_name, version) in &config.tools {
        let store_dir = path_manager.version_store_dir(tool_name, version);

        if !store_dir.exists() {
            missing.push(format!("{}@{}", tool_name, version));
            continue;
        }

        let env_tool_path = env_dir.join(tool_name);

        // Remove existing link if present
        if env_tool_path.exists() || env_tool_path.is_symlink() {
            std::fs::remove_file(&env_tool_path)
                .or_else(|_| std::fs::remove_dir_all(&env_tool_path))
                .ok();
        }

        // Create symlink
        link::create_link(&store_dir, &env_tool_path, LinkStrategy::SymLink)
            .with_context(|| format!("Failed to create symlink for {}", tool_name))?;

        synced += 1;
    }

    if synced > 0 {
        UI::success(&format!("Synced {} tool(s) to project environment", synced));
    }

    if !missing.is_empty() {
        UI::warning(&format!(
            "The following tools are not installed: {}",
            missing.join(", ")
        ));
        UI::hint("Run 'vx setup' to install missing tools");
    }

    if synced == 0 && missing.is_empty() {
        UI::info("No tools defined in vx.toml");
    }

    Ok(())
}

/// Enter an environment shell
///
/// Spawns an interactive shell with the environment's tools available in PATH.
async fn env_shell(
    name: Option<&str>,
    global: bool,
    shell: Option<String>,
    command: Option<Vec<String>>,
    export: bool,
    format: Option<String>,
) -> Result<()> {
    let path_manager = PathManager::new()?;

    // Resolve the environment directory and name
    let (env_dir, env_name) = resolve_env_for_shell(name, global, &path_manager)?;

    // Build tools map from environment symlinks
    let tools = build_tools_from_env_dir(&env_dir, &path_manager)?;

    if tools.is_empty() {
        UI::warning(&format!(
            "Environment '{}' has no tools. Add tools with 'vx env add <tool>@<version>'",
            env_name
        ));
        return Ok(());
    }

    // Create SessionContext from environment
    let mut session = SessionContext::new(&env_name).tools(&tools);

    // If we're in a project directory, set the project root
    if let Ok(current_dir) = env::current_dir() {
        session = session.project_root(current_dir);
    }

    // Create ShellSpawner
    let spawner = ShellSpawner::new(session)?;

    // Handle --export mode
    if export {
        let export_format = match format {
            Some(f) => ExportFormat::parse(&f).ok_or_else(|| {
                anyhow::anyhow!(
                    "Unknown format: {}. Use: shell, powershell, batch, or github",
                    f
                )
            })?,
            None => ExportFormat::detect(),
        };

        let output = spawner.export(export_format)?;
        print!("{}", output);
        return Ok(());
    }

    // Handle command execution or interactive shell
    if let Some(cmd) = command {
        if cmd.is_empty() {
            anyhow::bail!("No command specified");
        }
        let status = spawner.execute_command(&cmd)?;
        if !status.success() {
            std::process::exit(status.code().unwrap_or(1));
        }
    } else {
        UI::success(&format!("Entering environment '{}'", env_name));
        UI::info(&format!(
            "Tools: {}",
            tools.keys().cloned().collect::<Vec<_>>().join(", ")
        ));
        UI::hint("Type 'exit' to leave the environment");
        println!();

        let status = spawner.spawn_interactive(shell.as_deref())?;

        if !status.success() {
            std::process::exit(status.code().unwrap_or(1));
        }

        UI::info(&format!("Left environment '{}'", env_name));
    }

    Ok(())
}

/// Resolve environment directory for shell command
fn resolve_env_for_shell(
    name: Option<&str>,
    global: bool,
    path_manager: &PathManager,
) -> Result<(PathBuf, String)> {
    if global {
        let env_name =
            name.ok_or_else(|| anyhow::anyhow!("Environment name is required with --global"))?;

        if !path_manager.env_exists(env_name) {
            anyhow::bail!(
                "Global environment '{}' does not exist. Create it with 'vx env create --global {}'",
                env_name,
                env_name
            );
        }

        Ok((path_manager.env_dir(env_name), env_name.to_string()))
    } else if let Some(env_name) = name {
        // Check global environment by name
        if path_manager.env_exists(env_name) {
            Ok((path_manager.env_dir(env_name), env_name.to_string()))
        } else {
            anyhow::bail!(
                "Environment '{}' does not exist. Create it with 'vx env create --global {}'",
                env_name,
                env_name
            );
        }
    } else {
        // Use project environment if available
        if let Some(project_env) = get_project_env_dir() {
            if project_env.exists() {
                return Ok((project_env, "project".to_string()));
            }
        }

        // Fall back to default global environment
        let default_env = get_default_env()?;
        let env_dir = path_manager.env_dir(&default_env);

        if !env_dir.exists() {
            anyhow::bail!(
                "No environment found. Create one with 'vx env create' or 'vx env create --global <name>'"
            );
        }

        Ok((env_dir, default_env))
    }
}

/// Build tools map from environment directory symlinks
fn build_tools_from_env_dir(
    env_dir: &Path,
    _path_manager: &PathManager,
) -> Result<HashMap<String, String>> {
    let mut tools = HashMap::new();

    if !env_dir.exists() {
        return Ok(tools);
    }

    for entry in std::fs::read_dir(env_dir)? {
        let entry = entry?;
        let path = entry.path();

        if !path.is_symlink() && !path.is_dir() {
            continue;
        }

        let tool_name = entry.file_name().to_string_lossy().to_string();

        // Try to extract version from symlink target or directory name
        let version = if path.is_symlink() {
            if let Ok(target) = std::fs::read_link(&path) {
                // Extract version from store path: ~/.vx/store/<tool>/<version>/...
                extract_version_from_store_path(&target, &tool_name)
            } else {
                "latest".to_string()
            }
        } else {
            // For directories, try to detect version
            "latest".to_string()
        };

        tools.insert(tool_name, version);
    }

    Ok(tools)
}

/// Extract version from store path
fn extract_version_from_store_path(store_path: &Path, tool_name: &str) -> String {
    // Store path format: ~/.vx/store/<tool>/<version>/...
    let path_str = store_path.to_string_lossy();

    if let Some(store_idx) = path_str.find("store") {
        let after_store = &path_str[store_idx + 6..]; // Skip "store/"
        let parts: Vec<&str> = after_store.split(['/', '\\']).collect();

        // Expected: [<tool>, <version>, ...]
        if parts.len() >= 2 && parts[0] == tool_name {
            return parts[1].to_string();
        }
    }

    "latest".to_string()
}
