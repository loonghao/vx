//! Sync command implementation
//!
//! Synchronizes project tools from vx.toml configuration.
//! This is the core tool installation logic, also used by `vx setup`.
//!
//! When a `vx.lock` file exists, the sync command will use the exact
//! versions specified in the lock file for reproducible environments.

use crate::commands::common::{check_tools_status, ToolStatus};
use crate::commands::setup::{find_vx_config, parse_vx_config};
use crate::ui::{InstallProgress, UI};
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::env;
use std::path::PathBuf;
use std::process::Command;
use vx_paths::project::LOCK_FILE_NAME;
use vx_resolver::LockFile;
use vx_runtime::ProviderRegistry;

/// Install result with optional error message
type InstallResult = (String, bool, Option<String>);

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

    // Find vx.toml
    let config_path = find_vx_config(&current_dir)?;
    let config = parse_vx_config(&config_path)?;

    if config.tools.is_empty() {
        UI::info("No tools configured in vx.toml");
        return Ok(());
    }

    // Check for lock file
    let project_root = config_path.parent().unwrap_or(&current_dir);
    let lock_path = project_root.join(LOCK_FILE_NAME);
    let lockfile = if lock_path.exists() {
        match LockFile::load(&lock_path) {
            Ok(lf) => {
                if verbose {
                    UI::info(&format!("Using versions from {}", LOCK_FILE_NAME));
                }
                Some(lf)
            }
            Err(e) => {
                UI::warn(&format!("Failed to load {}: {}", LOCK_FILE_NAME, e));
                UI::hint("Run 'vx lock' to regenerate the lock file");
                None
            }
        }
    } else {
        if verbose {
            UI::detail(&format!(
                "No {} found, using versions from vx.toml",
                LOCK_FILE_NAME
            ));
        }
        None
    };

    // Resolve effective versions (lock file takes precedence)
    let effective_tools = resolve_effective_versions(&config.tools, &lockfile);

    // Check tool status
    let statuses = check_tools_status(&effective_tools)?;

    // Show status
    if verbose || check {
        println!("Project tools:");
        for (name, version, status, path, _) in &statuses {
            let installed = matches!(status, ToolStatus::Installed | ToolStatus::SystemFallback);
            let status_icon = if installed { "✓" } else { "✗" };
            let status_text = match status {
                ToolStatus::Installed => format!(
                    "installed at {}",
                    path.as_ref()
                        .map(|p| p.display().to_string())
                        .unwrap_or_default()
                ),
                ToolStatus::SystemFallback => "system".to_string(),
                ToolStatus::NotInstalled => "missing".to_string(),
            };
            println!("  {} {}@{} ({})", status_icon, name, version, status_text);
        }
        println!();
    }

    // Count missing tools
    let missing: Vec<_> = statuses
        .iter()
        .filter(|(_, _, status, _, _)| matches!(status, ToolStatus::NotInstalled) || force)
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
        for (name, version, _, _, _) in &missing {
            println!("  - {}@{}", name, version);
        }
        return Ok(());
    }

    // Install missing tools using InstallProgress for unified progress display
    let mut progress = InstallProgress::new(
        missing.len(),
        &format!("Installing {} tool(s)", missing.len()),
    );

    let results = if no_parallel {
        install_sequential_with_progress(&missing, verbose, &mut progress).await?
    } else {
        install_parallel_with_progress(&missing, verbose, &mut progress).await?
    };

    // Finish progress
    let successful = results.iter().filter(|(_, ok, _)| *ok).count();
    let failed = results.len() - successful;

    if failed == 0 {
        progress.finish(&format!("✓ Successfully synchronized {} tool(s)", successful));
    } else {
        progress.finish(&format!(
            "⚠ Synchronized {}/{} tools ({} failed)",
            successful,
            results.len(),
            failed
        ));
    }

    // Show detailed error information for failed tools
    if failed > 0 {
        println!();
        UI::error("Failed installations:");
        for (name, success, error) in &results {
            if !success {
                println!("  ✗ {}", name);
                if let Some(err) = error {
                    // Show error details, indented
                    for line in err.lines().take(5) {
                        // Skip empty lines and spinner characters
                        let trimmed = line.trim();
                        if !trimmed.is_empty() && !trimmed.starts_with('�') {
                            println!("    {}", trimmed);
                        }
                    }
                }
            }
        }

        println!();
        UI::hint("Run 'vx install <tool> <version>' for more details on specific failures");
    }

    Ok(())
}

/// Resolve effective versions by preferring lock file versions over config versions
fn resolve_effective_versions(
    config_tools: &HashMap<String, String>,
    lockfile: &Option<LockFile>,
) -> HashMap<String, String> {
    let mut effective = HashMap::new();

    for (name, config_version) in config_tools {
        let version = if let Some(lock) = lockfile {
            // If tool is in lock file, use the locked version
            if let Some(locked_tool) = lock.get_tool(name) {
                locked_tool.version.clone()
            } else {
                // Tool not in lock file, use config version
                config_version.clone()
            }
        } else {
            // No lock file, use config version
            config_version.clone()
        };

        effective.insert(name.clone(), version);
    }

    effective
}

/// Install tools sequentially with progress display
async fn install_sequential_with_progress(
    tools: &[&(String, String, ToolStatus, Option<PathBuf>, Option<String>)],
    verbose: bool,
    progress: &mut InstallProgress,
) -> Result<Vec<InstallResult>> {
    let mut results = Vec::new();

    for (name, version, _, _, _) in tools {
        progress.start_tool(name, version);

        let (success, error) = install_tool(name, version).await;
        results.push((name.clone(), success, error.clone()));

        progress.complete_tool(success, name, version);

        if verbose && !success {
            if let Some(err) = &error {
                // Show first line of error for brief context
                if let Some(first_line) = err.lines().next() {
                    UI::detail(&format!("    {}", first_line));
                }
            }
        }
    }

    Ok(results)
}

/// Install tools in parallel with progress display
async fn install_parallel_with_progress(
    tools: &[&(String, String, ToolStatus, Option<PathBuf>, Option<String>)],
    _verbose: bool,
    progress: &mut InstallProgress,
) -> Result<Vec<InstallResult>> {
    use tokio::task::JoinSet;

    let mut join_set = JoinSet::new();

    // Start all installations
    for (name, version, _, _, _) in tools {
        let name = name.clone();
        let version = version.clone();

        progress.start_tool(&name, &version);

        join_set.spawn(async move {
            let (success, error) = install_tool(&name, &version).await;
            (name, version, success, error)
        });
    }

    let mut results = Vec::new();
    while let Some(result) = join_set.join_next().await {
        if let Ok((name, version, success, error)) = result {
            results.push((name.clone(), success, error.clone()));
            progress.complete_tool(success, &name, &version);
        }
    }

    Ok(results)
}

/// Install a single tool, returns (success, error_message)
async fn install_tool(name: &str, version: &str) -> (bool, Option<String>) {
    let exe = match env::current_exe() {
        Ok(e) => e,
        Err(e) => return (false, Some(format!("Failed to get current exe: {}", e))),
    };

    let mut cmd = Command::new(exe);
    // Use tool@version format instead of separate arguments
    cmd.args(["install", &format!("{}@{}", name, version)]);

    // Capture output instead of suppressing it
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());

    match cmd.output() {
        Ok(output) => {
            if output.status.success() {
                (true, None)
            } else {
                // Combine stderr and stdout for error context
                let stderr = String::from_utf8_lossy(&output.stderr);
                let stdout = String::from_utf8_lossy(&output.stdout);
                let error_msg = if !stderr.is_empty() {
                    stderr.to_string()
                } else if !stdout.is_empty() {
                    stdout.to_string()
                } else {
                    format!(
                        "Install command failed with exit code: {:?}",
                        output.status.code()
                    )
                };
                (false, Some(error_msg))
            }
        }
        Err(e) => (
            false,
            Some(format!("Failed to execute install command: {}", e)),
        ),
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

    // Check for lock file
    let project_root = config_path.parent().unwrap_or(&current_dir);
    let lock_path = project_root.join(LOCK_FILE_NAME);
    let lockfile = if lock_path.exists() {
        LockFile::load(&lock_path).ok()
    } else {
        None
    };

    // Resolve effective versions
    let effective_tools = resolve_effective_versions(&config.tools, &lockfile);

    let statuses = check_tools_status(&effective_tools)?;
    let all_installed = statuses
        .iter()
        .all(|(_, _, status, _, _)| matches!(status, ToolStatus::Installed | ToolStatus::SystemFallback));

    Ok(all_installed)
}
