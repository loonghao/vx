// Update command implementation
// Updates tools to latest versions

use crate::ui::UI;
use crate::{with_progress_events, with_progress_span};
use anyhow::Result;

pub async fn handle(tool: Option<String>, check: bool, version: Option<String>) -> Result<()> {
    match tool {
        Some(tool_name) => {
            if check {
                check_tool_updates(&tool_name).await
            } else {
                update_tool(&tool_name, version).await
            }
        }
        None => {
            if check {
                check_all_updates().await
            } else {
                update_all_tools().await
            }
        }
    }
}

async fn update_tool(tool_name: &str, target_version: Option<String>) -> Result<()> {
    UI::step(&format!("Updating {}...", tool_name));

    let package_manager = crate::package_manager::PackageManager::new()?;
    let installed_versions = package_manager.list_versions(tool_name);

    if installed_versions.is_empty() {
        UI::warning(&format!("Tool '{}' is not installed", tool_name));
        UI::hint(&format!("Install with: vx install {}", tool_name));
        return Ok(());
    }

    // Get current version and latest version with progress
    let (current_version, latest_version) = with_progress_span!("check_versions", async {
        let current_version = get_current_version(tool_name).await;
        let latest_version = get_latest_version(tool_name).await?;
        Ok::<(Option<String>, String), anyhow::Error>((current_version, latest_version))
    })
    .await?;

    // Determine target version
    let target = target_version.unwrap_or_else(|| "latest".to_string());

    let version_to_install = if target == "latest" {
        latest_version.clone()
    } else {
        target
    };

    // Check if we need to update
    if let Some(current) = &current_version {
        if current == &version_to_install {
            UI::success(&format!(
                "{} is already up to date ({})",
                tool_name, current
            ));
            return Ok(());
        }

        UI::info(&format!("Current version: {}", current));
    }

    UI::info(&format!("Target version: {}", version_to_install));

    // Check if target version is already installed
    let version_strings: Vec<String> = installed_versions
        .iter()
        .map(|p| p.version.clone())
        .collect();
    if version_strings.contains(&version_to_install) {
        UI::info(&format!(
            "Version {} is already installed",
            version_to_install
        ));

        // Switch to this version
        let tool_version = format!("{}@{}", tool_name, version_to_install);
        crate::cli::switch::handle(tool_version, false).await?;

        UI::success(&format!("Switched to {} {}", tool_name, version_to_install));
        return Ok(());
    }

    // Clone values for use in async block
    let tool_name_clone = tool_name.to_string();
    let version_clone = version_to_install.clone();

    // Install the new version with progress
    let path = with_progress_events!(
        "install_new_version",
        "Update completed successfully",
        "Update failed",
        async move {
            let mut executor = crate::executor::Executor::new()?;
            executor
                .install_tool(&tool_name_clone, &version_clone)
                .await
        }
    )
    .await?;

    UI::info(&format!("Installed to: {}", path.display()));

    // Switch to the new version
    let tool_version = format!("{}@{}", tool_name, version_to_install);
    with_progress_span!(
        "switch_version",
        crate::cli::switch::handle(tool_version, false)
    )
    .await?;

    Ok(())
}

async fn update_all_tools() -> Result<()> {
    UI::header("Updating all installed tools");

    let package_manager = crate::package_manager::PackageManager::new()?;
    let installed_tools = package_manager.list_installed_tools()?;

    if installed_tools.is_empty() {
        UI::info("No tools are currently installed");
        return Ok(());
    }

    let mut updated_count = 0;
    let mut failed_count = 0;

    for tool_name in installed_tools {
        println!();
        match update_tool(&tool_name, None).await {
            Ok(()) => updated_count += 1,
            Err(e) => {
                UI::error(&format!("Failed to update {tool_name}: {e}"));
                failed_count += 1;
            }
        }
    }

    println!();
    UI::header("Update Summary");
    UI::success(&format!("Updated: {updated_count} tools"));
    if failed_count > 0 {
        UI::warning(&format!("Failed: {failed_count} tools"));
    }

    Ok(())
}

async fn check_tool_updates(tool_name: &str) -> Result<()> {
    UI::step(&format!("Checking for {tool_name} updates..."));

    let current_version = get_current_version(tool_name).await;
    let latest_version = get_latest_version(tool_name).await?;

    match current_version {
        Some(current) => {
            if current == latest_version {
                UI::success(&format!("{tool_name} is up to date ({current})"));
            } else {
                UI::info(&format!("Update available for {tool_name}:"));
                println!("  Current: {}", current);
                println!("  Latest:  {}", latest_version);
                UI::hint(&format!("Update with: vx update {tool_name}"));
            }
        }
        None => {
            UI::warning(&format!("Tool '{tool_name}' is not installed"));
            UI::hint(&format!("Install with: vx install {tool_name}"));
        }
    }

    Ok(())
}

async fn check_all_updates() -> Result<()> {
    UI::header("Checking for updates");

    let package_manager = crate::package_manager::PackageManager::new()?;
    let installed_tools = package_manager.list_installed_tools()?;

    if installed_tools.is_empty() {
        UI::info("No tools are currently installed");
        return Ok(());
    }

    let mut updates_available = Vec::new();
    let mut up_to_date = Vec::new();
    let mut check_failed = Vec::new();

    for tool_name in installed_tools {
        match (
            get_current_version(&tool_name).await,
            get_latest_version(&tool_name).await,
        ) {
            (Some(current), Ok(latest)) => {
                if current == latest {
                    up_to_date.push((tool_name, current));
                } else {
                    updates_available.push((tool_name, current, latest));
                }
            }
            (None, _) => {
                check_failed.push((tool_name, "Not in PATH".to_string()));
            }
            (_, Err(e)) => {
                check_failed.push((tool_name, e.to_string()));
            }
        }
    }

    if !updates_available.is_empty() {
        UI::header("Updates Available");
        for (tool, current, latest) in &updates_available {
            println!("  {} {} → {}", tool, current, latest);
        }
        println!();
        UI::hint("Update all with: vx update");
        UI::hint("Update specific tool with: vx update <tool>");
    }

    if !up_to_date.is_empty() {
        UI::header("Up to Date");
        for (tool, version) in &up_to_date {
            println!("  {} {}", tool, version);
        }
    }

    if !check_failed.is_empty() {
        UI::header("Check Failed");
        for (tool, reason) in &check_failed {
            println!("  {} ({})", tool, reason);
        }
    }

    Ok(())
}

async fn get_current_version(tool_name: &str) -> Option<String> {
    // Try to get version from system PATH
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

    None
}

async fn get_latest_version(tool_name: &str) -> Result<String> {
    match tool_name {
        "uv" => {
            let version = crate::version::VersionManager::get_latest_version("uv").await?;
            Ok(version.as_string())
        }
        "node" => {
            let version = crate::version::VersionManager::get_latest_version("node").await?;
            Ok(version.as_string())
        }
        "go" => {
            // For now, return a placeholder
            // TODO: Implement actual Go version checking
            Ok("1.21.6".to_string())
        }
        "rust" => {
            // For now, return a placeholder
            // TODO: Implement actual Rust version checking
            Ok("1.75.0".to_string())
        }
        _ => Err(anyhow::anyhow!(
            "Version checking not implemented for {tool_name}"
        )),
    }
}
