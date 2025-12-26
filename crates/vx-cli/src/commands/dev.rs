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
use vx_paths::PathManager;

/// Handle the dev command
pub async fn handle(
    shell: Option<String>,
    command: Option<Vec<String>>,
    no_install: bool,
    verbose: bool,
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
fn find_vx_config(start_dir: &Path) -> Result<PathBuf> {
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
fn build_dev_environment(config: &VxConfig, verbose: bool) -> Result<HashMap<String, String>> {
    let path_manager = PathManager::new()?;
    let mut env_vars = HashMap::new();

    // Collect all tool bin directories
    let mut path_entries = Vec::new();

    for (tool, version) in &config.tools {
        let tool_path = if version == "latest" {
            // Find the latest installed version
            let versions = path_manager.list_store_versions(tool)?;
            if let Some(latest) = versions.last() {
                get_tool_bin_path(&path_manager, tool, latest)?
            } else {
                if verbose {
                    UI::warn(&format!("Tool {} not installed", tool));
                }
                continue;
            }
        } else {
            get_tool_bin_path(&path_manager, tool, version)?
        };

        if let Some(path) = tool_path {
            if path.exists() {
                path_entries.push(path.to_string_lossy().to_string());
                if verbose {
                    UI::info(&format!("  {} -> {}", tool, path.display()));
                }
            }
        }
    }

    // Build PATH
    let current_path = env::var("PATH").unwrap_or_default();
    let new_path = if path_entries.is_empty() {
        current_path
    } else {
        let separator = if cfg!(windows) { ";" } else { ":" };
        format!(
            "{}{}{}",
            path_entries.join(separator),
            separator,
            current_path
        )
    };
    env_vars.insert("PATH".to_string(), new_path);

    // Add vx bin directory to PATH
    let vx_bin = path_manager.bin_dir();
    if vx_bin.exists() {
        let path = env_vars.get("PATH").cloned().unwrap_or_default();
        let separator = if cfg!(windows) { ";" } else { ":" };
        env_vars.insert(
            "PATH".to_string(),
            format!("{}{}{}", vx_bin.display(), separator, path),
        );
    }

    // Set VX_DEV environment variable to indicate we're in a dev shell
    env_vars.insert("VX_DEV".to_string(), "1".to_string());

    // Set VX_PROJECT_ROOT
    if let Ok(current_dir) = env::current_dir() {
        env_vars.insert(
            "VX_PROJECT_ROOT".to_string(),
            current_dir.to_string_lossy().to_string(),
        );
    }

    // Add custom environment variables from config
    for (key, value) in &config.env {
        env_vars.insert(key.clone(), value.clone());
    }

    Ok(env_vars)
}

/// Build environment variables for script execution (simplified version of build_dev_environment)
///
/// This function builds the PATH environment variable to include vx-managed tools,
/// allowing scripts defined in .vx.toml to use tools installed by vx.
pub fn build_script_environment(config: &VxConfig) -> Result<HashMap<String, String>> {
    let path_manager = PathManager::new()?;
    let mut env_vars = HashMap::new();

    // Collect all tool bin directories
    let mut path_entries = Vec::new();

    for (tool, version) in &config.tools {
        let tool_path = if version == "latest" {
            // Find the latest installed version
            let versions = path_manager.list_store_versions(tool)?;
            if let Some(latest) = versions.last() {
                get_tool_bin_path(&path_manager, tool, latest)?
            } else {
                continue;
            }
        } else {
            get_tool_bin_path(&path_manager, tool, version)?
        };

        if let Some(path) = tool_path {
            if path.exists() {
                path_entries.push(path.to_string_lossy().to_string());
            }
        }
    }

    // Build PATH
    let current_path = env::var("PATH").unwrap_or_default();
    let new_path = if path_entries.is_empty() {
        current_path
    } else {
        let separator = if cfg!(windows) { ";" } else { ":" };
        format!(
            "{}{}{}",
            path_entries.join(separator),
            separator,
            current_path
        )
    };
    env_vars.insert("PATH".to_string(), new_path);

    // Add vx bin directory to PATH
    let vx_bin = path_manager.bin_dir();
    if vx_bin.exists() {
        let path = env_vars.get("PATH").cloned().unwrap_or_default();
        let separator = if cfg!(windows) { ";" } else { ":" };
        env_vars.insert(
            "PATH".to_string(),
            format!("{}{}{}", vx_bin.display(), separator, path),
        );
    }

    // Add custom environment variables from config
    for (key, value) in &config.env {
        env_vars.insert(key.clone(), value.clone());
    }

    Ok(env_vars)
}

/// Get the bin path for a tool
fn get_tool_bin_path(
    path_manager: &PathManager,
    tool: &str,
    version: &str,
) -> Result<Option<PathBuf>> {
    // Check store first
    let store_dir = path_manager.version_store_dir(tool, version);
    if store_dir.exists() {
        // Different tools have different bin directory structures
        // Priority order:
        // 1. bin/ subdirectory (standard layout)
        // 2. Direct in version directory (some tools like uv on Windows)
        // 3. Subdirectories matching tool-* pattern (uv on Linux/macOS: uv-{platform}/)
        let mut bin_candidates = vec![
            store_dir.join("bin"),
            store_dir.clone(), // Some tools put executables directly in the version dir
        ];

        // Add subdirectories that might contain the executable
        // This handles cases like uv where the executable is in uv-{platform}/ subdirectory
        if let Ok(entries) = std::fs::read_dir(&store_dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                if path.is_dir() {
                    let dir_name = path.file_name().unwrap_or_default().to_string_lossy();
                    // Check for tool-* pattern (e.g., uv-x86_64-unknown-linux-gnu)
                    if dir_name.starts_with(&format!("{}-", tool)) {
                        bin_candidates.push(path);
                    }
                }
            }
        }

        for bin_dir in bin_candidates {
            if bin_dir.exists() {
                // Verify the directory actually contains an executable
                let exe_name = if cfg!(windows) {
                    format!("{}.exe", tool)
                } else {
                    tool.to_string()
                };
                if bin_dir.join(&exe_name).exists() || bin_dir == store_dir {
                    return Ok(Some(bin_dir));
                }
            }
        }

        // Fallback: if store_dir exists, return it (the tool might use a different executable name)
        return Ok(Some(store_dir));
    }

    // Check npm-tools
    let npm_bin = path_manager.npm_tool_bin_dir(tool, version);
    if npm_bin.exists() {
        return Ok(Some(npm_bin));
    }

    // Check pip-tools
    let pip_bin = path_manager.pip_tool_bin_dir(tool, version);
    if pip_bin.exists() {
        return Ok(Some(pip_bin));
    }

    Ok(None)
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
