//! CLI command implementations

use crate::cli::{Cli, Commands};
use crate::ui::UI;
use vx_runtime::ProviderRegistry;

pub mod cleanup;
pub mod config;
pub mod dev;
pub mod env;
pub mod execute;
#[cfg(test)]
mod execute_tests;
pub mod fetch;
pub mod global;
pub mod init;
pub mod install;
pub mod list;
pub mod plugin;
pub mod remove;
pub mod search;
pub mod self_update;
pub mod setup;
pub mod shell;
pub mod stats;
pub mod switch;
pub mod sync;

pub mod update;
pub mod venv_cmd;
pub mod version;
pub mod where_cmd;

// Tests moved to tests/ directory

pub struct CommandHandler;

impl CommandHandler {
    pub async fn handle(
        cli: Cli,
        registry: &ProviderRegistry,
        context: &vx_runtime::RuntimeContext,
    ) -> anyhow::Result<()> {
        // Set verbose mode
        UI::set_verbose(cli.verbose);

        match cli.command {
            Some(Commands::Version) => version::handle().await,

            Some(Commands::List {
                tool,
                status,
                installed: _,
                available: _,
                all,
            }) => list::handle(registry, context, tool.as_deref(), status, all).await,

            Some(Commands::Install {
                tool,
                version,
                force,
            }) => install::handle(registry, context, &tool, version.as_deref(), force).await,

            Some(Commands::Update { tool, apply: _ }) => {
                update::handle(registry, context, tool.as_deref(), false).await
            }

            Some(Commands::SelfUpdate {
                check,
                version: _,
                token,
                prerelease,
                force,
            }) => self_update::handle(token.as_deref(), prerelease, force, check).await,

            Some(Commands::Uninstall {
                tool,
                version,
                force,
            }) => remove::handle(registry, context, &tool, version.as_deref(), force).await,

            Some(Commands::Which { tool, all }) => {
                where_cmd::handle(registry, &tool, all, cli.use_system_path).await
            }

            Some(Commands::Versions {
                tool,
                latest,
                prerelease,
                detailed,
                interactive,
            }) => {
                fetch::handle(
                    registry,
                    context,
                    &tool,
                    latest,
                    detailed,
                    interactive,
                    prerelease,
                )
                .await
            }

            Some(Commands::Switch {
                tool_version,
                global,
            }) => switch::handle(registry, &tool_version, global).await,

            Some(Commands::Config { command }) => match command {
                Some(crate::cli::ConfigCommand::Show) | None => config::handle().await,
                Some(crate::cli::ConfigCommand::Set { key, value }) => {
                    config::handle_set(&key, &value).await
                }
                Some(crate::cli::ConfigCommand::Get { key }) => config::handle_get(&key).await,
                Some(crate::cli::ConfigCommand::Reset { key }) => {
                    config::handle_reset(key.clone()).await
                }
                Some(crate::cli::ConfigCommand::Edit) => config::handle_edit().await,
            },

            Some(Commands::Search {
                query,
                category,
                installed_only,
                available_only,
                format,
                verbose,
            }) => {
                search::handle(
                    registry,
                    query.clone(),
                    category.clone(),
                    installed_only,
                    available_only,
                    format.clone(),
                    verbose,
                )
                .await
            }

            Some(Commands::Sync {
                check,
                force,
                dry_run,
                verbose,
                no_parallel,
                no_auto_install,
            }) => {
                sync::handle(
                    registry,
                    check,
                    force,
                    dry_run,
                    verbose,
                    no_parallel,
                    no_auto_install,
                )
                .await
            }

            Some(Commands::Init {
                interactive,
                template,
                tools,
                force,
                dry_run,
                list_templates,
            }) => {
                init::handle(
                    interactive,
                    template.clone(),
                    tools.clone(),
                    force,
                    dry_run,
                    list_templates,
                )
                .await
            }

            Some(Commands::Clean {
                dry_run,
                cache,
                orphaned,
                all,
                force,
                older_than,
                verbose,
            }) => {
                // Map new clean options to cleanup options
                let cache_only = cache && !all;
                let orphaned_only = orphaned && !all;
                cleanup::handle(
                    dry_run,
                    cache_only,
                    orphaned_only,
                    force,
                    older_than,
                    verbose,
                )
                .await
            }

            Some(Commands::Stats) => stats::handle(registry).await,

            Some(Commands::Plugin { command }) => plugin::handle(registry, command).await,

            Some(Commands::Venv { command }) => venv_cmd::handle(command).await,

            Some(Commands::Global { command }) => global::handle(command).await,

            Some(Commands::Env { command }) => env::handle(command).await,

            Some(Commands::Dev {
                shell,
                command,
                no_install,
                verbose,
                export,
                format,
            }) => dev::handle(shell, command, no_install, verbose, export, format).await,

            Some(Commands::Setup {
                force,
                dry_run,
                verbose,
                no_parallel,
            }) => setup::handle(registry, force, dry_run, verbose, no_parallel).await,

            Some(Commands::Add { tool, version }) => {
                setup::add_tool(&tool, version.as_deref()).await
            }

            Some(Commands::RemoveTool { tool }) => setup::remove_tool(&tool).await,

            Some(Commands::Run { script, args }) => run_script(&script, &args).await,

            None => {
                // Handle tool execution
                if cli.args.is_empty() {
                    UI::error("No tool specified");
                    UI::hint("Usage: vx <tool> [args...]");
                    UI::hint("Example: vx uv pip install requests");
                    UI::hint("Run 'vx list --all' to see supported tools");
                    std::process::exit(1);
                }

                let tool_name = &cli.args[0];
                let tool_args = &cli.args[1..];

                // Use the executor to run the tool
                let exit_code = execute::execute_tool(
                    registry,
                    context,
                    tool_name,
                    tool_args,
                    cli.use_system_path,
                )
                .await?;
                if exit_code != 0 {
                    std::process::exit(exit_code);
                }
                Ok(())
            }

            Some(Commands::Shell { command }) => {
                use crate::cli::ShellCommand;
                match command {
                    ShellCommand::Init { shell } => shell::handle_shell_init(shell.clone()).await,
                    ShellCommand::Completions { shell } => {
                        shell::handle_completion(shell.clone()).await
                    }
                }
            }
        }
    }
}

/// Run a script defined in .vx.toml
///
/// This function generates a platform-specific wrapper script that:
/// 1. Sets up the environment variables (including PATH with vx-managed tools)
/// 2. Executes the user's command
///
/// This approach (inspired by rez) ensures environment variables are properly
/// set in the shell context, avoiding issues with subprocess environment inheritance.
async fn run_script(script_name: &str, args: &[String]) -> anyhow::Result<()> {
    use std::env;

    let current_dir = env::current_dir()?;
    let config_path = current_dir.join(".vx.toml");

    if !config_path.exists() {
        return Err(anyhow::anyhow!("No .vx.toml found. Run 'vx init' first."));
    }

    let config = setup::parse_vx_config(&config_path)?;

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

    // Build environment with vx-managed tools in PATH
    let env_vars = dev::build_script_environment(&config)?;

    // Append additional args to the command
    let full_cmd = if args.is_empty() {
        script_cmd.clone()
    } else {
        format!("{} {}", script_cmd, args.join(" "))
    };

    // Generate and execute a platform-specific wrapper script
    // This ensures environment variables are properly set in the shell context
    let status = execute_with_env_script(&full_cmd, &env_vars)?;

    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }

    Ok(())
}

/// Execute a command by generating a platform-specific wrapper script
///
/// This approach (inspired by rez's shell execution model) generates a temporary
/// script that sets up the environment and then executes the command. This ensures
/// that environment variables like PATH are properly available to the command and
/// any subprocesses it spawns.
fn execute_with_env_script(
    cmd: &str,
    env_vars: &std::collections::HashMap<String, String>,
) -> anyhow::Result<std::process::ExitStatus> {
    use std::fs;
    use std::io::Write;
    use std::process::Command;

    // Create a temporary directory for the script
    let temp_dir = std::env::temp_dir();
    let script_id = std::process::id();
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);

    #[cfg(windows)]
    let script_path = temp_dir.join(format!("vx_run_{}_{}.bat", script_id, timestamp));

    #[cfg(not(windows))]
    let script_path = temp_dir.join(format!("vx_run_{}_{}.sh", script_id, timestamp));

    // Generate the script content
    let script_content = generate_wrapper_script(cmd, env_vars);

    // Write the script
    {
        let mut file = fs::File::create(&script_path)?;
        file.write_all(script_content.as_bytes())?;
    }

    // Make executable on Unix
    #[cfg(not(windows))]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&script_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&script_path, perms)?;
    }

    // Execute the script
    #[cfg(windows)]
    let status = Command::new("cmd")
        .args(["/C", script_path.to_str().unwrap()])
        .status();

    #[cfg(not(windows))]
    let status = Command::new("sh").arg(&script_path).status();

    // Clean up the temporary script
    let _ = fs::remove_file(&script_path);

    status.map_err(|e| anyhow::anyhow!("Failed to execute script: {}", e))
}

/// Generate a platform-specific wrapper script that sets environment variables
/// and executes the command
fn generate_wrapper_script(
    cmd: &str,
    env_vars: &std::collections::HashMap<String, String>,
) -> String {
    let mut script = String::new();

    #[cfg(windows)]
    {
        // Windows batch script
        script.push_str("@echo off\r\n");

        // Set environment variables
        for (key, value) in env_vars {
            // Escape special characters for batch
            let escaped_value = value.replace('%', "%%");
            script.push_str(&format!("SET \"{}={}\"\r\n", key, escaped_value));
        }

        // Execute the command
        script.push_str(&format!("{}\r\n", cmd));
    }

    #[cfg(not(windows))]
    {
        // Unix shell script
        script.push_str("#!/bin/sh\n");

        // Set environment variables
        for (key, value) in env_vars {
            // Escape special characters for shell
            let escaped_value = value.replace('\\', "\\\\").replace('"', "\\\"");
            script.push_str(&format!("export {}=\"{}\"\n", key, escaped_value));
        }

        // Execute the command
        script.push_str(&format!("{}\n", cmd));
    }

    script
}
