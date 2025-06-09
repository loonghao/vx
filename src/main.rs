use clap::Parser;
use std::io::Write;
use vx::cli::{Cli, Commands, PluginCommand};
use vx::config::{Config, ToolConfig};
use vx::executor::Executor;
use vx::plugin::PluginCategory;
use vx::plugin_manager::PluginManager;
use vx::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Version) => {
            println!("vx {}", env!("CARGO_PKG_VERSION"));
            println!("Universal version executor for development tools");
        }

        Some(Commands::List) => {
            let executor = Executor::new()?;
            let tools = executor.list_tools();

            println!("📦 Supported tools:");
            for tool in tools {
                println!("  • {}", tool);
            }
        }

        Some(Commands::Install { tool, version, force }) => {
            let mut executor = Executor::new()?;
            let version = version.unwrap_or_else(|| "latest".to_string());

            // Check if already installed (only for system tools when not forcing)
            if !force {
                if let Ok(_) = which::which(&tool) {
                    // Check if it's a vx-managed package
                    let vx_versions = executor.get_package_manager().list_versions(&tool);
                    if vx_versions.is_empty() {
                        println!("✅ {} is already installed (system)", tool);
                        println!("💡 Use --force to install vx-managed version");
                        return Ok(());
                    }
                }
            }

            match executor.install_tool(&tool, &version).await {
                Ok(path) => {
                    println!("🎉 Successfully installed {} {} to {}", tool, version, path.display());

                    // Add to PATH if needed
                    if let Some(parent) = path.parent() {
                        println!("💡 Make sure {} is in your PATH", parent.display());
                        println!("💡 Or use 'vx {}' to run the vx-managed version", tool);
                    }
                }
                Err(e) => {
                    eprintln!("❌ Failed to install {}: {}", tool, e);
                    std::process::exit(1);
                }
            }
        }

        Some(Commands::Use { tool_version }) => {
            let parts: Vec<&str> = tool_version.split('@').collect();
            if parts.len() != 2 {
                eprintln!("❌ Invalid format. Use: tool@version (e.g., uv@1.0.0)");
                std::process::exit(1);
            }

            let tool_name = parts[0];
            let version = parts[1];

            let mut config = Config::load()?;
            config.set_tool(tool_name.to_string(), ToolConfig {
                version: Some(version.to_string()),
                install_method: None,
                proxy_command: None,
            });
            config.save()?;

            println!("✅ Set {} to version {}", tool_name, version);
        }

        Some(Commands::Config) => {
            let config = Config::load()?;
            let config_str = toml::to_string_pretty(&config)?;
            println!("📋 Current configuration:");
            println!("{}", config_str);
        }

        Some(Commands::Init) => {
            let config = Config::default();
            let config_str = toml::to_string_pretty(&config)?;
            std::fs::write(".vx.toml", config_str)?;
            println!("✅ Initialized .vx.toml in current directory");
        }

        Some(Commands::Switch { tool_version }) => {
            let mut executor = Executor::new()?;
            let parts: Vec<&str> = tool_version.split('@').collect();
            if parts.len() != 2 {
                eprintln!("❌ Invalid format. Use: tool@version (e.g., go@1.21.6)");
                std::process::exit(1);
            }

            let tool_name = parts[0];
            let version = parts[1];

            match executor.switch_version(tool_name, version) {
                Ok(()) => {
                    println!("✅ Switched {} to version {}", tool_name, version);
                }
                Err(e) => {
                    eprintln!("❌ Failed to switch version: {}", e);
                    std::process::exit(1);
                }
            }
        }

        Some(Commands::Remove { tool, version, force }) => {
            let mut executor = Executor::new()?;

            if !force {
                let confirmation = if let Some(version) = &version {
                    format!("Remove {} version {}? [y/N]: ", tool, version)
                } else {
                    format!("Remove all versions of {}? [y/N]: ", tool)
                };

                print!("{}", confirmation);
                std::io::stdout().flush()?;

                let mut input = String::new();
                std::io::stdin().read_line(&mut input)?;
                let input = input.trim().to_lowercase();

                if input != "y" && input != "yes" {
                    println!("❌ Cancelled");
                    return Ok(());
                }
            }

            match version {
                Some(version) => {
                    executor.remove_version(&tool, &version)?;
                }
                None => {
                    executor.remove_tool(&tool)?;
                }
            }
        }

        Some(Commands::Cleanup) => {
            let mut executor = Executor::new()?;
            executor.cleanup()?;
        }

        Some(Commands::Stats) => {
            let executor = Executor::new()?;
            let stats = executor.get_stats();

            println!("📊 Package Statistics:");
            println!("  📦 Total packages: {}", stats.total_packages);
            println!("  🔢 Total versions: {}", stats.total_versions);
            println!("  💾 Total size: {}", stats.format_size());
            println!("  🕒 Last updated: {}", stats.last_updated.format("%Y-%m-%d %H:%M:%S UTC"));

            // List installed packages
            let packages = executor.list_installed_packages();
            if !packages.is_empty() {
                println!("\n📋 Installed packages:");
                for package in packages {
                    let active = if let Some(active_pkg) = executor.get_package_manager().get_active_version(&package.name) {
                        if active_pkg.version == package.version { " (active)" } else { "" }
                    } else { "" };

                    println!("  • {} {} {}", package.name, package.version, active);
                }
            }
        }

        Some(Commands::Update { tool, apply }) => {
            let executor = Executor::new()?;

            println!("🔍 Checking for updates...");
            let updates = executor.check_updates(tool.as_deref()).await?;

            if updates.is_empty() {
                println!("✅ All packages are up to date");
            } else {
                println!("📦 Available updates:");
                for (tool_name, current, latest) in &updates {
                    println!("  • {} {} → {}", tool_name, current, latest);
                }

                if apply {
                    println!("🚀 Applying updates...");
                    // TODO: Implement actual update logic
                    println!("⚠️  Update functionality coming soon");
                } else {
                    println!("💡 Run with --apply to install updates");
                }
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
                                eprintln!("❌ Unknown category: {}", cat_str);
                                std::process::exit(1);
                            }
                        };
                        plugin_manager.find_plugins_by_category(&category)
                    } else {
                        plugins
                    };

                    if filtered_plugins.is_empty() {
                        println!("📦 No plugins found");
                    } else {
                        println!("📦 Available plugins:");
                        for plugin in filtered_plugins {
                            let metadata = plugin.metadata();
                            println!("  • {} - {}", metadata.name, metadata.description);
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
                        println!("🔍 No plugins found matching '{}'", query);
                    } else {
                        println!("🔍 Plugins matching '{}':", query);
                        for plugin in plugins {
                            let metadata = plugin.metadata();
                            println!("  • {} - {}", metadata.name, metadata.description);
                        }
                    }
                }

                PluginCommand::Stats => {
                    let stats = plugin_manager.get_stats();
                    println!("📊 Plugin Statistics:");
                    println!("  📦 Total plugins: {}", stats.total_plugins);
                    println!("  ✅ Enabled plugins: {}", stats.enabled_plugins);
                    println!("  ❌ Disabled plugins: {}", stats.disabled_plugins);

                    if !stats.categories.is_empty() {
                        println!("  📋 By category:");
                        for (category, count) in &stats.categories {
                            println!("    • {:?}: {}", category, count);
                        }
                    }
                }
            }
        }

        None => {
            // Handle tool execution
            if cli.args.is_empty() {
                println!("❌ No tool specified");
                println!("💡 Usage: vx <tool> [args...]");
                println!("💡 Example: vx uv pip install requests");
                println!("💡 Run 'vx list' to see supported tools");
                std::process::exit(1);
            }

            let tool_name = &cli.args[0];
            let tool_args = &cli.args[1..];

            let mut executor = Executor::new()?;
            let exit_code = executor.execute(tool_name, tool_args).await?;
            std::process::exit(exit_code);
        }
    }

    Ok(())
}
