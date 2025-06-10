use clap::Parser;
use vx::cli::{Cli, Commands, PluginCommand};
use vx::config::{Config, ToolConfig};
use vx::executor::Executor;
use vx::plugin::PluginCategory;
use vx::plugin_manager::PluginManager;
use vx::ui::UI;
use vx::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Set verbose mode
    UI::set_verbose(cli.verbose);

    match cli.command {
        Some(Commands::Version) => {
            UI::header(&format!("vx {}", env!("CARGO_PKG_VERSION")));
            UI::info("Universal version executor for development tools");
        }

        Some(Commands::List) => {
            let spinner = UI::new_spinner("Loading supported tools...");
            let mut executor = Executor::new()?;
            let tools = executor.list_tools()?;
            spinner.finish_and_clear();

            UI::header("Supported Tools");
            for tool in tools {
                println!("  * {}", tool);
            }
        }

        Some(Commands::Install {
            tool,
            version,
            force,
        }) => {
            let mut executor = Executor::new()?;
            let version = version.unwrap_or_else(|| "latest".to_string());

            // Check if already installed (only for system tools when not forcing)
            if !force {
                if let Ok(_) = which::which(&tool) {
                    // Check if it's a vx-managed package
                    let vx_versions = executor.get_package_manager()?.list_versions(&tool);
                    if vx_versions.is_empty() {
                        UI::success(&format!("{} is already installed (system)", tool));
                        UI::hint("Use --force to install vx-managed version");
                        return Ok(());
                    }
                }
            }

            UI::step(&format!("Installing {} {}...", tool, version));

            match executor.install_tool(&tool, &version).await {
                Ok(path) => {
                    UI::success(&format!(
                        "Successfully installed {} {} to {}",
                        tool,
                        version,
                        path.display()
                    ));

                    // Add to PATH if needed
                    if let Some(parent) = path.parent() {
                        UI::hint(&format!("Make sure {} is in your PATH", parent.display()));
                        UI::hint(&format!(
                            "Or use 'vx {}' to run the vx-managed version",
                            tool
                        ));
                    }
                }
                Err(e) => {
                    UI::error(&format!("Failed to install {}: {}", tool, e));
                    std::process::exit(1);
                }
            }
        }

        Some(Commands::Use { tool_version }) => {
            let parts: Vec<&str> = tool_version.split('@').collect();
            if parts.len() != 2 {
                UI::error("Invalid format. Use: tool@version (e.g., uv@1.0.0)");
                std::process::exit(1);
            }

            let tool_name = parts[0];
            let version = parts[1];

            let spinner =
                UI::new_spinner(&format!("Setting {} to version {}...", tool_name, version));

            let mut config = Config::load()?;
            config.set_tool(
                tool_name.to_string(),
                ToolConfig {
                    version: Some(version.to_string()),
                    install_method: None,
                    proxy_command: None,
                },
            );
            config.save()?;

            spinner.finish_and_clear();
            UI::success(&format!("Set {} to version {}", tool_name, version));
        }

        Some(Commands::Config) => {
            let spinner = UI::new_spinner("Loading configuration...");
            let config = Config::load()?;
            let config_str = toml::to_string_pretty(&config)?;
            spinner.finish_and_clear();

            UI::header("Current Configuration");
            println!("{}", config_str);
        }

        Some(Commands::Init) => {
            let spinner = UI::new_spinner("Initializing configuration...");
            let config = Config::default();
            let config_str = toml::to_string_pretty(&config)?;
            std::fs::write(".vx.toml", config_str)?;
            spinner.finish_and_clear();

            UI::success("Initialized .vx.toml in current directory");
        }

        Some(Commands::Switch { tool_version }) => {
            let mut executor = Executor::new()?;
            let parts: Vec<&str> = tool_version.split('@').collect();
            if parts.len() != 2 {
                UI::error("Invalid format. Use: tool@version (e.g., go@1.21.6)");
                std::process::exit(1);
            }

            let tool_name = parts[0];
            let version = parts[1];

            let spinner = UI::new_spinner(&format!(
                "Switching {} to version {}...",
                tool_name, version
            ));

            match executor.switch_version(tool_name, version) {
                Ok(()) => {
                    spinner.finish_and_clear();
                    UI::success(&format!("Switched {} to version {}", tool_name, version));
                }
                Err(e) => {
                    spinner.finish_and_clear();
                    UI::error(&format!("Failed to switch version: {}", e));
                    std::process::exit(1);
                }
            }
        }

        Some(Commands::Remove {
            tool,
            version,
            force,
        }) => {
            let mut executor = Executor::new()?;

            if !force {
                let confirmation_message = if let Some(version) = &version {
                    format!("Remove {} version {}?", tool, version)
                } else {
                    format!("Remove all versions of {}?", tool)
                };

                if !UI::confirm(&confirmation_message, false)? {
                    UI::info("Operation cancelled");
                    return Ok(());
                }
            }

            let spinner = if let Some(version) = &version {
                UI::new_spinner(&format!("Removing {} version {}...", tool, version))
            } else {
                UI::new_spinner(&format!("Removing all versions of {}...", tool))
            };

            match version {
                Some(version) => {
                    executor.remove_version(&tool, &version)?;
                    spinner.finish_and_clear();
                    UI::success(&format!("Removed {} version {}", tool, version));
                }
                None => {
                    executor.remove_tool(&tool)?;
                    spinner.finish_and_clear();
                    UI::success(&format!("Removed all versions of {}", tool));
                }
            }
        }

        Some(Commands::Cleanup) => {
            let mut executor = Executor::new()?;
            executor.cleanup()?;
        }

        Some(Commands::Stats) => {
            let spinner = UI::new_spinner("Collecting package statistics...");
            let mut executor = Executor::new()?;
            let stats = executor.get_stats()?;
            spinner.finish_and_clear();

            UI::show_stats(
                stats.total_packages,
                stats.total_versions,
                stats.total_size,
                &stats
                    .last_updated
                    .format("%Y-%m-%d %H:%M:%S UTC")
                    .to_string(),
            );

            // List installed packages
            if let Ok(packages) = executor.list_installed_packages() {
                if !packages.is_empty() {
                    // Create a simple list without active status for now
                    let package_list: Vec<(String, String, bool)> = packages
                        .iter()
                        .map(|package| {
                            // For now, mark all as inactive to avoid borrowing issues
                            // TODO: Improve this to show actual active status
                            (package.name.clone(), package.version.clone(), false)
                        })
                        .collect();

                    println!();
                    UI::show_package_list(&package_list);
                }
            }
        }

        Some(Commands::Update { tool, apply }) => {
            let executor = Executor::new()?;

            let spinner = UI::new_spinner("Checking for updates...");
            let updates = executor.check_updates(tool.as_deref()).await?;
            spinner.finish_and_clear();

            UI::show_updates(&updates);

            if !updates.is_empty() && apply {
                UI::step("Applying updates...");
                // TODO: Implement actual update logic
                UI::warning("Update functionality coming soon");
            } else if !updates.is_empty() {
                UI::hint("Run with --apply to install updates");
            }
        }

        Some(Commands::Plugin { command }) => {
            let mut plugin_manager = PluginManager::new()?;

            match command {
                PluginCommand::List { enabled, category } => {
                    let plugins = if enabled {
                        plugin_manager.list_enabled_plugins()
                    } else {
                        plugin_manager.list_plugins()
                    };

                    let filtered_plugins: Vec<_> = if let Some(cat_str) = category {
                        let category = match cat_str.to_lowercase().as_str() {
                            "language" => PluginCategory::Language,
                            "runtime" => PluginCategory::Runtime,
                            "package" | "packagemanager" => PluginCategory::PackageManager,
                            "build" | "buildtool" => PluginCategory::BuildTool,
                            "vcs" | "versioncontrol" => PluginCategory::VersionControl,
                            "database" => PluginCategory::Database,
                            "cloud" => PluginCategory::Cloud,
                            "devops" => PluginCategory::DevOps,
                            "editor" => PluginCategory::Editor,
                            "utility" => PluginCategory::Utility,
                            _ => {
                                UI::error(&format!("Unknown category: {}", cat_str));
                                std::process::exit(1);
                            }
                        };
                        plugin_manager.find_plugins_by_category(&category)
                    } else {
                        plugins
                    };

                    if filtered_plugins.is_empty() {
                        UI::info("No plugins found");
                    } else {
                        UI::header("Available Plugins");
                        for plugin in filtered_plugins {
                            let metadata = plugin.metadata();
                            println!("  * {} - {}", metadata.name, metadata.description);
                            println!("    Categories: {:?}", metadata.categories);
                        }
                    }
                }

                PluginCommand::Info { name } => {
                    plugin_manager.show_plugin_info(&name)?;
                }

                PluginCommand::Enable { name } => {
                    plugin_manager.enable_plugin(&name)?;
                }

                PluginCommand::Disable { name } => {
                    plugin_manager.disable_plugin(&name)?;
                }

                PluginCommand::Search { query } => {
                    let plugins = plugin_manager.search_plugins(&query);
                    if plugins.is_empty() {
                        UI::info(&format!("No plugins found matching '{}'", query));
                    } else {
                        UI::header(&format!("Plugins matching '{}'", query));
                        for plugin in plugins {
                            let metadata = plugin.metadata();
                            println!("  * {} - {}", metadata.name, metadata.description);
                        }
                    }
                }

                PluginCommand::Stats => {
                    let stats = plugin_manager.get_stats();
                    UI::header("Plugin Statistics");
                    println!("  Total plugins: {}", stats.total_plugins);
                    println!("  Enabled plugins: {}", stats.enabled_plugins);
                    println!("  Disabled plugins: {}", stats.disabled_plugins);

                    if !stats.categories.is_empty() {
                        println!("  By category:");
                        for (category, count) in &stats.categories {
                            println!("    * {:?}: {}", category, count);
                        }
                    }
                }
            }
        }

        None => {
            // Handle tool execution
            if cli.args.is_empty() {
                UI::error("No tool specified");
                UI::hint("Usage: vx <tool> [args...]");
                UI::hint("Example: vx uv pip install requests");
                UI::hint("Run 'vx list' to see supported tools");
                std::process::exit(1);
            }

            let tool_name = &cli.args[0];
            let tool_args = &cli.args[1..];

            let mut executor = Executor::new()?;
            match executor
                .execute(tool_name, tool_args, cli.use_system_path)
                .await
            {
                Ok(exit_code) => std::process::exit(exit_code),
                Err(e) => {
                    UI::error(&e.to_string());
                    std::process::exit(1);
                }
            }
        }
    }

    Ok(())
}
