//! Environment info display for `vx dev --info`

use super::tools::{ToolStatus, find_system_tool, get_tool_status, get_vx_tool_path};
use crate::commands::common::get_system_tool_version;
use crate::commands::setup::ConfigView;
use crate::commands::tool_paths::find_system_executable;
use anyhow::Result;
use colored::Colorize;
use std::env;
use vx_env::ToolEnvironment;
use vx_paths::PathManager;

/// Handle --info mode: display detailed environment information
pub async fn handle_info(config: &ConfigView) -> Result<()> {
    let path_manager = PathManager::new()?;

    println!("{}", "VX Development Environment Information".bold());
    println!("{}", "═".repeat(50));
    println!();

    // Display configured tools and their status
    println!("{}", "Configured Tools:".bold().cyan());
    println!();

    for (tool, version) in &config.tools {
        let (status, actual_path, actual_version) = get_tool_status(&path_manager, tool, version)?;

        // `check_tool_status()` only looks at the vx store and the system PATH.
        // Tools that live in a well-known directory declared through
        // `provider.star::runtimes[].system_paths` (MSVC's `cl`, LLVM's
        // `clang-cl`) are therefore reported as missing even though the dev
        // shell does put them on PATH. Fall back to that same detection here so
        // the report matches the environment `vx dev` actually builds.
        let (status, actual_path, actual_version) = if status == ToolStatus::NotInstalled {
            match find_system_executable(tool).await {
                Some(path) => (
                    ToolStatus::SystemFallback,
                    Some(path),
                    actual_version.or_else(|| get_system_tool_version(tool)),
                ),
                None => (status, actual_path, actual_version),
            }
        } else {
            (status, actual_path, actual_version)
        };

        let status_icon = match status {
            ToolStatus::Installed => "✓".green(),
            ToolStatus::NotInstalled => "✗".red(),
            ToolStatus::SystemFallback => "✓".green(),
        };

        // For system tools, show the detected version instead of "system"
        let display_version = if version == "system" {
            actual_version
                .clone()
                .unwrap_or_else(|| "system".to_string())
        } else {
            version.clone()
        };

        let version_suffix = if version == "system" {
            " (system)".dimmed().to_string()
        } else {
            String::new()
        };

        println!(
            "  {} {}@{}{}",
            status_icon,
            tool.cyan(),
            display_version,
            version_suffix
        );

        if let Some(path) = actual_path {
            println!("    {} {}", "Path:".dimmed(), path.display());
        }

        // Show actual version if different from configured (for non-system tools)
        if version != "system"
            && let Some(ver) = actual_version
            && ver != *version
            && version != "latest"
        {
            println!("    {} {}", "Actual version:".dimmed(), ver);
        }
    }

    println!();

    // Display PATH entries that will be added
    println!("{}", "PATH Entries (in priority order):".bold().cyan());
    println!();

    let env_vars = ToolEnvironment::new()
        .tools(&config.tools)
        .env_vars(&config.env)
        .warn_missing(false)
        .build()?;

    if let Some(path) = env_vars.get("PATH") {
        let sep = if cfg!(windows) { ";" } else { ":" };
        let current_path = env::var("PATH").unwrap_or_default();

        // Show only the new entries (before current PATH starts)
        for (i, entry) in path.split(sep).enumerate() {
            if current_path.starts_with(entry) {
                println!(
                    "  {} {}",
                    (i + 1).to_string().dimmed(),
                    "... (system PATH)".dimmed()
                );
                break;
            }
            println!("  {} {}", (i + 1).to_string().dimmed(), entry);
        }
    }

    println!();

    // Display custom environment variables
    if !config.env.is_empty() {
        println!("{}", "Custom Environment Variables:".bold().cyan());
        println!();
        for (key, value) in &config.env {
            println!("  {} = {}", key.yellow(), value);
        }
        println!();
    }

    // Show potential conflicts with system tools
    println!("{}", "System Tool Conflicts:".bold().cyan());
    println!();

    let mut has_conflicts = false;
    for tool in config.tools.keys() {
        if let Some(system_path) = find_system_tool(tool) {
            let vx_path = get_vx_tool_path(&path_manager, tool, &config.tools[tool])?;
            if let Some(vx_p) = vx_path {
                println!(
                    "  {} {}",
                    "⚠".yellow(),
                    format!("{} found in system PATH:", tool).yellow()
                );
                println!("    {} {}", "System:".dimmed(), system_path.display());
                println!("    {} {} (will be used)", "VX:".dimmed(), vx_p.display());
                has_conflicts = true;
            }
        }
    }

    if !has_conflicts {
        println!("  {} No conflicts detected", "✓".green());
    }

    println!();
    println!(
        "{}",
        "Run 'vx dev' to enter the development environment.".dimmed()
    );

    Ok(())
}
