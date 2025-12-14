// Plugin command implementation

use crate::cli::PluginCommand;
use crate::ui::UI;
use anyhow::Result;
use vx_plugin::BundleRegistry;

pub async fn handle(registry: &BundleRegistry, command: PluginCommand) -> Result<()> {
    // TODO: Replace with vx-core tool manager
    // let tool_manager = crate::tool_manager::ToolManager::new()
    //     .or_else(|_| crate::tool_manager::ToolManager::minimal())?;

    match command {
        PluginCommand::List {
            enabled: _,
            category: _,
        } => {
            UI::header("Available Bundles");

            let bundles = registry.list_bundles();
            if bundles.is_empty() {
                UI::warn("No bundles registered");
                return Ok(());
            }

            for bundle_name in bundles {
                UI::item(&format!("📦 {}", bundle_name));
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
            UI::warning("Plugin info not yet implemented in new architecture");
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
            UI::warning("Plugin search not yet implemented in new architecture");
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
            UI::warning("Plugin stats not yet implemented in new architecture");
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
