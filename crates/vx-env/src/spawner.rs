//! Shell spawner for vx environments
//!
//! This module provides unified shell spawning functionality used by both
//! `vx dev` and `vx env shell` commands.

use crate::assets::ShellScript;
use crate::session::SessionContext;
use crate::ToolEnvironment;
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::env;
use std::process::{Command, ExitStatus};

/// Export format for environment variables
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    /// Shell script (bash/zsh compatible)
    Shell,
    /// PowerShell script
    PowerShell,
    /// Windows batch file
    Batch,
    /// GitHub Actions format
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

/// Shell spawner for creating shell sessions with vx-managed environments
pub struct ShellSpawner {
    session: SessionContext,
    env_vars: HashMap<String, String>,
}

impl ShellSpawner {
    /// Create a new shell spawner from a session context
    pub fn new(session: SessionContext) -> Result<Self> {
        // Build environment variables using ToolEnvironment
        let env_vars = Self::build_env_vars(&session)?;

        Ok(Self { session, env_vars })
    }

    /// Build environment variables from session context
    fn build_env_vars(session: &SessionContext) -> Result<HashMap<String, String>> {
        let mut builder = ToolEnvironment::new()
            .tools(&session.tools)
            .env_vars(&session.env_vars)
            .isolation(session.isolation.enabled)
            .passenv(session.isolation.passenv.clone())
            .warn_missing(true);

        // Add project root to env vars
        if let Some(root) = &session.project_root {
            builder = builder.env_var("VX_PROJECT_ROOT", root.display().to_string());
        }

        builder = builder.env_var("VX_PROJECT_NAME", &session.name);
        builder = builder.env_var("VX_DEV", "1");

        builder.build().map_err(|e| anyhow::anyhow!("{}", e))
    }

    /// Get the built environment variables
    pub fn env_vars(&self) -> &HashMap<String, String> {
        &self.env_vars
    }

    /// Get the session context
    pub fn session(&self) -> &SessionContext {
        &self.session
    }

    /// Spawn an interactive shell
    pub fn spawn_interactive(&self, shell: Option<&str>) -> Result<ExitStatus> {
        let shell_path = shell
            .map(|s| s.to_string())
            .unwrap_or_else(detect_shell);

        let mut command = Command::new(&shell_path);

        // IMPORTANT: Clear inherited environment to ensure our PATH takes effect
        // Without this, the shell inherits the parent's PATH which may not include vx tools
        command.env_clear();

        // Set all environment variables from our built environment
        for (key, value) in &self.env_vars {
            command.env(key, value);
        }

        // Set VX_PROJECT_NAME for prompt customization
        command.env("VX_PROJECT_NAME", &self.session.name);

        // Platform-specific shell configuration
        self.configure_shell_platform(&mut command, &shell_path)?;

        let status = command.status().with_context(|| {
            format!(
                "Failed to spawn shell: {}. Try specifying a shell with --shell",
                shell_path
            )
        })?;

        Ok(status)
    }

    /// Configure shell for the current platform
    #[cfg(windows)]
    fn configure_shell_platform(&self, command: &mut Command, shell_path: &str) -> Result<()> {
        if shell_path.contains("powershell") || shell_path.contains("pwsh") {
            // Create a temporary init script for PowerShell
            let init_script = self.create_powershell_init_script()?;

            // Use a persistent path in vx directory for the init script
            let vx_temp = dirs::data_local_dir()
                .unwrap_or_else(std::env::temp_dir)
                .join("vx")
                .join("temp");
            std::fs::create_dir_all(&vx_temp)?;
            let init_path = vx_temp.join("vx_shell_init.ps1");
            std::fs::write(&init_path, init_script)?;

            command.args([
                "-NoLogo",
                "-NoExit",
                "-File",
                init_path.to_str().unwrap_or(""),
            ]);
        } else if shell_path.contains("cmd") {
            // For cmd.exe, we set the prompt via environment variable
            let prompt = format!("({}[vx]) $P$G", self.session.name);
            command.env("PROMPT", prompt);
            command.args(["/K"]);
        }

        Ok(())
    }

    #[cfg(not(windows))]
    fn configure_shell_platform(&self, command: &mut Command, shell_path: &str) -> Result<()> {
        // For bash/zsh, we can set a custom prompt
        if shell_path.contains("bash") {
            let prompt = format!("({}[vx]) \\w\\$ ", self.session.name);
            command.env("PS1", prompt);
        } else if shell_path.contains("zsh") {
            let prompt = format!("({}[vx]) %~%# ", self.session.name);
            command.env("PROMPT", prompt);
        }

        Ok(())
    }

    /// Create PowerShell initialization script
    #[cfg(windows)]
    fn create_powershell_init_script(&self) -> Result<String> {
        let tools = self.session.tools_display();
        let project_name = &self.session.name;

        // Use embedded asset for better maintainability
        ShellScript::PowerShell
            .render(project_name, &tools)
            .ok_or_else(|| anyhow::anyhow!("Failed to load PowerShell init script from assets"))
    }

    /// Execute a command in the environment
    pub fn execute_command(&self, cmd: &[String]) -> Result<ExitStatus> {
        if cmd.is_empty() {
            anyhow::bail!("No command specified");
        }

        let program = &cmd[0];
        let args = &cmd[1..];

        let mut command = Command::new(program);
        command.args(args);

        // Clear inherited environment and set our own
        command.env_clear();
        for (key, value) in &self.env_vars {
            command.env(key, value);
        }

        let status = command
            .status()
            .with_context(|| format!("Failed to execute: {}", program))?;

        Ok(status)
    }

    /// Generate environment export script
    pub fn export(&self, format: ExportFormat) -> Result<String> {
        // Extract PATH entries for export formatting
        let path = self.env_vars.get("PATH").cloned().unwrap_or_default();
        let sep = if cfg!(windows) { ";" } else { ":" };
        let current_path = env::var("PATH").unwrap_or_default();

        // Get only the new path entries (before current PATH)
        let path_entries: Vec<String> = path
            .split(sep)
            .take_while(|p| !current_path.starts_with(*p))
            .map(|s| s.to_string())
            .collect();

        // Generate output based on format
        let output = match format {
            ExportFormat::Shell => generate_shell_export(&path_entries, &self.session.env_vars),
            ExportFormat::PowerShell => {
                generate_powershell_export(&path_entries, &self.session.env_vars)
            }
            ExportFormat::Batch => generate_batch_export(&path_entries, &self.session.env_vars),
            ExportFormat::GithubActions => {
                generate_github_actions_export(&path_entries, &self.session.env_vars)
            }
        };

        Ok(output)
    }
}

/// Detect the user's preferred shell
pub fn detect_shell() -> String {
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

// Export format generators

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

/// Print welcome message for shell session
pub fn print_welcome(session: &SessionContext) {
    println!();
    println!("\x1b[32mVX Shell Environment\x1b[0m");
    println!("\x1b[36mProject: {}\x1b[0m", session.name);
    println!("\x1b[36mTools: {}\x1b[0m", session.tools_display());
    println!("\x1b[90mType 'exit' to leave the environment\x1b[0m");
    println!();
}

/// Print exit message for shell session
pub fn print_exit() {
    println!("\x1b[90mLeft vx environment\x1b[0m");
}
