//! Environment command handler

use super::Args;
use super::args::EnvCommand;
use super::helpers::{
    build_tools_from_env_dir, clone_env_contents, get_default_env, get_project_env_dir,
    list_env_runtimes, parse_runtime_version, resolve_env_for_shell, set_default_env,
};
use crate::commands::common::load_config_view_cwd;
use crate::commands::setup::find_vx_config as find_config_file;
use crate::ui::UI;
use anyhow::{Context, Result};
use std::env;
use std::io::Write;
use vx_env::{ExportFormat, SessionContext, SessionSource, ShellSpawner};
use vx_paths::{LinkStrategy, PROJECT_ENV_DIR, PathManager, link};

/// Handle env command with Args
pub async fn handle(args: &Args) -> Result<()> {
    match &args.command {
        EnvCommand::Create {
            name,
            global,
            from,
            set_default,
        } => create_env(name.as_deref(), *global, from.as_deref(), *set_default).await,
        EnvCommand::Use { name, global } => use_env(name.as_deref(), *global).await,
        EnvCommand::List { detailed, global } => list_envs(*detailed, *global).await,
        EnvCommand::Delete {
            name,
            force,
            global,
        } => delete_env(name.as_deref(), *force, *global).await,
        EnvCommand::Show { name } => show_env(name.as_deref()).await,
        EnvCommand::Add {
            runtime_version,
            env,
            global,
        } => add_runtime(runtime_version, env.as_deref(), *global).await,
        EnvCommand::Remove {
            runtime,
            env,
            global,
        } => remove_runtime(runtime, env.as_deref(), *global).await,
        EnvCommand::Sync => sync_env().await,
        EnvCommand::Shell {
            name,
            global,
            shell,
            command,
            export,
            format,
        } => {
            env_shell(
                name.as_deref(),
                *global,
                shell.clone(),
                command.clone(),
                *export,
                format.clone(),
            )
            .await
        }
    }
}

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

        if find_config_file(&current_dir).is_err() {
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

        UI::hint(
            "Run 'vx env sync' to populate from vx.toml, or 'vx env add <tool>@<version>' to add tools",
        );
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
    let (config_path, config) = load_config_view_cwd()?;
    let current_dir = config_path.parent().unwrap();
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
    let mut session = SessionContext::new(&env_name)
        .tools(&tools)
        .source(SessionSource::EnvDir {
            path: env_dir.clone(),
            name: env_name.clone(),
        });

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
