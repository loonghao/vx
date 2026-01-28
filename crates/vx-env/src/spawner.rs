//! Shell spawner for vx environments
//!
//! This module provides unified shell spawning functionality used by both
//! `vx dev` and `vx env shell` commands.

#[cfg(windows)]
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

        builder = builder.env_var("VX_PROJECT_NAME", session.prompt_name());
        builder = builder.env_var("VX_DEV", "1");

        builder.build().map_err(|e| anyhow::anyhow!("{}", e))
    }

    /// Install completion scripts for the shell
    fn install_completion_scripts(&self, shell_path: &str) -> Result<()> {
        let data_dir = Self::get_data_dir()?;
        std::fs::create_dir_all(&data_dir)?;

        // Install completion scripts based on shell type
        if shell_path.contains("bash") {
            self.install_bash_completion(&data_dir)?;
        } else if shell_path.contains("zsh") {
            self.install_zsh_completion(&data_dir)?;
        } else if shell_path.contains("powershell") || shell_path.contains("pwsh") {
            self.install_powershell_completion(&data_dir)?;
        }

        Ok(())
    }

    /// Get the vx data directory for storing completion scripts and history
    fn get_data_dir() -> Result<std::path::PathBuf> {
        if cfg!(windows) {
            Ok(dirs::data_local_dir()
                .ok_or_else(|| anyhow::anyhow!("Failed to get local data directory"))?
                .join("vx"))
        } else {
            if let Ok(xdg_data) = env::var("XDG_DATA_HOME") {
                Ok(std::path::PathBuf::from(xdg_data).join("vx"))
            } else {
                let home = env::var("HOME")
                    .map_err(|_| anyhow::anyhow!("HOME environment variable not set"))?;
                Ok(std::path::PathBuf::from(home).join(".local/share/vx"))
            }
        }
    }

    /// Install Bash completion script
    fn install_bash_completion(&self, data_dir: &std::path::Path) -> Result<()> {
        let completion_path = data_dir.join("vx_completion.bash");

        // Check if completion script already exists and is up-to-date
        if completion_path.exists() {
            // Optionally check if it needs updating (compare version or content)
            return Ok(());
        }

        // Get completion script from embedded assets
        let completion_content = crate::assets::CompletionScript::Bash
            .get_raw()
            .ok_or_else(|| anyhow::anyhow!("Failed to load Bash completion script"))?;

        // Write completion script
        std::fs::write(&completion_path, completion_content).with_context(|| {
            format!(
                "Failed to write completion script to {}",
                completion_path.display()
            )
        })?;

        Ok(())
    }

    /// Install Zsh completion script
    fn install_zsh_completion(&self, data_dir: &std::path::Path) -> Result<()> {
        let completion_path = data_dir.join("vx_completion.zsh");

        // Check if completion script already exists and is up-to-date
        if completion_path.exists() {
            return Ok(());
        }

        // Get completion script from embedded assets
        let completion_content = crate::assets::CompletionScript::Zsh
            .get_raw()
            .ok_or_else(|| anyhow::anyhow!("Failed to load Zsh completion script"))?;

        // Write completion script
        std::fs::write(&completion_path, completion_content).with_context(|| {
            format!(
                "Failed to write completion script to {}",
                completion_path.display()
            )
        })?;

        Ok(())
    }

    /// Install PowerShell completion script
    fn install_powershell_completion(&self, data_dir: &std::path::Path) -> Result<()> {
        let completion_path = data_dir.join("vx_completion.ps1");

        // Check if completion script already exists and is up-to-date
        if completion_path.exists() {
            return Ok(());
        }

        // Get completion script from embedded assets
        let completion_content = crate::assets::CompletionScript::PowerShell
            .get_raw()
            .ok_or_else(|| anyhow::anyhow!("Failed to load PowerShell completion script"))?;

        // Write completion script
        std::fs::write(&completion_path, completion_content).with_context(|| {
            format!(
                "Failed to write completion script to {}",
                completion_path.display()
            )
        })?;

        Ok(())
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
        let shell_path = shell.map(|s| s.to_string()).unwrap_or_else(detect_shell);

        // Install completion scripts if needed
        self.install_completion_scripts(&shell_path)?;

        let mut command = Command::new(&shell_path);

        // IMPORTANT: Clear inherited environment to ensure our PATH takes effect
        // Without this, the shell inherits the parent's PATH which may not include vx tools
        command.env_clear();

        // Set all environment variables from our built environment
        for (key, value) in &self.env_vars {
            command.env(key, value);
        }

        // Set VX_PROJECT_NAME for prompt customization
        command.env("VX_PROJECT_NAME", self.session.prompt_name());

        // Platform-specific shell configuration
        self.configure_shell_platform(&mut command, &shell_path)?;

        #[cfg(windows)]
        {
            // On Windows, wait for the shell to exit just like Unix
            // This ensures the user stays in the dev environment until they exit
            let status = command.status().with_context(|| {
                format!(
                    "Failed to spawn shell: {}. Try specifying a shell with --shell",
                    shell_path
                )
            })?;

            Ok(status)
        }

        #[cfg(not(windows))]
        {
            // On Unix, use status() as normal
            let status = command.status().with_context(|| {
                format!(
                    "Failed to spawn shell: {}. Try specifying a shell with --shell",
                    shell_path
                )
            })?;

            Ok(status)
        }
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
            let prompt = format!("({}[vx]) $P$G", self.session.prompt_name());
            command.env("PROMPT", prompt);
            command.args(["/K"]);
        }

        Ok(())
    }

    #[cfg(not(windows))]
    fn configure_shell_platform(&self, command: &mut Command, shell_path: &str) -> Result<()> {
        // For bash/zsh, we can set a custom prompt
        if shell_path.contains("bash") {
            let prompt = format!("({}[vx]) \\w\\$ ", self.session.prompt_name());
            command.env("PS1", prompt);
        } else if shell_path.contains("zsh") {
            let prompt = format!("({}[vx]) %~%# ", self.session.prompt_name());
            command.env("PROMPT", prompt);
        }

        Ok(())
    }

    /// Create PowerShell initialization script
    #[cfg(windows)]
    fn create_powershell_init_script(&self) -> Result<String> {
        let tools = self.session.tools_display();
        let project_name = self.session.prompt_name();

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

    // Header comment
    output.push_str("# VX Environment Activation Script\n");
    output.push_str("# Usage: eval \"$(vx dev --export)\"\n\n");

    // Define deactivate function (similar to venv)
    output.push_str(
        r#"# Deactivate function to restore previous environment
vx_deactivate() {
    # Restore old PATH
    if [ -n "${_OLD_VX_PATH:-}" ]; then
        export PATH="$_OLD_VX_PATH"
        unset _OLD_VX_PATH
    fi

    # Restore old PS1 prompt
    if [ -n "${_OLD_VX_PS1:-}" ]; then
        export PS1="$_OLD_VX_PS1"
        unset _OLD_VX_PS1
    fi

    # Unset VX environment variables
    unset VX_DEV
    unset VX_PROJECT_NAME
    unset VX_PROJECT_ROOT

    # Self-destruct
    unset -f vx_deactivate
}

"#,
    );

    // Save old environment (if not already in a vx environment)
    output.push_str(
        r#"# Save current environment (only if not already activated)
if [ -z "${VX_DEV:-}" ]; then
    export _OLD_VX_PATH="$PATH"
    export _OLD_VX_PS1="${PS1:-}"
fi

"#,
    );

    // Export PATH
    if !path_entries.is_empty() {
        let paths = path_entries.join(":");
        output.push_str(&format!("export PATH=\"{}:$PATH\"\n", paths));
    }

    // Export VX-specific environment variables
    output.push_str("\n# VX environment variables\n");
    output.push_str("export VX_DEV=1\n");

    // Export custom environment variables (filter out PATH)
    let project_name = env_vars.get("VX_PROJECT_NAME").cloned().unwrap_or_default();
    for (key, value) in env_vars {
        if key == "PATH" {
            continue;
        }
        let escaped = value.replace('\\', "\\\\").replace('"', "\\\"");
        output.push_str(&format!("export {}=\"{}\"\n", key, escaped));
    }

    // Update prompt
    if !project_name.is_empty() {
        output.push_str(&format!(
            "\n# Update prompt\nexport PS1=\"({}[vx]) ${{_OLD_VX_PS1:-\\w\\$ }}\"\n",
            project_name
        ));
    }

    output.push_str("\n# Type 'vx_deactivate' to exit the vx environment\n");

    output
}

fn generate_powershell_export(
    path_entries: &[String],
    env_vars: &HashMap<String, String>,
) -> String {
    let mut output = String::new();

    // Header
    output.push_str("# VX Environment Activation Script for PowerShell\n");
    output.push_str("# Usage: Invoke-Expression (vx dev --export --format powershell)\n\n");

    // Define deactivate function
    output.push_str(
        r#"# Deactivate function
function global:Vx-Deactivate {
    [CmdletBinding()]
    param([switch]$NonDestructive)

    if (Test-Path variable:global:_OLD_VX_PATH) {
        $env:PATH = $global:_OLD_VX_PATH
        Remove-Variable -Name _OLD_VX_PATH -Scope Global
    }

    if (Test-Path function:global:_old_vx_prompt) {
        $function:prompt = $function:_old_vx_prompt
        Remove-Item function:\_old_vx_prompt -ErrorAction SilentlyContinue
    }

    Remove-Item env:VX_DEV -ErrorAction SilentlyContinue
    Remove-Item env:VX_PROJECT_NAME -ErrorAction SilentlyContinue
    Remove-Item env:VX_PROJECT_ROOT -ErrorAction SilentlyContinue

    if (-not $NonDestructive) {
        Remove-Item function:Vx-Deactivate -ErrorAction SilentlyContinue
    }
}

"#,
    );

    // Save old environment
    output.push_str(
        r#"# Save current environment
if (-not $env:VX_DEV) {
    $global:_OLD_VX_PATH = $env:PATH
    if (Test-Path function:prompt) {
        $function:global:_old_vx_prompt = $function:prompt
    }
}

"#,
    );

    // Export PATH
    if !path_entries.is_empty() {
        let paths = path_entries.join(";");
        output.push_str(&format!("$env:PATH = \"{};$env:PATH\"\n", paths));
    }

    // Export VX environment variables
    output.push_str("\n# VX environment variables\n");
    output.push_str("$env:VX_DEV = \"1\"\n");

    let project_name = env_vars.get("VX_PROJECT_NAME").cloned().unwrap_or_default();
    for (key, value) in env_vars {
        if key == "PATH" {
            continue;
        }
        let escaped = value.replace('"', "`\"");
        output.push_str(&format!("$env:{} = \"{}\"\n", key, escaped));
    }

    // Update prompt
    if !project_name.is_empty() {
        output.push_str(&format!(
            r#"
# Update prompt
function global:prompt {{
    $previous_prompt = if (Test-Path function:global:_old_vx_prompt) {{
        & $function:_old_vx_prompt
    }} else {{
        "PS $($executionContext.SessionState.Path.CurrentLocation)$('>' * ($nestedPromptLevel + 1)) "
    }}
    "({}[vx]) $previous_prompt"
}}
"#,
            project_name
        ));
    }

    output.push_str("\n# Type 'Vx-Deactivate' to exit the vx environment\n");

    output
}

fn generate_batch_export(path_entries: &[String], env_vars: &HashMap<String, String>) -> String {
    let mut output = String::new();

    // Header
    output.push_str("@REM VX Environment Activation Script for CMD\n\n");

    // Save old PATH
    output.push_str("@REM Save current environment\n");
    output.push_str("@if not defined VX_DEV (\n");
    output.push_str("    @set \"_OLD_VX_PATH=%PATH%\"\n");
    output.push_str("    @set \"_OLD_VX_PROMPT=%PROMPT%\"\n");
    output.push_str(")\n\n");

    // Export PATH
    if !path_entries.is_empty() {
        let paths = path_entries.join(";");
        output.push_str(&format!("@set \"PATH={};%PATH%\"\n", paths));
    }

    // VX environment variables
    output.push_str("\n@REM VX environment variables\n");
    output.push_str("@set VX_DEV=1\n");

    let project_name = env_vars.get("VX_PROJECT_NAME").cloned().unwrap_or_default();
    for (key, value) in env_vars {
        if key == "PATH" {
            continue;
        }
        output.push_str(&format!("@set \"{}={}\"\n", key, value));
    }

    // Update prompt
    if !project_name.is_empty() {
        output.push_str(&format!(
            "\n@REM Update prompt\n@set \"PROMPT=({}[vx]) $P$G\"\n",
            project_name
        ));
    }

    output.push_str("\n@REM To deactivate: set PATH=%_OLD_VX_PATH%\n");

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
