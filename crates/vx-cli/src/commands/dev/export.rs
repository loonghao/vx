//! Environment export functionality for various formats

use crate::commands::setup::ConfigView;
use crate::commands::dev::tools::get_registry;
use anyhow::Result;
use std::collections::HashMap;
use std::env;
use vx_env::ToolEnvironment;

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
pub async fn handle_export(config: &ConfigView, format: Option<String>) -> Result<()> {
    let export_format = match format {
        Some(f) => ExportFormat::parse(&f).ok_or_else(|| {
            anyhow::anyhow!(
                "Unknown format: {}. Use: shell, powershell, batch, or github",
                f
            )
        })?,
        None => ExportFormat::detect(),
    };

    let output = generate_env_export(config, export_format).await?;
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
pub async fn generate_env_export(config: &ConfigView, format: ExportFormat) -> Result<String> {
    // Merge env from vx.toml with setenv from settings
    let mut env_vars = config.env.clone();
    env_vars.extend(config.setenv.clone());

    // Get registry to query runtime bin directories
    let (registry, context) = get_registry()?;

    // Create ToolSpecs with proper bin directories from runtime providers
    let mut tool_specs = Vec::new();
    for (tool_name, version) in &config.tools {
        // Find the runtime for this tool to get bin directories
        let (bin_dirs, resolved_bin_dir) = if let Some(provider) = registry.providers().iter().find(|p| p.supports(tool_name)) {
            if let Some(runtime) = provider.get_runtime(tool_name) {
                // Call prepare_environment to get runtime-specific environment variables
                if let Ok(runtime_env) = runtime.prepare_environment(version, &context).await {
                    // Merge runtime-specific environment variables (e.g., MSVC's INCLUDE, LIB)
                    for (key, value) in runtime_env {
                        env_vars.insert(key, value);
                    }
                }
                
                // Try to get the resolved bin directory from the runtime
                let resolved = if let Ok(Some(exe_path)) = runtime.get_executable_path_for_version(version, &context).await {
                    // Get the parent directory of the executable as the bin directory
                    exe_path.parent().map(|p| p.to_path_buf())
                } else {
                    None
                };
                
                let dirs = runtime.possible_bin_dirs().into_iter().map(|s| s.to_string()).collect();
                (dirs, resolved)
            } else {
                (vec!["bin".to_string()], None)
            }
        } else {
            (vec!["bin".to_string()], None)
        };

        let mut spec = vx_env::ToolSpec::with_bin_dirs(tool_name.clone(), version.clone(), bin_dirs);
        if let Some(bin_dir) = resolved_bin_dir {
            spec = spec.set_resolved_bin_dir(bin_dir);
        }
        tool_specs.push(spec);
    }

    // Build environment using ToolEnvironment
    let all_env_vars = ToolEnvironment::new()
        .tools_from_specs(tool_specs)
        .env_vars(&env_vars)
        .warn_missing(false)
        .build()?;

    // Extract PATH entries for export formatting
    let path = all_env_vars.get("PATH").cloned().unwrap_or_default();
    let sep = if cfg!(windows) { ";" } else { ":" };
    let current_path = std::env::var("PATH").unwrap_or_default();

    // Get only the new path entries (the ones we added, not from system PATH)
    // The new entries are at the beginning of PATH before the original PATH
    let current_path_entries: std::collections::HashSet<&str> = 
        current_path.split(sep).collect();
    
    let path_entries: Vec<String> = path
        .split(sep)
        .filter(|p| !p.is_empty() && !current_path_entries.contains(*p))
        .map(|s| s.to_string())
        .collect();

    // Generate output based on format
    let output = match format {
        ExportFormat::Shell => generate_shell_export(&path_entries, &all_env_vars),
        ExportFormat::PowerShell => generate_powershell_export(&path_entries, &all_env_vars),
        ExportFormat::Batch => generate_batch_export(&path_entries, &all_env_vars),
        ExportFormat::GithubActions => generate_github_actions_export(&path_entries, &all_env_vars),
    };

    Ok(output)
}

fn generate_shell_export(path_entries: &[String], env_vars: &HashMap<String, String>) -> String {
    let mut output = String::new();

    // Header comment
    output.push_str("# VX Environment Activation Script\n");
    output.push_str("# Usage: eval \"$(vx dev --export)\"\n\n");

    // Define deactivate function (similar to venv)
    output.push_str(r#"# Deactivate function to restore previous environment
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

"#);

    // Save old environment (if not already in a vx environment)
    output.push_str(r#"# Save current environment (only if not already activated)
if [ -z "${VX_DEV:-}" ]; then
    export _OLD_VX_PATH="$PATH"
    export _OLD_VX_PS1="${PS1:-}"
fi

"#);

    // Export PATH
    if !path_entries.is_empty() {
        let paths = path_entries.join(":");
        output.push_str(&format!("export PATH=\"{}:$PATH\"\n", paths));
    }

    // Export VX-specific environment variables
    output.push_str("\n# VX environment variables\n");
    output.push_str("export VX_DEV=1\n");

    // Export custom environment variables (filter out PATH as it's handled above)
    let project_name = env_vars.get("VX_PROJECT_NAME").cloned().unwrap_or_default();
    for (key, value) in env_vars {
        if key == "PATH" {
            continue; // Already handled
        }
        // Escape special characters in value
        let escaped = value.replace('\\', "\\\\").replace('"', "\\\"");
        output.push_str(&format!("export {}=\"{}\"\n", key, escaped));
    }

    // Update prompt to show vx environment
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

    // Header comment
    output.push_str("# VX Environment Activation Script for PowerShell\n");
    output.push_str("# Usage: Invoke-Expression (vx dev --export --format powershell)\n\n");

    // Define deactivate function (similar to venv's activate.ps1)
    output.push_str(r#"# Deactivate function to restore previous environment
function global:Vx-Deactivate {
    [CmdletBinding()]
    param([switch]$NonDestructive)

    # Restore old PATH
    if (Test-Path variable:global:_OLD_VX_PATH) {
        $env:PATH = $global:_OLD_VX_PATH
        Remove-Variable -Name _OLD_VX_PATH -Scope Global
    }

    # Restore old prompt
    if (Test-Path function:global:_old_vx_prompt) {
        $function:prompt = $function:_old_vx_prompt
        Remove-Item function:\_old_vx_prompt -ErrorAction SilentlyContinue
    }

    # Unset VX environment variables
    Remove-Item env:VX_DEV -ErrorAction SilentlyContinue
    Remove-Item env:VX_PROJECT_NAME -ErrorAction SilentlyContinue
    Remove-Item env:VX_PROJECT_ROOT -ErrorAction SilentlyContinue

    if (-not $NonDestructive) {
        # Self destruct
        Remove-Item function:Vx-Deactivate -ErrorAction SilentlyContinue
    }
}

"#);

    // Save old environment (only if not already activated)
    output.push_str(r#"# Save current environment (only if not already activated)
if (-not $env:VX_DEV) {
    $global:_OLD_VX_PATH = $env:PATH

    # Save old prompt function
    if (Test-Path function:prompt) {
        $function:global:_old_vx_prompt = $function:prompt
    }
}

"#);

    // Export PATH
    if !path_entries.is_empty() {
        let paths = path_entries.join(";");
        // Don't double-escape backslashes on Windows paths
        output.push_str(&format!("$env:PATH = \"{};$env:PATH\"\n", paths));
    }

    // Export VX-specific environment variables
    output.push_str("\n# VX environment variables\n");
    output.push_str("$env:VX_DEV = \"1\"\n");

    // Export custom environment variables (filter out PATH)
    let project_name = env_vars.get("VX_PROJECT_NAME").cloned().unwrap_or_default();
    for (key, value) in env_vars {
        if key == "PATH" {
            continue; // Already handled
        }
        // PowerShell uses backtick for escaping
        let escaped = value.replace('"', "`\"");
        output.push_str(&format!("$env:{} = \"{}\"\n", key, escaped));
    }

    // Update prompt to show vx environment
    if !project_name.is_empty() {
        output.push_str(&format!(
            r#"
# Update prompt to show vx environment
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

    // Header comment
    output.push_str("@REM VX Environment Activation Script for CMD\n");
    output.push_str("@REM Usage: vx dev --export --format batch > activate.bat && activate.bat\n\n");

    // Save old PATH (only if not already activated)
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

    // Export VX-specific environment variables
    output.push_str("\n@REM VX environment variables\n");
    output.push_str("@set VX_DEV=1\n");

    // Export custom environment variables (filter out PATH)
    let project_name = env_vars.get("VX_PROJECT_NAME").cloned().unwrap_or_default();
    for (key, value) in env_vars {
        if key == "PATH" {
            continue; // Already handled
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

    // Add deactivate instructions
    output.push_str("\n@REM To deactivate, run: vx_deactivate.bat\n");
    output.push_str("@REM Or manually restore: set PATH=%_OLD_VX_PATH%\n");

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
