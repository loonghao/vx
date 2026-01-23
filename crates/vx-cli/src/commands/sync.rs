//! Sync command implementation
//!
//! Synchronizes project tools from vx.toml configuration.
//! This is the core tool installation logic, also used by `vx setup`.
//!
//! When a `vx.lock` file exists, the sync command will use the exact
//! versions specified in the lock file for reproducible environments.

use crate::commands::setup::{find_vx_config, parse_vx_config};
use crate::ui::UI;
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::env;
use std::path::PathBuf;
use std::process::Command;
use vx_paths::project::LOCK_FILE_NAME;
use vx_paths::PathManager;
use vx_resolver::LockFile;
use vx_runtime::ProviderRegistry;

/// Tool status tuple: (name, version, installed, path)
type ToolStatusTuple = (String, String, bool, Option<PathBuf>);

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
    let statuses = check_tool_status(&effective_tools)?;

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
    let successful = results.iter().filter(|(_, ok, _)| *ok).count();
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

        // Show detailed error information for failed tools
        println!();
        UI::error("Failed installations:");
        for (result, tool) in results.iter().zip(missing.iter()) {
            if !result.1 {
                println!("  ✗ {}@{}", tool.0, tool.1);
                if let Some(error) = &result.2 {
                    // Show error details, indented
                    for line in error.lines().take(5) {
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

/// Install tools sequentially
async fn install_sequential(
    tools: &[&(String, String, bool, Option<PathBuf>)],
    verbose: bool,
) -> Result<Vec<InstallResult>> {
    let mut results = Vec::new();

    for (name, version, _, _) in tools {
        if verbose {
            UI::info(&format!("Installing {}@{}...", name, version));
        }

        let (success, error) = install_tool(name, version).await;
        results.push((name.clone(), success, error.clone()));

        if success {
            if verbose {
                UI::success(&format!("  ✓ {}@{}", name, version));
            }
        } else {
            UI::error(&format!("  ✗ {}@{}", name, version));
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

/// Install tools in parallel
async fn install_parallel(
    tools: &[&(String, String, bool, Option<PathBuf>)],
    _verbose: bool,
) -> Result<Vec<InstallResult>> {
    use tokio::task::JoinSet;

    let mut join_set = JoinSet::new();

    for (name, version, _, _) in tools {
        let name = name.clone();
        let version = version.clone();

        join_set.spawn(async move {
            let (success, error) = install_tool(&name, &version).await;
            (name, success, error)
        });
    }

    let mut results = Vec::new();
    while let Some(result) = join_set.join_next().await {
        if let Ok((name, success, error)) = result {
            let icon = if success { "✓" } else { "✗" };
            println!("  {} {}", icon, name);
            results.push((name, success, error));
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

    let statuses = check_tool_status(&effective_tools)?;
    let all_installed = statuses.iter().all(|(_, _, installed, _)| *installed);

    Ok(all_installed)
}
