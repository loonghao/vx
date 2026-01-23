//! Tool installation and status checking

use super::tools::{find_system_tool, get_system_tool_version, ToolStatus};
use crate::ui::{InstallProgress, UI};
use anyhow::{Context, Result};
use colored::Colorize;
use std::collections::HashMap;
use std::env;
use std::process::{Command, Stdio};
use vx_paths::PathManager;

/// Check if tools are installed and install missing ones
pub async fn check_and_install_tools(tools: &HashMap<String, String>, verbose: bool) -> Result<()> {
    let path_manager = PathManager::new()?;
    // tool_states: (tool_name, display_version, status)
    // display_version is the actual detected version for system tools
    let mut tool_states: Vec<(String, String, ToolStatus)> = Vec::new();
    let mut missing_tools: Vec<(String, String)> = Vec::new();

    // First pass: check all tools
    for (tool, version) in tools {
        // "system" version means use system-installed tool, skip vx management
        if version == "system" {
            if find_system_tool(tool).is_some() {
                // Try to detect the actual version
                let display_version =
                    get_system_tool_version(tool).unwrap_or_else(|| "system".to_string());
                tool_states.push((tool.clone(), display_version, ToolStatus::SystemFallback));
            } else {
                // System tool not found, but we don't try to install it
                UI::warn(&format!(
                    "{} specified as 'system' but not found in PATH",
                    tool
                ));
                tool_states.push((tool.clone(), "system".to_string(), ToolStatus::NotInstalled));
            }
            continue;
        }

        let status = if version == "latest" {
            let versions = path_manager.list_store_versions(tool)?;
            if versions.is_empty() {
                missing_tools.push((tool.clone(), version.clone()));
                ToolStatus::NotInstalled
            } else {
                ToolStatus::Installed
            }
        } else if path_manager.is_version_in_store(tool, version) {
            ToolStatus::Installed
        } else {
            missing_tools.push((tool.clone(), version.clone()));
            ToolStatus::NotInstalled
        };
        tool_states.push((tool.clone(), version.clone(), status));
    }

    // Show status of all tools
    if verbose || !missing_tools.is_empty() {
        println!();
        for (tool, version, status) in &tool_states {
            let icon = match status {
                ToolStatus::Installed => "✓".green(),
                ToolStatus::NotInstalled => "○".yellow(),
                ToolStatus::SystemFallback => "✓".green(), // System tools are valid
            };
            let status_text = match status {
                ToolStatus::Installed => "installed".green(),
                ToolStatus::NotInstalled => "pending".yellow(),
                ToolStatus::SystemFallback => "system".dimmed(),
            };
            println!("  {} {}@{} ({})", icon, tool, version, status_text);
        }
        println!();
    }

    if missing_tools.is_empty() {
        UI::success("All tools installed");
        return Ok(());
    }

    // Use InstallProgress for modern progress display
    let mut progress = InstallProgress::new(
        missing_tools.len(),
        &format!("Installing {} missing tool(s)", missing_tools.len()),
    );

    let mut install_results: Vec<(String, String, bool)> = Vec::new();

    for (tool, version) in &missing_tools {
        progress.start_tool(tool, version);

        // Use vx install command with suppressed output
        // Use tool@version format instead of separate arguments
        let status = Command::new(env::current_exe()?)
            .args(["install", &format!("{}@{}", tool, version)])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .with_context(|| format!("Failed to install {}@{}", tool, version))?;

        let success = status.success();
        progress.complete_tool(success, tool, version);
        install_results.push((tool.clone(), version.clone(), success));
    }

    // Check if all installations succeeded
    let all_success = install_results.iter().all(|(_, _, s)| *s);
    if all_success {
        progress.finish("✓ All tools installed");
    } else {
        progress.finish("⚠ Some tools failed to install");

        // Show which tools failed
        for (tool, version, success) in &install_results {
            if !success {
                UI::error(&format!("Failed to install {}@{}", tool, version));
            }
        }
    }

    Ok(())
}
