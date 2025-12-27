//! Sync command implementation
//!
//! Synchronizes project tools from .vx.toml configuration.
//! This is the core tool installation logic, also used by `vx setup`.

use crate::commands::setup::{find_vx_config, parse_vx_config};
use crate::ui::UI;
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::env;
use std::path::PathBuf;
use std::process::Command;
use vx_paths::PathManager;
use vx_runtime::ProviderRegistry;

/// Tool status tuple: (name, version, installed, path)
type ToolStatusTuple = (String, String, bool, Option<PathBuf>);

/// Handle the sync command
pub async fn handle(
    _registry: &ProviderRegistry,
    check: bool,
    force: bool,
    dry_run: bool,
    verbose: bool,
    no_parallel: bool,
    _no_auto_install: bool,
) -> Result<()> {
    let current_dir = env::current_dir().context("Failed to get current directory")?;

    // Find .vx.toml
    let config_path = find_vx_config(&current_dir)?;
    let config = parse_vx_config(&config_path)?;

    if config.tools.is_empty() {
        UI::info("No tools configured in .vx.toml");
        return Ok(());
    }

    // Check tool status
    let statuses = check_tool_status(&config.tools)?;

    // Show status
    if verbose || check {
        println!("Project tools:");
        for (name, version, installed, path) in &statuses {
            let status_icon = if *installed { "✓" } else { "✗" };
            let status_text = if *installed {
                format!(
                    "installed at {}",
                    path.as_ref()
                        .map(|p| p.display().to_string())
                        .unwrap_or_default()
                )
            } else {
                "missing".to_string()
            };
            println!("  {} {}@{} ({})", status_icon, name, version, status_text);
        }
        println!();
    }

    // Count missing tools
    let missing: Vec<_> = statuses
        .iter()
        .filter(|(_, _, installed, _)| !installed || force)
        .collect();

    if missing.is_empty() {
        UI::success("All tools are synchronized");
        return Ok(());
    }

    if check {
        UI::warn(&format!("{} tool(s) need to be installed", missing.len()));
        UI::hint("Run 'vx sync' or 'vx setup' to install missing tools");
        return Ok(());
    }

    if dry_run {
        UI::info(&format!("Would install {} tool(s):", missing.len()));
        for (name, version, _, _) in &missing {
            println!("  - {}@{}", name, version);
        }
        return Ok(());
    }

    // Install missing tools
    UI::info(&format!("Installing {} tool(s)...", missing.len()));

    let results = if no_parallel {
        install_sequential(&missing, verbose).await?
    } else {
        install_parallel(&missing, verbose).await?
    };

    // Show results
    let successful = results.iter().filter(|(_, ok)| *ok).count();
    let failed = results.len() - successful;

    if failed == 0 {
        UI::success(&format!("Successfully synchronized {} tool(s)", successful));
    } else {
        UI::warn(&format!(
            "Synchronized {}/{} tools ({} failed)",
            successful,
            results.len(),
            failed
        ));
        for (result, tool) in results.iter().zip(missing.iter()) {
            if !result.1 {
                UI::error(&format!("  Failed: {}@{}", tool.0, tool.1));
            }
        }
    }

    Ok(())
}

/// Check the installation status of all tools
fn check_tool_status(tools: &HashMap<String, String>) -> Result<Vec<ToolStatusTuple>> {
    let path_manager = PathManager::new()?;
    let mut statuses = Vec::new();

    for (name, version) in tools {
        let (installed, path) = if version == "latest" {
            // For latest, check if any version is installed
            let versions = path_manager.list_store_versions(name)?;
            if let Some(latest) = versions.last() {
                let store_path = path_manager.version_store_dir(name, latest);
                (true, Some(store_path))
            } else {
                (false, None)
            }
        } else {
            let store_path = path_manager.version_store_dir(name, version);
            (store_path.exists(), Some(store_path))
        };

        statuses.push((
            name.clone(),
            version.clone(),
            installed,
            if installed { path } else { None },
        ));
    }

    // Sort by name
    statuses.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(statuses)
}

/// Install tools sequentially
async fn install_sequential(
    tools: &[&(String, String, bool, Option<PathBuf>)],
    verbose: bool,
) -> Result<Vec<(String, bool)>> {
    let mut results = Vec::new();

    for (name, version, _, _) in tools {
        if verbose {
            UI::info(&format!("Installing {}@{}...", name, version));
        }

        let success = install_tool(name, version).await;
        results.push((name.clone(), success));

        if success {
            if verbose {
                UI::success(&format!("  ✓ {}@{}", name, version));
            }
        } else {
            UI::error(&format!("  ✗ {}@{}", name, version));
        }
    }

    Ok(results)
}

/// Install tools in parallel
async fn install_parallel(
    tools: &[&(String, String, bool, Option<PathBuf>)],
    _verbose: bool,
) -> Result<Vec<(String, bool)>> {
    use tokio::task::JoinSet;

    let mut join_set = JoinSet::new();

    for (name, version, _, _) in tools {
        let name = name.clone();
        let version = version.clone();

        join_set.spawn(async move {
            let success = install_tool(&name, &version).await;
            (name, success)
        });
    }

    let mut results = Vec::new();
    while let Some(result) = join_set.join_next().await {
        if let Ok((name, success)) = result {
            let icon = if success { "✓" } else { "✗" };
            println!("  {} {}", icon, name);
            results.push((name, success));
        }
    }

    Ok(results)
}

/// Install a single tool
async fn install_tool(name: &str, version: &str) -> bool {
    let exe = match env::current_exe() {
        Ok(e) => e,
        Err(_) => return false,
    };

    let mut cmd = Command::new(exe);
    cmd.args(["install", name, version]);

    // Suppress output
    cmd.stdout(std::process::Stdio::null());
    cmd.stderr(std::process::Stdio::null());

    match cmd.status() {
        Ok(status) => status.success(),
        Err(_) => false,
    }
}

/// Quick check if project is in sync (for shell integration)
pub async fn quick_check() -> Result<bool> {
    let current_dir = env::current_dir()?;

    let config_path = match find_vx_config(&current_dir) {
        Ok(p) => p,
        Err(_) => return Ok(true), // No config = in sync
    };

    let config = parse_vx_config(&config_path)?;

    if config.tools.is_empty() {
        return Ok(true);
    }

    let statuses = check_tool_status(&config.tools)?;
    let all_installed = statuses.iter().all(|(_, _, installed, _)| *installed);

    Ok(all_installed)
}
