//! Execute command implementation - Transparent proxy for tool execution

use crate::ui::UI;
use anyhow::Result;
use std::collections::HashMap;
use vx_plugin::PluginRegistry;

/// Handle the execute command
pub async fn handle(
    registry: &PluginRegistry,
    tool_name: &str,
    args: &[String],
    use_system_path: bool,
) -> Result<()> {
    let exit_code = execute_tool(registry, tool_name, args, use_system_path).await?;
    if exit_code != 0 {
        std::process::exit(exit_code);
    }
    Ok(())
}

/// Execute tool with given arguments with smart tool resolution
pub async fn execute_tool(
    registry: &PluginRegistry,
    tool_name: &str,
    args: &[String],
    use_system_path: bool,
) -> Result<i32> {
    UI::debug(&format!("Executing: {} {}", tool_name, args.join(" ")));

    // Try to find the tool in vx-managed tools first
    if let Some(tool) = registry.get_tool(tool_name) {
        return execute_vx_tool(tool, args).await;
    }

    // If use_system_path is true, try system PATH
    if use_system_path {
        if let Ok(exit_code) = execute_system_tool(tool_name, args).await {
            return Ok(exit_code);
        }
    }

    // Tool not found, try to auto-install if supported
    if let Some(tool) = registry.get_tool(tool_name) {
        UI::info(&format!(
            "Tool '{}' not found, attempting to install...",
            tool_name
        ));

        // Try to install the latest version
        if let Err(e) = tool.install_version("latest", false).await {
            UI::warn(&format!("Failed to auto-install {}: {}", tool_name, e));
            return Err(anyhow::anyhow!(
                "Tool not found and auto-install failed: {}",
                tool_name
            ));
        }

        UI::success(&format!("Successfully installed {}", tool_name));
        return execute_vx_tool(tool, args).await;
    }

    Err(anyhow::anyhow!("Tool not found: {}", tool_name))
}

/// Execute a vx-managed tool
async fn execute_vx_tool(tool: Box<dyn vx_plugin::VxTool>, args: &[String]) -> Result<i32> {
    let context = vx_plugin::ToolContext {
        working_directory: std::env::current_dir().ok(),
        environment_variables: HashMap::new(),
        use_system_path: false,
        options: HashMap::new(),
    };

    let result = tool.execute(args, &context).await?;
    Ok(result.exit_code)
}

/// Execute a tool using system PATH
async fn execute_system_tool(tool_name: &str, args: &[String]) -> Result<i32> {
    let status = std::process::Command::new(tool_name)
        .args(args)
        .status()
        .map_err(|_| anyhow::anyhow!("Tool not found: {}", tool_name))?;

    Ok(status.code().unwrap_or(1))
}
