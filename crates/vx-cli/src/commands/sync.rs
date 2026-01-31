//! Sync command implementation
//!
//! Synchronizes project tools from vx.toml configuration.
//! This is the core tool installation logic, also used by `vx setup`.
//!
//! When a `vx.lock` file exists, the sync command will use the exact
//! versions specified in the lock file for reproducible environments.
//!
//! ## Lock File Behavior
//!
//! - If `vx.lock` exists and is consistent: use locked versions
//! - If `vx.lock` exists but is inconsistent: warn and suggest `vx lock`
//! - If `vx.lock` doesn't exist and `--auto-lock` is set: generate it automatically
//! - If `vx.lock` doesn't exist: use versions from vx.toml

use crate::commands::common::{check_tools_status_ordered, ToolStatus};
use crate::commands::setup::{find_vx_config, parse_vx_config};
use crate::ui::{InstallProgress, UI};
use anyhow::{Context, Result};
use std::collections::BTreeMap;
use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;
use vx_paths::project::LOCK_FILE_NAME;
use vx_project_analyzer::{AnalyzerConfig, ProjectAnalyzer};
use vx_resolver::{LockFile, LockFileInconsistency};
use vx_runtime::ProviderRegistry;

/// Type alias for complex tool tuple reference
type ToolInfoRef<'a> = &'a (String, String, ToolStatus, Option<PathBuf>, Option<String>);

/// Install result with optional error message
type InstallResult = (String, bool, Option<String>);

/// Lock file status check result
enum LockStatus {
    /// Lock file is up to date
    UpToDate(LockFile),
    /// Lock file has inconsistencies
    NeedsUpdate(LockFile, Vec<LockFileInconsistency>),
    /// Lock file doesn't exist
    NotFound,
    /// Lock file failed to load
    LoadError(String),
}

/// Check lock file status against config
fn check_lock_status(
    lock_path: &std::path::Path,
    config_tools: &BTreeMap<String, String>,
) -> LockStatus {
    if !lock_path.exists() {
        return LockStatus::NotFound;
    }

    match LockFile::load(lock_path) {
        Ok(lockfile) => {
            let inconsistencies = lockfile.check_consistency(config_tools);
            if inconsistencies.is_empty() {
                LockStatus::UpToDate(lockfile)
            } else {
                LockStatus::NeedsUpdate(lockfile, inconsistencies)
            }
        }
        Err(e) => LockStatus::LoadError(e.to_string()),
    }
}

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
    handle_with_options(
        _registry,
        SyncOptions {
            check,
            force,
            dry_run,
            verbose,
            no_parallel,
            auto_lock: false, // Default behavior
            analyze: true,    // Enable project analysis by default
        },
    )
    .await
}

/// Sync options
pub struct SyncOptions {
    /// Only check, don't install
    pub check: bool,
    /// Force reinstall all tools
    pub force: bool,
    /// Preview operations without executing
    pub dry_run: bool,
    /// Show verbose output
    pub verbose: bool,
    /// Disable parallel installation
    pub no_parallel: bool,
    /// Automatically generate/update lock file if needed
    pub auto_lock: bool,
    /// Analyze project files for additional tools (e.g., detect just from Justfile)
    pub analyze: bool,
}

/// Handle the sync command with options
pub async fn handle_with_options(_registry: &ProviderRegistry, options: SyncOptions) -> Result<()> {
    let current_dir = env::current_dir().context("Failed to get current directory")?;

    // Find vx.toml
    let config_path = find_vx_config(&current_dir)?;
    let config = parse_vx_config(&config_path)?;

    if config.tools.is_empty() {
        UI::info("No tools configured in vx.toml");
        return Ok(());
    }

    // Get tools as BTreeMap for deterministic ordering
    let config_tools = config.tools_as_btreemap();

    // Check lock file status
    let project_root = config_path.parent().unwrap_or(&current_dir);
    let lock_path = project_root.join(LOCK_FILE_NAME);
    let lock_status = check_lock_status(&lock_path, &config_tools);

    let lockfile = match lock_status {
        LockStatus::UpToDate(lf) => {
            if options.verbose {
                UI::info(&format!("Using versions from {}", LOCK_FILE_NAME));
            }

            // RFC 0023: Check version range warnings
            check_version_ranges(&lf, &config_tools, options.verbose);

            Some(lf)
        }
        LockStatus::NeedsUpdate(lf, inconsistencies) => {
            UI::warn(&format!("{} is out of sync with vx.toml:", LOCK_FILE_NAME));
            for inc in &inconsistencies {
                UI::detail(&format!("  - {}", inc));
            }

            if options.auto_lock {
                UI::info("Auto-updating lock file...");
                // Run vx lock to update
                run_lock_command()?;
                // Reload the updated lock file
                match LockFile::load(&lock_path) {
                    Ok(updated_lf) => Some(updated_lf),
                    Err(_) => {
                        UI::warn("Failed to reload lock file, using config versions");
                        None
                    }
                }
            } else {
                UI::hint("Run 'vx lock' to update, or use 'vx sync --auto-lock'");
                // Use existing lock file but warn
                Some(lf)
            }
        }
        LockStatus::NotFound => {
            if options.auto_lock && !config.tools.is_empty() {
                UI::info("No lock file found, generating...");
                run_lock_command()?;
                // Load the newly generated lock file
                match LockFile::load(&lock_path) {
                    Ok(lf) => {
                        UI::success(&format!("Generated {}", LOCK_FILE_NAME));
                        Some(lf)
                    }
                    Err(_) => {
                        UI::warn("Failed to load generated lock file, using config versions");
                        None
                    }
                }
            } else {
                if options.verbose {
                    UI::detail(&format!(
                        "No {} found, using versions from vx.toml",
                        LOCK_FILE_NAME
                    ));
                }
                None
            }
        }
        LockStatus::LoadError(e) => {
            UI::warn(&format!("Failed to load {}: {}", LOCK_FILE_NAME, e));
            UI::hint("Run 'vx lock' to regenerate the lock file");
            None
        }
    };

    // Resolve effective versions (lock file takes precedence)
    let mut effective_tools = resolve_effective_versions(&config_tools, &lockfile);

    // Analyze project files for additional required tools if enabled
    if options.analyze {
        let analyzed_tools = analyze_project_tools(project_root, options.verbose).await?;
        #[allow(clippy::map_entry)]
        for (name, version) in analyzed_tools {
            if !effective_tools.contains_key(&name) {
                if options.verbose {
                    UI::info(&format!("Detected {} from project analysis", name));
                }
                effective_tools.insert(name, version);
            }
        }
    }

    // Check tool status
    let statuses = check_tools_status_ordered(&effective_tools)?;

    // Show status
    if options.verbose || options.check {
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
        .filter(|(_, _, status, _, _)| matches!(status, ToolStatus::NotInstalled) || options.force)
        .collect();

    if missing.is_empty() {
        UI::success("All tools are synchronized");
        return Ok(());
    }

    if options.check {
        UI::warn(&format!("{} tool(s) need to be installed", missing.len()));
        UI::hint("Run 'vx sync' or 'vx setup' to install missing tools");
        return Ok(());
    }

    if options.dry_run {
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

    let results = if options.no_parallel {
        install_sequential_with_progress(&missing, options.verbose, &mut progress).await?
    } else {
        install_parallel_with_progress(&missing, options.verbose, &mut progress).await?
    };

    // Finish progress
    let successful = results.iter().filter(|(_, ok, _)| *ok).count();
    let failed = results.len() - successful;

    if failed == 0 {
        progress.finish(&format!(
            "✓ Successfully synchronized {} tool(s)",
            successful
        ));
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

/// Run `vx lock` command to generate/update lock file
fn run_lock_command() -> Result<()> {
    let exe = env::current_exe().context("Failed to get current exe")?;

    let output = Command::new(exe)
        .args(["lock"])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .output()
        .context("Failed to run vx lock")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("vx lock failed: {}", stderr);
    }

    Ok(())
}

/// Resolve effective versions by preferring lock file versions over config versions
fn resolve_effective_versions(
    config_tools: &BTreeMap<String, String>,
    lockfile: &Option<LockFile>,
) -> BTreeMap<String, String> {
    let mut effective = BTreeMap::new();

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
    tools: &[ToolInfoRef<'_>],
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
    tools: &[ToolInfoRef<'_>],
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

    // Get tools as BTreeMap for deterministic ordering
    let config_tools = config.tools_as_btreemap();

    // Check for lock file
    let project_root = config_path.parent().unwrap_or(&current_dir);
    let lock_path = project_root.join(LOCK_FILE_NAME);
    let lockfile = if lock_path.exists() {
        LockFile::load(&lock_path).ok()
    } else {
        None
    };

    // Resolve effective versions
    let effective_tools = resolve_effective_versions(&config_tools, &lockfile);

    let statuses = check_tools_status_ordered(&effective_tools)?;
    let all_installed = statuses.iter().all(|(_, _, status, _, _)| {
        matches!(status, ToolStatus::Installed | ToolStatus::SystemFallback)
    });

    Ok(all_installed)
}

/// Analyze project files to detect additional required tools
///
/// This function uses the ProjectAnalyzer to detect tools needed by the project
/// that may not be explicitly listed in vx.toml. For example:
/// - `just` if a Justfile exists
/// - Package managers based on lock files
/// - Build tools based on project configuration
///
/// **Important:** Only tools with `InstallMethod::vx()` are added to the result.
/// Tools that should be installed via other methods (npm, pip, cargo, etc.) are
/// filtered out to avoid attempting to install unsupported tools.
async fn analyze_project_tools(
    project_root: &Path,
    verbose: bool,
) -> Result<BTreeMap<String, String>> {
    let mut detected_tools = BTreeMap::new();

    let analyzer_config = AnalyzerConfig {
        check_installed: false, // We just want to detect, not check installation
        check_tools: true,
        generate_sync_actions: false,
        max_depth: 1,
    };

    let analyzer = ProjectAnalyzer::new(analyzer_config);

    match analyzer.analyze(project_root).await {
        Ok(analysis) => {
            // Extract required tools from analysis
            for tool in &analysis.required_tools {
                // Only add tools that should be installed via vx
                // Skip tools that should be installed via npm, pip, cargo, etc.
                let should_install_via_vx = matches!(
                    &tool.install_method,
                    vx_project_analyzer::InstallMethod::Vx { .. }
                );

                if !should_install_via_vx {
                    if verbose {
                        UI::detail(&format!(
                            "  Skipping {}: should be installed via {}",
                            tool.name,
                            match &tool.install_method {
                                vx_project_analyzer::InstallMethod::Npm { .. } => "npm",
                                vx_project_analyzer::InstallMethod::Uv { .. } => "uv",
                                vx_project_analyzer::InstallMethod::Pip { .. } => "pip",
                                vx_project_analyzer::InstallMethod::Cargo { .. } => "cargo",
                                vx_project_analyzer::InstallMethod::Go { .. } => "go install",
                                vx_project_analyzer::InstallMethod::Manual { .. } => "manual",
                                vx_project_analyzer::InstallMethod::System { .. } =>
                                    "system package manager",
                                _ => "other",
                            }
                        ));
                    }
                    continue;
                }

                // Use "latest" as default version for detected tools
                // unless the tool has a specific version requirement
                let version = "latest".to_string();

                if verbose {
                    UI::detail(&format!(
                        "  Project analysis detected: {} ({})",
                        tool.name,
                        if tool.reason.is_empty() {
                            "auto-detected"
                        } else {
                            &tool.reason
                        }
                    ));
                }

                detected_tools.insert(tool.name.clone(), version);
            }
        }
        Err(e) => {
            if verbose {
                UI::warn(&format!("Project analysis failed: {}", e));
            }
        }
    }

    Ok(detected_tools)
}

/// Check version ranges for RFC 0023 compliance
///
/// This function checks if locked versions are still within their original ranges
/// and warns the user if updates are available within the range.
fn check_version_ranges(
    lockfile: &LockFile,
    _config_tools: &BTreeMap<String, String>,
    verbose: bool,
) {
    let mut outdated_in_range = Vec::new();

    for (name, locked) in &lockfile.tools {
        // Check if the locked version is marked as latest in range
        if let Some(false) = locked.is_latest_in_range {
            // Not the latest in range - there might be an update available
            if let Some(ref range) = locked.original_range {
                outdated_in_range.push((name.clone(), locked.version.clone(), range.clone()));
            }
        }
    }

    if !outdated_in_range.is_empty() && verbose {
        UI::warn("Some tools have updates available within their version ranges:");
        for (name, current, range) in &outdated_in_range {
            UI::detail(&format!(
                "  {} {} is not the latest in range {}",
                name, current, range
            ));
        }
        UI::hint("Run 'vx update' to update within ranges, or 'vx check' for details");
    }
}
