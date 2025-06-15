//! Execute command implementation - Transparent proxy for tool execution

use crate::ui::UI;
use vx_core::{PluginRegistry, Result, ToolProxy, VxError};

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

/// Execute tool with given arguments using the transparent proxy system
pub async fn execute_tool(
    _registry: &PluginRegistry,
    tool_name: &str,
    args: &[String],
    use_system_path: bool,
) -> Result<i32> {
    UI::debug(&format!("Executing: {} {}", tool_name, args.join(" ")));

    if use_system_path {
        // Use system PATH directly
        UI::debug(&format!("Using system PATH for: {}", tool_name));
        return execute_system_tool(tool_name, args).await;
    }

    // Use the transparent proxy system
    match ToolProxy::new() {
        Ok(proxy) => {
            UI::debug(&format!("Using vx transparent proxy for: {}", tool_name));

            // Check if tool is available
            if !proxy.is_tool_available(tool_name).await {
                UI::warn(&format!(
                    "Tool '{}' not found in vx or system PATH",
                    tool_name
                ));
                UI::hint(&format!(
                    "Try running 'vx install {}' to install it",
                    tool_name
                ));
                UI::hint("Or use --use-system-path to search system PATH only");
                return Err(VxError::ToolNotFound {
                    tool_name: tool_name.to_string(),
                });
            }

            // Show which version will be used
            if let Ok(version) = proxy.get_effective_version(tool_name).await {
                UI::debug(&format!("Using {} version: {}", tool_name, version));
            }

            // Execute through proxy
            proxy.execute_tool(tool_name, args).await
        }
        Err(e) => {
            UI::warn(&format!("Failed to create tool proxy: {}", e));
            UI::info("Falling back to system PATH execution");
            execute_system_tool(tool_name, args).await
        }
    }
}

/// Execute a tool using system PATH
async fn execute_system_tool(tool_name: &str, args: &[String]) -> Result<i32> {
    let status = std::process::Command::new(tool_name)
        .args(args)
        .status()
        .map_err(|_| VxError::ToolNotFound {
            tool_name: tool_name.to_string(),
        })?;

    Ok(status.code().unwrap_or(1))
}
