//! Dev command - Enter the development environment
//!
//! This command creates a shell environment with all project tools available.
//! It reads the .vx.toml configuration and sets up PATH to include all
//! managed tool versions.

use crate::commands::setup::{parse_vx_config, VxConfig};
use crate::ui::{InstallProgress, UI};
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::env;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use vx_env::ToolEnvironment;
use vx_paths::PathManager;

/// Handle the dev command
pub async fn handle(
    shell: Option<String>,
    command: Option<Vec<String>>,
    no_install: bool,
    verbose: bool,
    export: bool,
    format: Option<String>,
) -> Result<()> {
    let current_dir = env::current_dir().context("Failed to get current directory")?;

    // Find and parse .vx.toml
    let config_path = find_vx_config(&current_dir)?;
    let config = parse_vx_config(&config_path)?;

    if config.tools.is_empty() {
        UI::warn("No tools configured in .vx.toml");
        UI::hint("Run 'vx init' to initialize the project configuration");
        return Ok(());
    }

    // Handle --export mode
    if export {
        return handle_export(&config, format);
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

/// Find .vx.toml in current directory or parent directories
///
/// If VX_PROJECT_ROOT is set, only search in the current directory
/// (used for test isolation).
fn find_vx_config(start_dir: &Path) -> Result<PathBuf> {
    // Check if VX_PROJECT_ROOT is set (test isolation mode)
    if std::env::var("VX_PROJECT_ROOT").is_ok() {
        let config_path = start_dir.join(".vx.toml");
        if config_path.exists() {
            return Ok(config_path);
        }
        return Err(anyhow::anyhow!(
            "No .vx.toml found in current directory.\n\
             Run 'vx init' to create one."
        ));
    }

    // Normal mode: search up the directory tree
    let mut current = start_dir.to_path_buf();

    loop {
        let config_path = current.join(".vx.toml");
        if config_path.exists() {
            return Ok(config_path);
        }

        if !current.pop() {
            break;
        }
    }

    Err(anyhow::anyhow!(
        "No .vx.toml found in current directory or parent directories.\n\
         Run 'vx init' to create one."
    ))
}

/// Handle --export mode: output shell script for environment activation
fn handle_export(config: &VxConfig, format: Option<String>) -> Result<()> {
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

/// Check if tools are installed and install missing ones
async fn check_and_install_tools(tools: &HashMap<String, String>, verbose: bool) -> Result<()> {
    let path_manager = PathManager::new()?;
    let mut missing_tools = Vec::new();

    for (tool, version) in tools {
        let version_to_check = if version == "latest" {
            // For latest, check if any version is installed
            let versions = path_manager.list_store_versions(tool)?;
            if versions.is_empty() {
                missing_tools.push((tool.clone(), version.clone()));
            }
            continue;
        } else {
            version.clone()
        };

        if !path_manager.is_version_in_store(tool, &version_to_check) {
            missing_tools.push((tool.clone(), version.clone()));
        }
    }

    if missing_tools.is_empty() {
        if verbose {
            UI::success("All tools are installed");
        }
        return Ok(());
    }

    // Use InstallProgress for modern progress display
    let mut progress = InstallProgress::new(
        missing_tools.len(),
        &format!("Installing {} missing tool(s)", missing_tools.len()),
    );

    for (tool, version) in &missing_tools {
        progress.start_tool(tool, version);

        // Use vx install command with suppressed output
        let status = Command::new(env::current_exe()?)
            .args(["install", tool, version])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .with_context(|| format!("Failed to install {}@{}", tool, version))?;

        progress.complete_tool(status.success(), tool, version);
    }

    progress.finish("✓ All tools installed");
    Ok(())
}

/// Build environment variables for the dev shell
///
/// Uses vx-env's ToolEnvironment for consistent environment building.
fn build_dev_environment(config: &VxConfig, verbose: bool) -> Result<HashMap<String, String>> {
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
/// allowing scripts defined in .vx.toml to use tools installed by vx.
///
/// Uses vx-env's ToolEnvironment for consistent environment building.
pub fn build_script_environment(config: &VxConfig) -> Result<HashMap<String, String>> {
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
        std::process::exit(status.code().unwrap_or(1));
    }

    Ok(())
}

/// Spawn an interactive dev shell
fn spawn_dev_shell(
    shell: Option<String>,
    env_vars: &HashMap<String, String>,
    config: &VxConfig,
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
            command.args(["-NoLogo", "-NoExit"]);
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
        std::process::exit(status.code().unwrap_or(1));
    }

    UI::info("Left vx development environment");
    Ok(())
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
pub fn generate_env_export(config: &VxConfig, format: ExportFormat) -> Result<String> {
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
