// Plugin command implementation

use crate::cli::PluginCommand;
use crate::ui::UI;
use anyhow::Result;
use vx_plugin::PluginRegistry;

pub async fn handle(registry: &PluginRegistry, command: PluginCommand) -> Result<()> {
    // TODO: Replace with vx-core tool manager
    // let tool_manager = crate::tool_manager::ToolManager::new()
    //     .or_else(|_| crate::tool_manager::ToolManager::minimal())?;

    match command {
        PluginCommand::List {
            enabled: _,
            category: _,
        } => {
            UI::header("Available Plugins");

            let plugins = registry.list_plugins();
            if plugins.is_empty() {
                UI::warn("No plugins registered");
                return Ok(());
            }

            for plugin_name in plugins {
                UI::item(&format!("📦 {}", plugin_name));
            }
            // for tool in tools {
            //     let status_icon = if tool.installed { "✅" } else { "❌" };
            //     let version_str = tool
            //         .version
            //         .as_ref()
            //         .map(|v| format!(" ({v})"))
            //         .unwrap_or_default();

            //     println!(
            //         "  {} {} - {}{}",
            //         status_icon, tool.name, tool.description, version_str
            //     );

            //     if let Some(homepage) = &tool.homepage {
            //         println!("    🌐 {homepage}");
            //     }
            // }
        }

        PluginCommand::Info { name } => {
            UI::header(&format!("Tool: {name}"));

            // Check if tool exists in registry
            let available_tools = registry.list_tools();
            if available_tools.contains(&name) {
                println!("📦 Name: {}", name);
                println!("📝 Description: Available tool plugin");
                println!("✅ Available: Yes");

                // Check if tool is installed
                let paths = vx_paths::VxPaths::default();
                let tool_dir = paths.tools_dir.join(&name);
                let is_installed = tool_dir.exists()
                    && tool_dir
                        .read_dir()
                        .map(|mut d| d.next().is_some())
                        .unwrap_or(false);
                println!("💾 Installed: {}", if is_installed { "Yes" } else { "No" });

                if is_installed {
                    // Show installed versions
                    if let Ok(entries) = std::fs::read_dir(&tool_dir) {
                        let versions: Vec<String> = entries
                            .filter_map(|e| e.ok())
                            .filter(|e| e.path().is_dir())
                            .filter_map(|e| e.file_name().to_str().map(|s| s.to_string()))
                            .collect();

                        if !versions.is_empty() {
                            println!("📋 Installed versions: {}", versions.join(", "));
                        }
                    }
                }
            } else {
                UI::error(&format!("Tool '{}' not found in registry", name));
            }
            // match tool_manager.get_tool_info(&name) {
            //     Ok(info) => {
            //         UI::header(&format!("Tool: {}", info.name));
            //         println!("Description: {}", info.description);
            //         println!("Installed: {}", if info.installed { "Yes" } else { "No" });

            //         if let Some(version) = &info.version {
            //             println!("Version: {version}");
            //         }

            //         if let Some(homepage) = &info.homepage {
            //             println!("Homepage: {homepage}");
            //         }

            //         println!(
            //             "Auto-install: {}",
            //             if info.supports_auto_install {
            //                 "Yes"
            //             } else {
            //                 "No"
            //             }
            //         );
            //     }
            //     Err(e) => {
            //         UI::error(&format!("Tool not found: {e}"));
            //     }
            // }
        }

        PluginCommand::Enable { name: _ } => {
            UI::warning("Enable/disable commands not applicable to the new tool system");
            UI::hint("All tools are automatically available");
        }

        PluginCommand::Disable { name: _ } => {
            UI::warning("Enable/disable commands not applicable to the new tool system");
            UI::hint("All tools are automatically available");
        }

        PluginCommand::Search { query } => {
            UI::header(&format!("Tools matching '{query}'"));

            let available_tools = registry.list_tools();
            let matching_tools: Vec<_> = available_tools
                .into_iter()
                .filter(|tool| tool.to_lowercase().contains(&query.to_lowercase()))
                .collect();

            if matching_tools.is_empty() {
                UI::info(&format!("No tools found matching '{query}'"));
            } else {
                println!("Found {} matching tools:", matching_tools.len());
                for tool in matching_tools {
                    println!("  📦 {}", tool);
                }
            }
            // let tools = tool_manager.get_all_tools();
            // let matching_tools: Vec<_> = tools
            //     .into_iter()
            //     .filter(|tool| {
            //         tool.name.contains(&query)
            //             || tool
            //                 .description
            //                 .to_lowercase()
            //                 .contains(&query.to_lowercase())
            //     })
            //     .collect();

            // if matching_tools.is_empty() {
            //     UI::info(&format!("No tools found matching '{query}'"));
            // } else {
            //     UI::header(&format!("Tools matching '{query}'"));
            //     for tool in matching_tools {
            //         println!("  * {} - {}", tool.name, tool.description);
            //     }
            // }
        }

        PluginCommand::Stats => {
            UI::header("Tool Statistics");

            let available_tools = registry.list_tools();
            let total_tools = available_tools.len();

            // Count installed tools
            let paths = vx_paths::VxPaths::default();
            let mut installed_count = 0;

            if paths.tools_dir.exists() {
                for tool_name in &available_tools {
                    let tool_dir = paths.tools_dir.join(tool_name);
                    if tool_dir.exists()
                        && tool_dir
                            .read_dir()
                            .map(|mut d| d.next().is_some())
                            .unwrap_or(false)
                    {
                        installed_count += 1;
                    }
                }
            }

            println!("📊 Tool Statistics:");
            println!("  Total available tools: {}", total_tools);
            println!("  Installed tools: {}", installed_count);
            println!("  Not installed: {}", total_tools - installed_count);
            println!(
                "  Installation rate: {:.1}%",
                if total_tools > 0 {
                    (installed_count as f64 / total_tools as f64) * 100.0
                } else {
                    0.0
                }
            );
            // let tools = tool_manager.get_all_tools();
            // let total = tools.len();
            // let installed = tools.iter().filter(|t| t.installed).count();
            // let auto_install = tools.iter().filter(|t| t.supports_auto_install).count();

            // UI::header("Tool Statistics");
            // println!("  Total tools: {total}");
            // println!("  Installed tools: {installed}");
            // println!("  Not installed: {}", total - installed);
            // println!("  Support auto-install: {auto_install}");
        }
    }

    Ok(())
}
