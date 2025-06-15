//! Execute command implementation

use crate::ui::UI;
use vx_core::{PluginRegistry, Result, ToolContext, VxError};

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

/// Execute tool with given arguments
pub async fn execute_tool(
    registry: &PluginRegistry,
    tool_name: &str,
    args: &[String],
    use_system_path: bool,
) -> Result<i32> {
    UI::debug(&format!("Executing: {} {}", tool_name, args.join(" ")));

    // Create execution context
    let mut context = ToolContext::new(
        tool_name.to_string(),
        "latest".to_string(), // Default version for now
        args.to_vec(),
    );
    context.use_system_path = use_system_path;
    context.working_directory = std::env::current_dir().ok();

    // Check if tool is supported by vx
    if !use_system_path && registry.supports_tool(tool_name) {
        // Try to get the tool from registry
        if let Some(tool) = registry.get_tool(tool_name) {
            // Check if tool is installed, if not, auto-install
            if !tool.is_version_installed("latest").await.unwrap_or(false) {
                UI::info(&format!(
                    "Tool '{}' not installed, installing automatically...",
                    tool_name
                ));

                // Auto-install the latest version
                match tool.install_version("latest", false).await {
                    Ok(()) => {
                        UI::success(&format!("Successfully auto-installed {}", tool_name));
                    }
                    Err(e) => {
                        UI::error(&format!("Failed to auto-install {}: {}", tool_name, e));
                        UI::hint(&format!("Try running 'vx install {}' manually", tool_name));
                        return Err(e);
                    }
                }
            }

            UI::debug(&format!("Using vx-managed tool: {}", tool_name));

            // Execute using the tool's implementation
            let result = tool.execute(args, &context).await?;
            return Ok(result.exit_code);
        }
    }

    // Fall back to system PATH execution
    if use_system_path {
        UI::debug(&format!("Using system PATH for: {}", tool_name));
    } else {
        UI::warn(&format!(
            "Tool '{}' not found in vx registry, falling back to system PATH",
            tool_name
        ));
    }

    let status = std::process::Command::new(tool_name)
        .args(args)
        .status()
        .map_err(|_| VxError::ToolNotFound {
            tool_name: tool_name.to_string(),
        })?;

    Ok(status.code().unwrap_or(1))
}
