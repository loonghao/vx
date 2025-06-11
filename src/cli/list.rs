// List command implementation
// Shows installed tools and their versions

use crate::tool_manager::ToolManager;
use crate::ui::UI;
use anyhow::Result;

pub async fn handle(tool: Option<String>, all: bool, detailed: bool) -> Result<()> {
    let tool_manager = ToolManager::new().or_else(|_| ToolManager::minimal())?;

    match tool {
        Some(tool_name) => {
            // List versions for specific tool
            list_tool_versions(&tool_name, detailed).await
        }
        None => {
            if all {
                // List all available tools
                list_all_tools(&tool_manager, detailed).await
            } else {
                // List only installed tools
                list_installed_tools(&tool_manager, detailed).await
            }
        }
    }
}

async fn list_tool_versions(tool_name: &str, detailed: bool) -> Result<()> {
    UI::header(&format!("Installed versions of {tool_name}"));

    // Get installed versions from package manager
    let package_manager = crate::package_manager::PackageManager::new()?;
    let versions = package_manager.list_versions(tool_name);

    if versions.is_empty() {
        UI::warning(&format!("No versions of {tool_name} are installed"));
        UI::hint(&format!("Install with: vx install {tool_name}"));
        return Ok(());
    }

    // Get current active version
    let active_version = get_active_version(tool_name).await;

    for version in versions {
        let is_active = active_version.as_ref() == Some(&version.version);
        let status_icon = if is_active { "→" } else { " " };

        if detailed {
            let install_path = package_manager.get_version_path(tool_name, version)?;
            let size = calculate_size(&install_path).unwrap_or(0);
            let size_str = format_size(size);

            println!(
                "  {} {} ({}) - {}",
                status_icon,
                version,
                size_str,
                install_path.display()
            );
        } else {
            println!("  {status_icon} {version}");
        }
    }

    if let Some(active) = active_version {
        println!();
        UI::info(&format!("Active version: {active}"));
    }

    Ok(())
}

async fn list_installed_tools(_tool_manager: &ToolManager, detailed: bool) -> Result<()> {
    UI::header("Installed Tools");

    let package_manager = crate::package_manager::PackageManager::new()?;
    let installed_tools = package_manager.list_installed_tools()?;

    if installed_tools.is_empty() {
        UI::info("No tools are currently installed");
        UI::hint("Install tools with: vx install <tool>");
        UI::hint("See available tools with: vx list --all");
        return Ok(());
    }

    for tool_name in installed_tools {
        let versions = package_manager.list_versions(&tool_name);
        let version_count = versions.len();
        let active_version = get_active_version(&tool_name).await;

        if detailed {
            let total_size = calculate_tool_total_size(&package_manager, &tool_name)?;
            let size_str = format_size(total_size);

            println!("  {tool_name} ({version_count} versions, {size_str})");

            if let Some(active) = active_version {
                println!("    → Active: {active}");
            }
        } else {
            let active_str = active_version
                .map(|v| format!(" (active: {v})"))
                .unwrap_or_default();

            println!("  {tool_name} ({version_count} versions){active_str}");
        }
    }

    Ok(())
}

async fn list_all_tools(tool_manager: &ToolManager, detailed: bool) -> Result<()> {
    UI::header("Available Tools");

    let tools = tool_manager.get_all_tools();
    let package_manager = crate::package_manager::PackageManager::new()?;

    for tool in &tools {
        let installed_versions = package_manager.list_versions(&tool.name);
        let is_installed = !installed_versions.is_empty();
        let status_icon = if is_installed { "✅" } else { "❌" };

        if detailed {
            let version_info = if is_installed {
                format!(" ({} versions installed)", installed_versions.len())
            } else {
                " (not installed)".to_string()
            };

            println!(
                "  {} {}{} - {}",
                status_icon, tool.name, version_info, tool.description
            );

            if let Some(homepage) = &tool.homepage {
                println!("    🌐 {homepage}");
            }

            if tool.supports_auto_install {
                println!("    📦 Auto-install supported");
            }
        } else {
            println!("  {} {} - {}", status_icon, tool.name, tool.description);
        }
    }

    let installed_count = tools.iter().filter(|t| t.installed).count();
    let total_count = tools.len();

    println!();
    UI::info(&format!("Installed: {installed_count}/{total_count} tools"));

    Ok(())
}

async fn get_active_version(tool_name: &str) -> Option<String> {
    // Try to get version from system PATH first
    if let Ok(output) = std::process::Command::new(tool_name)
        .arg("--version")
        .output()
    {
        if output.status.success() {
            let version_output = String::from_utf8_lossy(&output.stdout);
            if let Ok(version) =
                crate::version::VersionManager::extract_version_from_output(&version_output)
            {
                return Some(version);
            }
        }
    }

    // Fallback to checking vx-managed versions
    None
}

fn calculate_size(path: &std::path::Path) -> Result<u64> {
    if path.is_file() {
        Ok(path.metadata()?.len())
    } else if path.is_dir() {
        let mut size = 0;
        for entry in walkdir::WalkDir::new(path) {
            let entry = entry?;
            if entry.file_type().is_file() {
                size += entry.metadata()?.len();
            }
        }
        Ok(size)
    } else {
        Ok(0)
    }
}

fn calculate_tool_total_size(
    package_manager: &crate::package_manager::PackageManager,
    tool_name: &str,
) -> Result<u64> {
    let versions = package_manager.list_versions(tool_name);
    let mut total_size = 0;

    for version in versions {
        if let Ok(path) = package_manager.get_version_path(tool_name, version) {
            total_size += calculate_size(&path).unwrap_or(0);
        }
    }

    Ok(total_size)
}

fn format_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", bytes, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}
