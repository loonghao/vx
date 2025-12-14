//! Execute command implementation - Transparent proxy for tool execution

use crate::ui::UI;
use anyhow::Result;
use vx_plugin::BundleRegistry;
// TODO: DynamicExecutor needs to be implemented or imported from appropriate crate

/// Handle the execute command
pub async fn handle(
    registry: &BundleRegistry,
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

/// Execute tool with given arguments (simplified for closed-loop toolchain)
pub async fn execute_tool(
    _registry: &BundleRegistry,
    tool_name: &str,
    args: &[String],
    _use_system_path: bool,
) -> Result<i32> {
    UI::debug(&format!("Executing: {} {}", tool_name, args.join(" ")));

    // For now, use direct system execution
    // TODO: Implement smart tool resolution with vx-managed tools
    execute_system_tool(tool_name, args).await
}

/// Execute a tool using system PATH
async fn execute_system_tool(tool_name: &str, args: &[String]) -> Result<i32> {
    let status = std::process::Command::new(tool_name)
        .args(args)
        .status()
        .map_err(|_| anyhow::anyhow!("Tool not found: {}", tool_name))?;

    Ok(status.code().unwrap_or(1))
}
