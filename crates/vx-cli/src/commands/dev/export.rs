//! Environment export functionality for various formats

use std::collections::HashMap;
use std::env;
use anyhow::Result;
use vx_env::ToolEnvironment;
use crate::commands::setup::ConfigView;

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

/// Handle --export mode: output shell script for environment activation
pub fn handle_export(config: &ConfigView, format: Option<String>) -> Result<()> {
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
