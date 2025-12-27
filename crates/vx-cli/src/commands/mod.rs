//! CLI command implementations

use crate::cli::{Cli, Commands};
use crate::ui::UI;
use vx_runtime::ProviderRegistry;

pub mod cleanup;
pub mod config;
pub mod container;
pub mod dev;
pub mod env;
pub mod execute;
#[cfg(test)]
mod execute_tests;
pub mod fetch;
pub mod global;
pub mod hook;
pub mod init;
pub mod install;
pub mod list;
pub mod plugin;
pub mod remove;
pub mod search;
pub mod self_update;
pub mod services;
pub mod setup;
pub mod shell;
pub mod stats;
pub mod switch;
pub mod sync;

pub mod update;
pub mod venv_cmd;
pub mod version;
pub mod where_cmd;

// Re-export script generation functions for testing
pub use self::script_generator::{execute_with_env_script, generate_wrapper_script};

// Script generator module
mod script_generator;

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
                Some(crate::cli::ConfigCommand::Migrate {
                    path,
                    dry_run,
                    backup,
                    force,
                }) => config::handle_migrate(path, dry_run, backup, force).await,
                Some(crate::cli::ConfigCommand::Validate { path, verbose }) => {
                    config::handle_validate(path, verbose).await
                }
                Some(crate::cli::ConfigCommand::Schema { output }) => {
                    config::handle_schema(output).await
                }
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
                no_hooks,
            }) => setup::handle(registry, force, dry_run, verbose, no_parallel, no_hooks).await,

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

            Some(Commands::Services { command }) => {
                use crate::cli::ServicesCommand;
                match command {
                    ServicesCommand::Start {
                        services,
                        foreground,
                        force,
                        verbose,
                    } => {
                        let services = if services.is_empty() {
                            None
                        } else {
                            Some(services)
                        };
                        services::handle_start(services, !foreground, force, verbose).await
                    }
                    ServicesCommand::Stop { services, verbose } => {
                        let services = if services.is_empty() {
                            None
                        } else {
                            Some(services)
                        };
                        services::handle_stop(services, verbose).await
                    }
                    ServicesCommand::Status { verbose } => services::handle_status(verbose).await,
                    ServicesCommand::Logs {
                        service,
                        follow,
                        tail,
                    } => services::handle_logs(&service, follow, tail).await,
                    ServicesCommand::Restart { services, verbose } => {
                        let services = if services.is_empty() {
                            None
                        } else {
                            Some(services)
                        };
                        services::handle_restart(services, verbose).await
                    }
                }
            }

            Some(Commands::Hook { command }) => {
                use crate::cli::HookCommand;
                match command {
                    HookCommand::PreCommit => hook::handle_pre_commit().await,
                    HookCommand::Enter => hook::handle_enter().await,
                    HookCommand::Install { force } => hook::handle_install(force).await,
                    HookCommand::Uninstall => hook::handle_uninstall().await,
                    HookCommand::Status => hook::handle_status().await,
                    HookCommand::Run { name } => hook::handle_run(&name).await,
                    HookCommand::ShellInit { shell } => hook::handle_shell_init(shell).await,
                }
            }

            Some(Commands::Container { command }) => {
                use crate::cli::ContainerCommand;
                match command {
                    ContainerCommand::Generate {
                        output,
                        with_ignore,
                        dry_run,
                        template,
                    } => container::handle_generate(output, with_ignore, dry_run, template).await,
                    ContainerCommand::Build {
                        tag,
                        target,
                        build_arg,
                        platform,
                        no_cache,
                        push,
                        verbose,
                    } => {
                        container::handle_build(
                            tag, target, build_arg, platform, no_cache, push, verbose,
                        )
                        .await
                    }
                    ContainerCommand::Push { tag, verbose } => {
                        container::handle_push(tag, verbose).await
                    }
                    ContainerCommand::Status => container::handle_status().await,
                    ContainerCommand::Login {
                        registry,
                        username,
                        password,
                    } => container::handle_login(registry, username, password).await,
                    ContainerCommand::Tags { all } => container::handle_tags(all).await,
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
    let status = script_generator::execute_with_env_script(&full_cmd, &env_vars)?;

    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }

    Ok(())
}
