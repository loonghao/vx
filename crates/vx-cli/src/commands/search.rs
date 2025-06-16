// Search command implementation

use crate::cli::OutputFormat;
use crate::ui::UI;
use std::collections::HashMap;
use vx_core::PluginRegistry;
use vx_core::Result;

pub async fn handle(
    registry: &PluginRegistry,
    query: Option<String>,
    category: Option<String>,
    _installed_only: bool,
    _available_only: bool,
    format: OutputFormat,
    verbose: bool,
) -> Result<()> {
    let spinner = UI::new_spinner("Searching tools...");

    // Get all available tools from plugins
    let mut all_tools = Vec::new();
    for plugin in registry.get_plugins() {
        for tool in plugin.tools() {
            all_tools.push((
                tool.name().to_string(),
                tool.description().to_string(),
                plugin.name().to_string(),
            ));
        }
    }

    spinner.finish_and_clear();

    // Filter tools based on query
    let filtered_tools: Vec<_> = all_tools
        .into_iter()
        .filter(|(name, description, _plugin)| {
            // Apply query filter
            if let Some(ref q) = query {
                let q_lower = q.to_lowercase();
                if !name.to_lowercase().contains(&q_lower)
                    && !description.to_lowercase().contains(&q_lower)
                {
                    return false;
                }
            }

            // Apply category filter
            if let Some(ref cat) = category {
                let cat_lower = cat.to_lowercase();
                // Simple category mapping based on tool names
                let tool_category = get_tool_category(name);
                if !tool_category.to_lowercase().contains(&cat_lower) {
                    return false;
                }
            }

            true
        })
        .collect();

    // Display results
    match format {
        OutputFormat::Table => display_table(&filtered_tools, verbose, &query),
        OutputFormat::Json => display_json(&filtered_tools),
        OutputFormat::Yaml => display_yaml(&filtered_tools),
    }

    Ok(())
}

fn get_tool_category(tool_name: &str) -> &'static str {
    match tool_name {
        "node" | "npm" | "npx" => "javascript",
        "go" => "go",
        "cargo" => "rust",
        "uv" | "uvx" => "python",
        _ => "utility",
    }
}

fn display_table(tools: &[(String, String, String)], verbose: bool, query: &Option<String>) {
    if let Some(q) = query {
        UI::info(&format!("Search results for: {}", q));
    } else {
        UI::info("Available tools:");
    }

    if tools.is_empty() {
        UI::warn("No tools found matching the criteria");
        return;
    }

    println!();
    if verbose {
        for (name, description, plugin) in tools {
            let category = get_tool_category(name);
            println!("{} ({})", name, category);
            println!("  Plugin: {}", plugin);
            println!("  Description: {}", description);
            println!("  Status: Available"); // TODO: Check if installed
            println!();
        }
    } else {
        // Simple table format
        println!("┌─────────────┬─────────────┬─────────────────────────────────────┐");
        println!("│ Tool        │ Category    │ Description                         │");
        println!("├─────────────┼─────────────┼─────────────────────────────────────┤");

        for (name, description, _plugin) in tools {
            let category = get_tool_category(name);
            let truncated_desc = if description.len() > 35 {
                format!("{}...", &description[..32])
            } else {
                description.clone()
            };

            println!(
                "│ {:<11} │ {:<11} │ {:<35} │",
                name, category, truncated_desc
            );
        }

        println!("└─────────────┴─────────────┴─────────────────────────────────────┘");
    }

    UI::info(&format!("Found {} tools", tools.len()));
}

fn display_json(tools: &[(String, String, String)]) {
    let mut json_tools = Vec::new();

    for (name, description, plugin) in tools {
        let mut tool_info = HashMap::new();
        tool_info.insert("name", name.clone());
        tool_info.insert("description", description.clone());
        tool_info.insert("plugin", plugin.clone());
        tool_info.insert("category", get_tool_category(name).to_string());
        tool_info.insert("status", "available".to_string()); // TODO: Check if installed
        json_tools.push(tool_info);
    }

    match serde_json::to_string_pretty(&json_tools) {
        Ok(json) => println!("{}", json),
        Err(e) => UI::error(&format!("Failed to serialize to JSON: {}", e)),
    }
}

fn display_yaml(tools: &[(String, String, String)]) {
    let mut yaml_tools = Vec::new();

    for (name, description, plugin) in tools {
        let mut tool_info = HashMap::new();
        tool_info.insert("name", name.clone());
        tool_info.insert("description", description.clone());
        tool_info.insert("plugin", plugin.clone());
        tool_info.insert("category", get_tool_category(name).to_string());
        tool_info.insert("status", "available".to_string()); // TODO: Check if installed
        yaml_tools.push(tool_info);
    }

    match serde_yaml::to_string(&yaml_tools) {
        Ok(yaml) => println!("{}", yaml),
        Err(e) => UI::error(&format!("Failed to serialize to YAML: {}", e)),
    }
}
