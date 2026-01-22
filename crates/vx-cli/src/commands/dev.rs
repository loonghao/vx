//! Dev command - Enter the development environment
//!
//! This command creates a shell environment with all project tools available.
//! It reads the vx.toml configuration and sets up PATH to include all
//! managed tool versions. Updated: env_clear for isolation.
//!
//! ## Isolation Mode
//!
//! By default, `vx dev` runs in isolation mode where only vx-managed tools
//! are available in PATH. System tools are NOT inherited unless:
//! - `--inherit-system` flag is used
//! - `isolation = false` is set in `vx.toml` settings
//!
//! ## Environment Variable Passthrough
//!
//! In isolation mode, only essential system variables and those matching
//! `passenv` patterns are available. Configure in `vx.toml`:
//!
//! ```toml
//! [settings]
//! passenv = ["GITHUB_*", "CI", "SSH_*"]
//! ```

use crate::commands::setup::{parse_vx_config, ConfigView};
use crate::ui::{InstallProgress, UI};
use anyhow::{Context, Result};
use colored::Colorize;
use std::collections::HashMap;
use std::env;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use vx_env::{SessionContext, ShellSpawner, ToolEnvironment};
use vx_paths::{find_vx_config, PathManager};

/// Handle the dev command
pub async fn handle(
    shell: Option<String>,
    command: Option<Vec<String>>,
    no_install: bool,
    verbose: bool,
    export: bool,
    format: Option<String>,
    info: bool,
    inherit_system: bool,
    cli_passenv: Vec<String>,
) -> Result<()> {
    let current_dir = env::current_dir().context("Failed to get current directory")?;

    // Find and parse vx.toml using unified function from vx-paths
    let config_path = find_vx_config(&current_dir).map_err(|e| anyhow::anyhow!("{}", e))?;
    let mut config = parse_vx_config(&config_path)?;

    // Override isolation mode if --inherit-system is specified
    if inherit_system {
        config.isolation = false;
    }

    // Merge CLI passenv with config passenv
    if !cli_passenv.is_empty() {
        config.passenv.extend(cli_passenv);
    }

    if config.tools.is_empty() {
        UI::warn("No tools configured in vx.toml");
        UI::hint("Run 'vx init' to initialize the project configuration");
        return Ok(());
    }

    // Handle --export mode
    if export {
        return handle_export(&config, format);
    }

    // Handle --info mode
    if info {
        return handle_info(&config).await;
    }

    // Check and install missing tools if needed
    if !no_install {
        let auto_install = config
            .settings
            .get("auto_install")
            .map(|v| v == "true")
            .unwrap_or(true);

        if auto_install {
            check_and_install_tools(&config.tools, verbose).await?;
        }
    }

    // Build the environment
    let env_vars = build_dev_environment(&config, verbose)?;

    // Execute command or spawn shell
    if let Some(cmd) = command {
        execute_command_in_env(&cmd, &env_vars)?;
    } else {
        spawn_dev_shell(shell, &env_vars, &config)?;
    }

    Ok(())
}

/// Tool installation status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ToolStatus {
    /// Tool is installed by vx
    Installed,
    /// Tool is not installed
    NotInstalled,
    /// Tool is available from system PATH (not vx managed)
    SystemFallback,
}

/// Get the status and path of a tool
fn get_tool_status(
    path_manager: &PathManager,
    tool: &str,
    version: &str,
) -> Result<(ToolStatus, Option<PathBuf>, Option<String>)> {
    // Handle "system" version specially - use system-installed tool
    if version == "system" {
        if let Some(system_path) = find_system_tool(tool) {
            let detected_version = get_system_tool_version(tool);
            return Ok((ToolStatus::SystemFallback, Some(system_path), detected_version));
        }
        return Ok((ToolStatus::NotInstalled, None, None));
    }
    
    let actual_version = if version == "latest" {
        path_manager
            .list_store_versions(tool)?
            .last()
            .cloned()
            .unwrap_or_else(|| version.to_string())
    } else {
        version.to_string()
    };

    // Check store first
    let store_dir = path_manager.version_store_dir(tool, &actual_version);
    if store_dir.exists() {
        let bin_path = find_tool_bin_dir(&store_dir, tool);
        return Ok((ToolStatus::Installed, Some(bin_path), Some(actual_version)));
    }

    // Check npm-tools
    let npm_bin = path_manager.npm_tool_bin_dir(tool, &actual_version);
    if npm_bin.exists() {
        return Ok((ToolStatus::Installed, Some(npm_bin), Some(actual_version)));
    }

    // Check pip-tools
    let pip_bin = path_manager.pip_tool_bin_dir(tool, &actual_version);
    if pip_bin.exists() {
        return Ok((ToolStatus::Installed, Some(pip_bin), Some(actual_version)));
    }

    // Check if available in system PATH as fallback
    if let Some(system_path) = find_system_tool(tool) {
        let detected_version = get_system_tool_version(tool);
        return Ok((ToolStatus::SystemFallback, Some(system_path), detected_version));
    }

    Ok((ToolStatus::NotInstalled, None, None))
}

/// Get the vx-managed path for a tool
fn get_vx_tool_path(
    path_manager: &PathManager,
    tool: &str,
    version: &str,
) -> Result<Option<PathBuf>> {
    let actual_version = if version == "latest" {
        path_manager
            .list_store_versions(tool)?
            .last()
            .cloned()
            .unwrap_or_else(|| version.to_string())
    } else {
        version.to_string()
    };

    // Check store
    let store_dir = path_manager.version_store_dir(tool, &actual_version);
    if store_dir.exists() {
        return Ok(Some(find_tool_bin_dir(&store_dir, tool)));
    }

    // Check npm-tools
    let npm_bin = path_manager.npm_tool_bin_dir(tool, &actual_version);
    if npm_bin.exists() {
        return Ok(Some(npm_bin));
    }

    // Check pip-tools
    let pip_bin = path_manager.pip_tool_bin_dir(tool, &actual_version);
    if pip_bin.exists() {
        return Ok(Some(pip_bin));
    }

    Ok(None)
}

/// Get the version command and parser for a tool
fn get_version_command(tool: &str) -> Option<(&'static str, &'static [&'static str], fn(&str) -> Option<String>)> {
    match tool {
        "rust" => Some(("cargo", &["--version"][..], |output| {
            // "cargo 1.91.1 (ea2d97820 2025-10-10)" -> "1.91.1"
            output.split_whitespace().nth(1).map(|s| s.to_string())
        })),
        "go" | "golang" => Some(("go", &["version"][..], |output| {
            // "go version go1.21.0 linux/amd64" -> "1.21.0"
            output.split_whitespace()
                .find(|s| s.starts_with("go"))
                .and_then(|s| s.strip_prefix("go"))
                .map(|s| s.to_string())
        })),
        "node" | "nodejs" => Some(("node", &["--version"][..], |output| {
            // "v20.0.0" -> "20.0.0"
            output.trim().strip_prefix('v').map(|s| s.to_string())
        })),
        "python" => Some(("python", &["--version"][..], |output| {
            // "Python 3.11.0" -> "3.11.0"
            output.split_whitespace().nth(1).map(|s| s.to_string())
        })),
        "uv" => Some(("uv", &["--version"][..], |output| {
            // "uv 0.5.0" -> "0.5.0"
            output.split_whitespace().nth(1).map(|s| s.to_string())
        })),
        "deno" => Some(("deno", &["--version"][..], |output| {
            // "deno 1.40.0 ..." -> "1.40.0"
            output.lines().next()
                .and_then(|line| line.split_whitespace().nth(1))
                .map(|s| s.to_string())
        })),
        "bun" => Some(("bun", &["--version"][..], |output| {
            // "1.0.0" -> "1.0.0"
            Some(output.trim().to_string())
        })),
        // For unknown tools, we can't get version without knowing the executable
        _ => None,
    }
}

/// Get the version of a system-installed tool
fn get_system_tool_version(tool: &str) -> Option<String> {
    let (exe, args, parser) = get_version_command(tool)?;
    
    let output = Command::new(exe)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .ok()?;
    
    if !output.status.success() {
        return None;
    }
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    // Some tools output version to stderr
    parser(&stdout).or_else(|| parser(&stderr))
}

/// Find a tool in the system PATH (excluding vx paths)
fn find_system_tool(tool: &str) -> Option<PathBuf> {
    // Map tool names to their actual executables
    // Some tools have different names for the provider vs the executable
    let executables: Vec<&str> = match tool {
        "rust" => vec!["cargo", "rustc"],
        "go" | "golang" => vec!["go"],
        "node" | "nodejs" => vec!["node"],
        "python" => vec!["python", "python3"],
        "uv" => vec!["uv"],
        _ => vec![tool],
    };

    let path_var = env::var("PATH").ok()?;
    let sep = if cfg!(windows) { ';' } else { ':' };

    for exe in executables {
        let exe_name = if cfg!(windows) {
            format!("{}.exe", exe)
        } else {
            exe.to_string()
        };

        for dir in path_var.split(sep) {
            // Skip vx directories
            if dir.contains(".vx") {
                continue;
            }

            let exe_path = PathBuf::from(dir).join(&exe_name);
            if exe_path.exists() {
                return Some(exe_path);
            }
        }
    }

    None
}

/// Find the bin directory within a tool installation
fn find_tool_bin_dir(store_dir: &PathBuf, tool: &str) -> PathBuf {
    // Check bin/ subdirectory
    let bin_dir = store_dir.join("bin");
    if bin_dir.exists() && has_executable(&bin_dir, tool) {
        return bin_dir;
    }

    // Check for platform-specific subdirectories (e.g., cmake-4.2.2-windows-x86_64)
    if let Ok(entries) = std::fs::read_dir(store_dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_dir() {
                let dir_name = path.file_name().unwrap_or_default().to_string_lossy();
                if dir_name.starts_with(&format!("{}-", tool)) {
                    // Check for bin/ inside the platform-specific directory
                    let nested_bin = path.join("bin");
                    if nested_bin.exists() && has_executable(&nested_bin, tool) {
                        return nested_bin;
                    }
                    // Check for executable directly in the platform-specific directory
                    if has_executable(&path, tool) {
                        return path;
                    }
                }
            }
        }
    }

    // Search recursively for bin/ directory (handles nested structures)
    if let Some(bin_path) = find_bin_recursive(store_dir, tool, 2) {
        return bin_path;
    }

    // Return store_dir as fallback
    store_dir.clone()
}

/// Check if a directory contains the tool executable
fn has_executable(dir: &std::path::Path, tool: &str) -> bool {
    let exe_name = if cfg!(windows) {
        format!("{}.exe", tool)
    } else {
        tool.to_string()
    };
    dir.join(&exe_name).exists()
}

/// Recursively search for a bin directory containing the tool executable
fn find_bin_recursive(dir: &PathBuf, tool: &str, max_depth: u32) -> Option<PathBuf> {
    if max_depth == 0 {
        return None;
    }

    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_dir() {
                let dir_name = path.file_name().unwrap_or_default().to_string_lossy();
                
                // Check if this is a bin directory
                if dir_name == "bin" && has_executable(&path, tool) {
                    return Some(path);
                }
                
                // Recurse into subdirectories
                if let Some(found) = find_bin_recursive(&path, tool, max_depth - 1) {
                    return Some(found);
                }
            }
        }
    }

    None
}

/// Handle --export mode: output shell script for environment activation
fn handle_export(config: &ConfigView, format: Option<String>) -> Result<()> {
    let export_format = match format {
        Some(f) => ExportFormat::parse(&f).ok_or_else(|| {
            anyhow::anyhow!(
                "Unknown format: {}. Use: shell, powershell, batch, or github",
                f
            )
        })?,
        None => ExportFormat::detect(),
    };

    let output = generate_env_export(config, export_format)?;
    print!("{}", output);

    Ok(())
}

/// Handle --info mode: display detailed environment information
async fn handle_info(config: &ConfigView) -> Result<()> {
    let path_manager = PathManager::new()?;

    println!("{}", "VX Development Environment Information".bold());
    println!("{}", "═".repeat(50));
    println!();

    // Display configured tools and their status
    println!("{}", "Configured Tools:".bold().cyan());
    println!();

    for (tool, version) in &config.tools {
        let (status, actual_path, actual_version) =
            get_tool_status(&path_manager, tool, version)?;

        let status_icon = match status {
            ToolStatus::Installed => "✓".green(),
            ToolStatus::NotInstalled => "✗".red(),
            ToolStatus::SystemFallback => "✓".green(),  // System tools are valid
        };
        
        // For system tools, show the detected version instead of "system"
        let display_version = if version == "system" {
            actual_version.clone().unwrap_or_else(|| "system".to_string())
        } else {
            version.clone()
        };
        
        let version_suffix = if version == "system" {
            " (system)".dimmed().to_string()
        } else {
            String::new()
        };

        println!("  {} {}@{}{}", status_icon, tool.cyan(), display_version, version_suffix);

        if let Some(path) = actual_path {
            println!("    {} {}", "Path:".dimmed(), path.display());
        }

        // Show actual version if different from configured (for non-system tools)
        if version != "system" {
            if let Some(ver) = actual_version {
                if ver != *version && version != "latest" {
                    println!("    {} {}", "Actual version:".dimmed(), ver);
                }
            }
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
                println!("  {} {}", (i + 1).to_string().dimmed(), "... (system PATH)".dimmed());
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
    for (tool, _version) in &config.tools {
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
        println!("  {} {}", "✓".green(), "No conflicts detected");
    }

    println!();
    println!(
        "{}",
        "Run 'vx dev' to enter the development environment.".dimmed()
    );

    Ok(())
}

/// Check if tools are installed and install missing ones
async fn check_and_install_tools(tools: &HashMap<String, String>, verbose: bool) -> Result<()> {
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
                let display_version = get_system_tool_version(tool)
                    .unwrap_or_else(|| "system".to_string());
                tool_states.push((tool.clone(), display_version, ToolStatus::SystemFallback));
            } else {
                // System tool not found, but we don't try to install it
                UI::warn(&format!("{} specified as 'system' but not found in PATH", tool));
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
                ToolStatus::SystemFallback => "✓".green(),  // System tools are valid
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
        let status = Command::new(env::current_exe()?)
            .args(["install", tool, version])
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

/// Build environment variables for the dev shell
///
/// Uses vx-env's ToolEnvironment for consistent environment building.
fn build_dev_environment(config: &ConfigView, verbose: bool) -> Result<HashMap<String, String>> {
    // Merge env from vx.toml with setenv from settings
    let mut env_vars = config.env.clone();
    env_vars.extend(config.setenv.clone());

    // Use ToolEnvironment from vx-env with isolation settings
    let mut builder = ToolEnvironment::new()
        .tools(&config.tools)
        .env_vars(&env_vars)
        .warn_missing(verbose)
        .isolation(config.isolation);

    // Add passenv patterns if in isolation mode
    if config.isolation && !config.passenv.is_empty() {
        builder = builder.passenv(config.passenv.clone());
    }

    let mut env_result = builder.build()?;

    // Set VX_DEV environment variable to indicate we're in a dev shell
    env_result.insert("VX_DEV".to_string(), "1".to_string());

    // Set VX_PROJECT_NAME for prompt customization
    env_result.insert("VX_PROJECT_NAME".to_string(), config.project_name.clone());

    // Set VX_PROJECT_ROOT
    if let Ok(current_dir) = env::current_dir() {
        env_result.insert(
            "VX_PROJECT_ROOT".to_string(),
            current_dir.to_string_lossy().to_string(),
        );
    }

    // Log tool paths if verbose
    if verbose {
        if config.isolation {
            UI::info("Running in isolation mode");
            if !config.passenv.is_empty() {
                UI::info(&format!("  passenv: {}", config.passenv.join(", ")));
            }
        }
        if let Some(path) = env_result.get("PATH") {
            let sep = if cfg!(windows) { ";" } else { ":" };
            for entry in path.split(sep).take(config.tools.len() + 1) {
                UI::info(&format!("  PATH: {}", entry));
            }
        }
    }

    Ok(env_result)
}

/// Build environment variables for script execution
///
/// This function builds the PATH environment variable to include vx-managed tools,
/// allowing scripts defined in vx.toml to use tools installed by vx.
///
/// Uses vx-env's ToolEnvironment for consistent environment building.
pub fn build_script_environment(config: &ConfigView) -> Result<HashMap<String, String>> {
    // Merge env from vx.toml with setenv from settings
    let mut env_vars = config.env.clone();
    env_vars.extend(config.setenv.clone());

    let mut builder = ToolEnvironment::new()
        .tools(&config.tools)
        .env_vars(&env_vars)
        .isolation(config.isolation);

    // Add passenv patterns if in isolation mode
    if config.isolation && !config.passenv.is_empty() {
        builder = builder.passenv(config.passenv.clone());
    }

    builder.build()
}

/// Execute a command in the dev environment
fn execute_command_in_env(cmd: &[String], env_vars: &HashMap<String, String>) -> Result<()> {
    if cmd.is_empty() {
        return Err(anyhow::anyhow!("No command specified"));
    }

    let program = &cmd[0];
    let args = &cmd[1..];

    // Clear inherited environment and set only our variables
    // This ensures the command uses our PATH, not the parent process's PATH
    let mut command = Command::new(program);
    command.args(args);
    command.env_clear();
    
    // Set all environment variables from our isolated/configured environment
    for (key, value) in env_vars {
        command.env(key, value);
    }

    let status = command
        .status()
        .with_context(|| format!("Failed to execute: {}", program))?;

    if !status.success() {
        std::process::exit(vx_resolver::exit_code_from_status(&status));
    }

    Ok(())
}

/// Spawn an interactive dev shell
fn spawn_dev_shell(
    shell: Option<String>,
    _env_vars: &HashMap<String, String>,
    config: &ConfigView,
) -> Result<()> {
    // Create SessionContext from config
    let mut session = SessionContext::new(&config.project_name)
        .tools(&config.tools)
        .env_vars(&config.env)
        .env_vars(&config.setenv)
        .isolated(config.isolation)
        .passenv(config.passenv.clone());

    if let Ok(current_dir) = env::current_dir() {
        session = session.project_root(current_dir);
    }

    // Use ShellSpawner for unified shell management
    let spawner = ShellSpawner::new(session)?;

    UI::success("Entering vx development environment");
    UI::info(&format!(
        "Tools: {}",
        config.tools.keys().cloned().collect::<Vec<_>>().join(", ")
    ));
    UI::hint("Type 'exit' to leave the dev environment");
    println!();

    let status = spawner.spawn_interactive(shell.as_deref())?;

    if !status.success() {
        std::process::exit(vx_resolver::exit_code_from_status(&status));
    }

    UI::info("Left vx development environment");
    Ok(())
}

/// Output format for environment export
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    /// Shell script (bash/zsh compatible)
    Shell,
    /// PowerShell script
    PowerShell,
    /// Windows batch file
    Batch,
    /// GitHub Actions format (GITHUB_ENV and GITHUB_PATH)
    GithubActions,
}

impl ExportFormat {
    /// Detect the best format based on the current environment
    pub fn detect() -> Self {
        // Check if running in GitHub Actions
        if env::var("GITHUB_ACTIONS").is_ok() {
            return Self::GithubActions;
        }

        #[cfg(windows)]
        {
            // Check if running in PowerShell
            if env::var("PSModulePath").is_ok() {
                return Self::PowerShell;
            }
            Self::Batch
        }

        #[cfg(not(windows))]
        {
            Self::Shell
        }
    }

    /// Parse format from string
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "shell" | "sh" | "bash" | "zsh" => Some(Self::Shell),
            "powershell" | "pwsh" | "ps1" => Some(Self::PowerShell),
            "batch" | "bat" | "cmd" => Some(Self::Batch),
            "github" | "github-actions" | "gha" => Some(Self::GithubActions),
            _ => None,
        }
    }
}

/// Generate environment export script for the given config
///
/// This function generates a script that can be sourced/executed to set up
/// the environment with all vx-managed tools in PATH.
///
/// Usage:
/// - Bash/Zsh: `eval "$(vx env --export)"`
/// - PowerShell: `Invoke-Expression (vx env --export --format powershell)`
/// - GitHub Actions: `vx env --export --format github >> $GITHUB_ENV`
pub fn generate_env_export(config: &ConfigView, format: ExportFormat) -> Result<String> {
    // Build environment using ToolEnvironment
    let env_vars = ToolEnvironment::new()
        .tools(&config.tools)
        .env_vars(&config.env)
        .warn_missing(false)
        .build()?;

    // Extract PATH entries for export formatting
    let path = env_vars.get("PATH").cloned().unwrap_or_default();
    let sep = if cfg!(windows) { ";" } else { ":" };
    let current_path = std::env::var("PATH").unwrap_or_default();

    // Get only the new path entries (before current PATH)
    let path_entries: Vec<String> = path
        .split(sep)
        .take_while(|p| !current_path.starts_with(*p))
        .map(|s| s.to_string())
        .collect();

    // Generate output based on format
    let output = match format {
        ExportFormat::Shell => generate_shell_export(&path_entries, &config.env),
        ExportFormat::PowerShell => generate_powershell_export(&path_entries, &config.env),
        ExportFormat::Batch => generate_batch_export(&path_entries, &config.env),
        ExportFormat::GithubActions => generate_github_actions_export(&path_entries, &config.env),
    };

    Ok(output)
}

fn generate_shell_export(path_entries: &[String], env_vars: &HashMap<String, String>) -> String {
    let mut output = String::new();

    // Export PATH
    if !path_entries.is_empty() {
        let paths = path_entries.join(":");
        output.push_str(&format!("export PATH=\"{}:$PATH\"\n", paths));
    }

    // Export custom environment variables
    for (key, value) in env_vars {
        // Escape special characters in value
        let escaped = value.replace('\\', "\\\\").replace('"', "\\\"");
        output.push_str(&format!("export {}=\"{}\"\n", key, escaped));
    }

    output
}

fn generate_powershell_export(
    path_entries: &[String],
    env_vars: &HashMap<String, String>,
) -> String {
    let mut output = String::new();

    // Export PATH
    if !path_entries.is_empty() {
        let paths = path_entries.join(";");
        output.push_str(&format!(
            "$env:PATH = \"{};$env:PATH\"\n",
            paths.replace('\\', "\\\\")
        ));
    }

    // Export custom environment variables
    for (key, value) in env_vars {
        let escaped = value.replace('\\', "\\\\").replace('"', "`\"");
        output.push_str(&format!("$env:{} = \"{}\"\n", key, escaped));
    }

    output
}

fn generate_batch_export(path_entries: &[String], env_vars: &HashMap<String, String>) -> String {
    let mut output = String::new();

    // Export PATH
    if !path_entries.is_empty() {
        let paths = path_entries.join(";");
        output.push_str(&format!("set PATH={};%PATH%\n", paths));
    }

    // Export custom environment variables
    for (key, value) in env_vars {
        output.push_str(&format!("set {}={}\n", key, value));
    }

    output
}

fn generate_github_actions_export(
    path_entries: &[String],
    env_vars: &HashMap<String, String>,
) -> String {
    let mut output = String::new();

    // For GitHub Actions, we output in a format that can be appended to GITHUB_ENV and GITHUB_PATH
    // The caller should redirect this appropriately

    // PATH entries (one per line for GITHUB_PATH)
    output.push_str("# Add the following to GITHUB_PATH:\n");
    for path in path_entries {
        output.push_str(&format!("# {}\n", path));
    }

    // Generate shell commands that work in GitHub Actions
    output.push_str("\n# Shell commands to set environment:\n");
    if !path_entries.is_empty() {
        for path in path_entries {
            output.push_str(&format!("echo \"{}\" >> $GITHUB_PATH\n", path));
        }
        // Also export for current step
        let paths = path_entries.join(":");
        output.push_str(&format!("export PATH=\"{}:$PATH\"\n", paths));
    }

    // Environment variables
    for (key, value) in env_vars {
        output.push_str(&format!("echo \"{}={}\" >> $GITHUB_ENV\n", key, value));
        output.push_str(&format!("export {}=\"{}\"\n", key, value));
    }

    output
}
