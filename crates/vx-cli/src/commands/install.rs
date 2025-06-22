//! Install command implementation

use crate::ui::UI;
use anyhow::Result;
use tracing::{info_span, Instrument};
use vx_plugin::PluginRegistry;

pub async fn handle(
    registry: &PluginRegistry,
    tool_name: &str,
    version: Option<&str>,
    force: bool,
) -> Result<()> {
    // Parse tool@version format if present
    let (actual_tool_name, parsed_version) = if tool_name.contains('@') {
        let parts: Vec<&str> = tool_name.splitn(2, '@').collect();
        if parts.len() == 2 {
            (parts[0], Some(parts[1]))
        } else {
            (tool_name, version)
        }
    } else {
        (tool_name, version)
    };

    // Get the tool from registry
    let tool = registry
        .get_tool(actual_tool_name)
        .ok_or_else(|| anyhow::anyhow!("Tool not found: {}", actual_tool_name))?;

    // Determine version to install (prefer parsed version over explicit version)
    let target_version = if let Some(v) = parsed_version.or(version) {
        v.to_string()
    } else {
        // Get latest version with progress span
        let span = info_span!("Fetching latest version", tool = actual_tool_name);
        let versions = async {
            UI::info(&format!(
                "Fetching latest version for {}...",
                actual_tool_name
            ));
            tool.fetch_versions(false).await
        }
        .instrument(span)
        .await?;

        if versions.is_empty() {
            return Err(anyhow::anyhow!(
                "No versions found for tool: {}",
                actual_tool_name
            ));
        }
        versions[0].version.clone()
    };

    UI::info(&format!(
        "Installing {} {}...",
        actual_tool_name, target_version
    ));

    // Check if already installed
    if !force && tool.is_version_installed(&target_version).await? {
        UI::success(&format!(
            "{} {} is already installed",
            actual_tool_name, target_version
        ));
        UI::hint("Use --force to reinstall");
        return Ok(());
    }

    // Install the version with progress span
    let install_span =
        info_span!("Installing tool", tool = actual_tool_name, version = %target_version);
    let install_result = async { tool.install_version(&target_version, force).await }
        .instrument(install_span)
        .await;

    match install_result {
        Ok(()) => {
            UI::success(&format!(
                "Successfully installed {} {}",
                actual_tool_name, target_version
            ));

            // Show installation path
            let install_dir = tool.get_version_install_dir(&target_version);
            UI::detail(&format!("Installed to: {}", install_dir.display()));

            // Show usage hint
            UI::hint(&format!(
                "Use 'vx {} --version' to verify installation",
                actual_tool_name
            ));
        }
        Err(e) => {
            UI::error(&format!(
                "Failed to install {} {}: {}",
                actual_tool_name, target_version, e
            ));
            return Err(e);
        }
    }

    Ok(())
}

/// Install multiple tools concurrently for better performance
pub async fn handle_batch(
    registry: &PluginRegistry,
    tool_specs: &[String],
    force: bool,
) -> Result<()> {
    if tool_specs.is_empty() {
        return Ok(());
    }

    UI::info(&format!(
        "Installing {} tools concurrently...",
        tool_specs.len()
    ));

    // Parse all tool specifications
    let mut install_tasks = Vec::new();
    for tool_spec in tool_specs {
        let (tool_name, version) = if tool_spec.contains('@') {
            let parts: Vec<&str> = tool_spec.splitn(2, '@').collect();
            if parts.len() == 2 {
                (parts[0], Some(parts[1]))
            } else {
                (tool_spec.as_str(), None)
            }
        } else {
            (tool_spec.as_str(), None)
        };

        // Get the tool from registry
        let tool = registry
            .get_tool(tool_name)
            .ok_or_else(|| anyhow::anyhow!("Tool not found: {}", tool_name))?;

        install_tasks.push((tool_name.to_string(), tool, version.map(|s| s.to_string())));
    }

    // Create concurrent installation futures
    let install_futures = install_tasks
        .into_iter()
        .map(|(tool_name, tool, version)| async move {
            let result = handle_single_tool_install(&tool_name, tool, version, force).await;
            (tool_name, result)
        });

    // Execute all installations concurrently
    let results = futures::future::join_all(install_futures).await;

    // Report results
    let mut success_count = 0;
    let mut failed_tools = Vec::new();

    for (tool_name, result) in results {
        match result {
            Ok(()) => {
                success_count += 1;
                UI::success(&format!("✓ {}", tool_name));
            }
            Err(e) => {
                let error_msg = e.to_string();
                failed_tools.push((tool_name.clone(), e));
                UI::error(&format!("✗ {}: {}", tool_name, error_msg));
            }
        }
    }

    // Summary
    if failed_tools.is_empty() {
        UI::success(&format!(
            "Successfully installed all {} tools concurrently!",
            success_count
        ));
    } else {
        UI::warning(&format!(
            "Installed {}/{} tools. {} failed:",
            success_count,
            success_count + failed_tools.len(),
            failed_tools.len()
        ));
        for (tool_name, _) in &failed_tools {
            UI::error(&format!("  - {}", tool_name));
        }
    }

    Ok(())
}

/// Helper function to install a single tool (used by batch installer)
async fn handle_single_tool_install(
    tool_name: &str,
    tool: Box<dyn vx_plugin::VxTool>,
    version: Option<String>,
    force: bool,
) -> Result<()> {
    // Determine version to install
    let target_version = if let Some(v) = version {
        v
    } else {
        // Get latest version
        let versions = tool.fetch_versions(false).await?;
        if versions.is_empty() {
            return Err(anyhow::anyhow!("No versions found for tool: {}", tool_name));
        }
        versions[0].version.clone()
    };

    // Check if already installed
    if !force && tool.is_version_installed(&target_version).await? {
        return Ok(()); // Already installed, skip
    }

    // Install the version
    tool.install_version(&target_version, force).await?;
    Ok(())
}
