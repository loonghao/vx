//! Dev command - Enter the development environment
//!
//! This command creates a shell environment with all project tools available.
//! It reads the vx.toml configuration and sets up PATH to include all
//! managed tool versions.

use crate::commands::setup::{parse_vx_config, ConfigView};
use crate::ui::{InstallProgress, UI};
use anyhow::{Context, Result};
use colored::Colorize;
use std::collections::HashMap;
use std::env;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use vx_env::ToolEnvironment;
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
) -> Result<()> {
    let current_dir = env::current_dir().context("Failed to get current directory")?;

    // Find and parse vx.toml using unified function from vx-paths
    let config_path = find_vx_config(&current_dir).map_err(|e| anyhow::anyhow!("{}", e))?;
    let config = parse_vx_config(&config_path)?;

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

    // Check if available in system PATH
    if let Some(system_path) = find_system_tool(tool) {
        return Ok((ToolStatus::SystemFallback, Some(system_path), None));
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

/// Find a tool in the system PATH (excluding vx paths)
fn find_system_tool(tool: &str) -> Option<PathBuf> {
    let exe_name = if cfg!(windows) {
        format!("{}.exe", tool)
    } else {
        tool.to_string()
    };

    let path_var = env::var("PATH").ok()?;
    let sep = if cfg!(windows) { ';' } else { ':' };

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

    None
}

/// Find the bin directory within a tool installation
fn find_tool_bin_dir(store_dir: &PathBuf, tool: &str) -> PathBuf {
    // Check bin/ subdirectory
    let bin_dir = store_dir.join("bin");
    if bin_dir.exists() {
        return bin_dir;
    }

    // Check for platform-specific subdirectories
    if let Ok(entries) = std::fs::read_dir(store_dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_dir() {
                let dir_name = path.file_name().unwrap_or_default().to_string_lossy();
                if dir_name.starts_with(&format!("{}-", tool)) {
                    return path;
                }
            }
        }
    }

    // Return store_dir as fallback
    store_dir.clone()
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
            ToolStatus::SystemFallback => "⚠".yellow(),
        };

        println!("  {} {}@{}", status_icon, tool.cyan(), version);

        if let Some(path) = actual_path {
            println!("    {} {}", "Path:".dimmed(), path.display());
        }

        if let Some(ver) = actual_version {
            if ver != *version && version != "latest" {
                println!("    {} {}", "Actual version:".dimmed(), ver);
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
    let mut tool_states: Vec<(String, String, ToolStatus)> = Vec::new();
    let mut missing_tools: Vec<(String, String)> = Vec::new();

    // First pass: check all tools
    for (tool, version) in tools {
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
                ToolStatus::SystemFallback => "⚠".yellow(),
            };
            let status_text = match status {
                ToolStatus::Installed => "installed".green(),
                ToolStatus::NotInstalled => "pending".yellow(),
                ToolStatus::SystemFallback => "system".yellow(),
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
    // Use ToolEnvironment from vx-env
    let mut env_vars = ToolEnvironment::new()
        .tools(&config.tools)
        .env_vars(&config.env)
        .warn_missing(verbose)
        .build()?;

    // Set VX_DEV environment variable to indicate we're in a dev shell
    env_vars.insert("VX_DEV".to_string(), "1".to_string());

    // Set VX_PROJECT_ROOT
    if let Ok(current_dir) = env::current_dir() {
        env_vars.insert(
            "VX_PROJECT_ROOT".to_string(),
            current_dir.to_string_lossy().to_string(),
        );
    }

    // Log tool paths if verbose
    if verbose {
        if let Some(path) = env_vars.get("PATH") {
            let sep = if cfg!(windows) { ";" } else { ":" };
            for entry in path.split(sep).take(config.tools.len() + 1) {
                UI::info(&format!("  PATH: {}", entry));
            }
        }
    }

    Ok(env_vars)
}

/// Build environment variables for script execution
///
/// This function builds the PATH environment variable to include vx-managed tools,
/// allowing scripts defined in vx.toml to use tools installed by vx.
///
/// Uses vx-env's ToolEnvironment for consistent environment building.
pub fn build_script_environment(config: &ConfigView) -> Result<HashMap<String, String>> {
    ToolEnvironment::new()
        .tools(&config.tools)
        .env_vars(&config.env)
        .build()
}

/// Execute a command in the dev environment
fn execute_command_in_env(cmd: &[String], env_vars: &HashMap<String, String>) -> Result<()> {
    if cmd.is_empty() {
        return Err(anyhow::anyhow!("No command specified"));
    }

    let program = &cmd[0];
    let args = &cmd[1..];

    let mut command = Command::new(program);
    command.args(args);

    // Set environment variables
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
    env_vars: &HashMap<String, String>,
    config: &ConfigView,
) -> Result<()> {
    let shell_path = shell.unwrap_or_else(detect_shell);

    UI::success("Entering vx development environment");
    UI::info(&format!(
        "Tools: {}",
        config.tools.keys().cloned().collect::<Vec<_>>().join(", ")
    ));
    UI::hint("Type 'exit' to leave the dev environment");
    println!();

    let mut command = Command::new(&shell_path);

    // Set environment variables
    for (key, value) in env_vars {
        command.env(key, value);
    }

    // Platform-specific shell configuration
    #[cfg(windows)]
    {
        if shell_path.contains("powershell") || shell_path.contains("pwsh") {
            // Create a temporary init script for PowerShell
            let init_script = create_powershell_init_script(config)?;
            let init_path =
                std::env::temp_dir().join(format!("vx_dev_init_{}.ps1", std::process::id()));
            std::fs::write(&init_path, init_script)?;

            // Use -NoLogo to suppress banner, -NoExit to keep shell open
            // -NoProfile to avoid loading user profile (which might conflict)
            // -File to execute our init script
            command.args([
                "-NoLogo",
                "-NoExit",
                "-Command",
                &format!(
                    ". '{}'; Remove-Item '{}'",
                    init_path.display(),
                    init_path.display()
                ),
            ]);
        } else if shell_path.contains("cmd") {
            command.args(["/K"]);
        }
    }

    #[cfg(not(windows))]
    {
        // For bash/zsh, we can set a custom prompt
        if shell_path.contains("bash") || shell_path.contains("zsh") {
            let prompt = format!(
                "(vx) {}",
                env::var("PS1").unwrap_or_else(|_| "\\$ ".to_string())
            );
            command.env("PS1", prompt);
        }
    }

    let status = command.status().with_context(|| {
        format!(
            "Failed to spawn shell: {}. Try specifying a shell with --shell",
            shell_path
        )
    })?;

    if !status.success() {
        std::process::exit(vx_resolver::exit_code_from_status(&status));
    }

    UI::info("Left vx development environment");
    Ok(())
}

/// Create PowerShell initialization script for dev environment
#[cfg(windows)]
fn create_powershell_init_script(config: &ConfigView) -> Result<String> {
    let tools = config.tools.keys().cloned().collect::<Vec<_>>().join(", ");

    Ok(format!(
        r#"
# VX Development Environment Initialization

# Set custom prompt to indicate vx dev environment
function prompt {{
    "(vx) " + $(if (Test-Path function:\DefaultPrompt) {{ & $function:DefaultPrompt }} else {{ "PS $($executionContext.SessionState.Path.CurrentLocation)$('>' * ($nestedPromptLevel + 1)) " }})
}}

# Show welcome message
Write-Host "VX Development Environment" -ForegroundColor Green
Write-Host "Tools: {}" -ForegroundColor Cyan
Write-Host ""

# Enable PSReadLine for better command history and completion
if (Get-Module -ListAvailable -Name PSReadLine) {{
    Import-Module PSReadLine -ErrorAction SilentlyContinue
    
    # Use shared history file across sessions
    Set-PSReadLineOption -HistorySavePath "$env:APPDATA\vx\powershell_history.txt" -ErrorAction SilentlyContinue
    
    # Enable predictive IntelliSense
    Set-PSReadLineOption -PredictionSource History -ErrorAction SilentlyContinue
}}

# Define helpful aliases
function vx-tools {{ Get-Command | Where-Object {{ $_.Source -match "vx" }} }}
function vx-exit {{ exit }}

# Load user's profile if it exists (for custom completions/aliases)
if (Test-Path $PROFILE) {{
    . $PROFILE
}}
"#,
        tools
    ))
}

/// Detect the user's preferred shell
fn detect_shell() -> String {
    // Check SHELL environment variable (Unix)
    if let Ok(shell) = env::var("SHELL") {
        return shell;
    }

    // Check COMSPEC for Windows
    if let Ok(comspec) = env::var("COMSPEC") {
        return comspec;
    }

    // Check for PowerShell on Windows
    #[cfg(windows)]
    {
        // Try to find pwsh (PowerShell Core) first
        if which::which("pwsh").is_ok() {
            return "pwsh".to_string();
        }
        // Fall back to Windows PowerShell
        if which::which("powershell").is_ok() {
            return "powershell".to_string();
        }
        // Last resort: cmd
        "cmd".to_string()
    }

    #[cfg(not(windows))]
    {
        // Default to bash on Unix
        "/bin/bash".to_string()
    }
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
