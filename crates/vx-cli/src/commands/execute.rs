//! Execute command implementation

use vx_core::{Result, VxError, ToolContext};
use crate::ui::UI;
use std::collections::HashMap;

/// Execute tool with given arguments
pub async fn execute_tool(
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

    // For now, just execute using system PATH
    if use_system_path {
        let status = std::process::Command::new(tool_name)
            .args(args)
            .status()
            .map_err(|_| VxError::ToolNotFound {
                tool_name: tool_name.to_string()
            })?;

        Ok(status.code().unwrap_or(1))
    } else {
        // TODO: Implement vx-managed tool execution
        UI::warn(&format!("vx-managed execution not yet implemented for {}", tool_name));
        UI::hint("Using system PATH as fallback");

        let status = std::process::Command::new(tool_name)
            .args(args)
            .status()
            .map_err(|_| VxError::ToolNotFound {
                tool_name: tool_name.to_string()
            })?;

        Ok(status.code().unwrap_or(1))
    }
}


