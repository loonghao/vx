// Shell integration commands

use anyhow::Result;
use std::env;

pub async fn handle_shell_init(shell: Option<String>) -> Result<()> {
    let shell_type = shell.unwrap_or_else(detect_shell);

    match shell_type.as_str() {
        "bash" => print_bash_init(),
        "zsh" => print_zsh_init(),
        "fish" => print_fish_init(),
        "powershell" | "pwsh" => print_powershell_init(),
        "cmd" => print_cmd_init(),
        _ => {
            return Err(anyhow::anyhow!("Unsupported shell: {}", shell_type));
        }
    }

    Ok(())
}

pub async fn handle_completion(shell: String) -> Result<()> {
    match shell.as_str() {
        "bash" => print_bash_completion(),
        "zsh" => print_zsh_completion(),
        "fish" => print_fish_completion(),
        "powershell" | "pwsh" => print_powershell_completion(),
        _ => {
            return Err(anyhow::anyhow!("Unsupported shell: {}", shell));
        }
    }

    Ok(())
}

fn detect_shell() -> String {
    // Try to detect shell from environment variables
    if let Ok(shell) = env::var("SHELL") {
        if shell.contains("bash") {
            return "bash".to_string();
        } else if shell.contains("zsh") {
            return "zsh".to_string();
        } else if shell.contains("fish") {
            return "fish".to_string();
        }
    }

    // Check for PowerShell
    if env::var("PSModulePath").is_ok() {
        return "powershell".to_string();
    }

    // Default to bash on Unix-like systems, cmd on Windows
    if cfg!(windows) {
        "cmd".to_string()
    } else {
        "bash".to_string()
    }
}

fn print_bash_init() {
    let vx_home = dirs::home_dir()
        .map(|p| p.join(".vx").display().to_string())
        .unwrap_or_else(|| "$HOME/.vx".to_string());

    println!(
        r#"# VX Shell Integration for Bash
# Add this to your ~/.bashrc or source it directly

# Set VX environment variables
export VX_HOME="{vx_home}"
export VX_SHELL="bash"

# Add VX bin directory to PATH if not already present
if [[ ":$PATH:" != *":$VX_HOME/bin:"* ]]; then
    export PATH="$VX_HOME/bin:$PATH"
fi

# VX project detection function
__vx_detect_project() {{
    local dir="$PWD"
    while [[ "$dir" != "/" ]]; do
        if [[ -f "$dir/vx.toml" ]]; then
            export VX_PROJECT_ROOT="$dir"
            return 0
        fi
        dir="$(dirname "$dir")"
    done
    unset VX_PROJECT_ROOT
    return 1
}}

# Auto-sync on directory change
__vx_auto_sync() {{
    if __vx_detect_project && [[ -f "$VX_PROJECT_ROOT/vx.toml" ]]; then
        if command -v vx >/dev/null 2>&1; then
            vx sync --check --quiet 2>/dev/null || true
        fi
    fi
}}

# Hook into cd command
__vx_original_cd=$(declare -f cd)
cd() {{
    builtin cd "$@"
    __vx_auto_sync
}}

# Initialize on shell startup
__vx_auto_sync

# VX prompt integration (optional)
__vx_prompt() {{
    if [[ -n "$VX_PROJECT_ROOT" ]]; then
        echo "[vx]"
    fi
}}

# Uncomment to add VX info to prompt
# PS1="$(__vx_prompt)$PS1"
"#,
        vx_home = vx_home
    );
}

fn print_zsh_init() {
    let vx_home = dirs::home_dir()
        .map(|p| p.join(".vx").display().to_string())
        .unwrap_or_else(|| "$HOME/.vx".to_string());

    println!(
        r#"# VX Shell Integration for Zsh
# Add this to your ~/.zshrc or source it directly

# Set VX environment variables
export VX_HOME="{vx_home}"
export VX_SHELL="zsh"

# Add VX bin directory to PATH if not already present
if [[ ":$PATH:" != *":$VX_HOME/bin:"* ]]; then
    export PATH="$VX_HOME/bin:$PATH"
fi

# VX project detection function
__vx_detect_project() {{
    local dir="$PWD"
    while [[ "$dir" != "/" ]]; do
        if [[ -f "$dir/vx.toml" ]]; then
            export VX_PROJECT_ROOT="$dir"
            return 0
        fi
        dir="${{dir:h}}"
    done
    unset VX_PROJECT_ROOT
    return 1
}}

# Auto-sync on directory change
__vx_auto_sync() {{
    if __vx_detect_project && [[ -f "$VX_PROJECT_ROOT/vx.toml" ]]; then
        if command -v vx >/dev/null 2>&1; then
            vx sync --check --quiet 2>/dev/null || true
        fi
    fi
}}

# Hook into chpwd
autoload -U add-zsh-hook
add-zsh-hook chpwd __vx_auto_sync

# Initialize on shell startup
__vx_auto_sync

# VX prompt integration (optional)
__vx_prompt() {{
    if [[ -n "$VX_PROJECT_ROOT" ]]; then
        echo "[vx]"
    fi
}}

# Uncomment to add VX info to prompt
# PROMPT="$(__vx_prompt)$PROMPT"
"#,
        vx_home = vx_home
    );
}

fn print_fish_init() {
    let vx_home = dirs::home_dir()
        .map(|p| p.join(".vx").display().to_string())
        .unwrap_or_else(|| "$HOME/.vx".to_string());

    println!(
        r#"# VX Shell Integration for Fish
# Add this to your ~/.config/fish/config.fish or source it directly

# Set VX environment variables
set -gx VX_HOME "{vx_home}"
set -gx VX_SHELL "fish"

# Add VX bin directory to PATH if not already present
if not contains "$VX_HOME/bin" $PATH
    set -gx PATH "$VX_HOME/bin" $PATH
end

# VX project detection function
function __vx_detect_project
    set dir (pwd)
    while test "$dir" != "/"
        if test -f "$dir/vx.toml"
            set -gx VX_PROJECT_ROOT "$dir"
            return 0
        end
        set dir (dirname "$dir")
    end
    set -e VX_PROJECT_ROOT
    return 1
end

# Auto-sync on directory change
function __vx_auto_sync
    if __vx_detect_project; and test -f "$VX_PROJECT_ROOT/vx.toml"
        if command -v vx >/dev/null 2>&1
            vx sync --check --quiet 2>/dev/null; or true
        end
    end
end

# Hook into directory change
function __vx_pwd_handler --on-variable PWD
    __vx_auto_sync
end

# Initialize on shell startup
__vx_auto_sync

# VX prompt integration (optional)
function __vx_prompt
    if set -q VX_PROJECT_ROOT
        echo "[vx]"
    end
end

# Uncomment to add VX info to prompt
# function fish_prompt
#     echo (__vx_prompt)(fish_prompt)
# end
"#,
        vx_home = vx_home
    );
}

fn print_powershell_init() {
    let vx_home = dirs::home_dir()
        .map(|p| p.join(".vx").display().to_string())
        .unwrap_or_else(|| "$env:USERPROFILE\\.vx".to_string());

    println!(
        r#"# VX Shell Integration for PowerShell
# Add this to your $PROFILE or dot-source it directly

# Set VX environment variables
$env:VX_HOME = "{vx_home}"
$env:VX_SHELL = "powershell"

# Add VX bin directory to PATH if not already present
$vxBinPath = Join-Path $env:VX_HOME "bin"
if ($env:PATH -notlike "*$vxBinPath*") {{
    $env:PATH = "$vxBinPath;$env:PATH"
}}

# VX project detection function
function Find-VxProject {{
    $dir = Get-Location
    while ($dir.Path -ne $dir.Root.Name) {{
        $vxConfig = Join-Path $dir.Path "vx.toml"
        if (Test-Path $vxConfig) {{
            $env:VX_PROJECT_ROOT = $dir.Path
            return $true
        }}
        $dir = $dir.Parent
    }}
    Remove-Item Env:VX_PROJECT_ROOT -ErrorAction SilentlyContinue
    return $false
}}

# Auto-sync on directory change
function Invoke-VxAutoSync {{
    if (Find-VxProject -and (Test-Path "$env:VX_PROJECT_ROOT\vx.toml")) {{
        if (Get-Command vx -ErrorAction SilentlyContinue) {{
            try {{
                vx sync --check --quiet 2>$null
            }} catch {{
                # Ignore errors
            }}
        }}
    }}
}}

# Hook into location change
$ExecutionContext.SessionState.InvokeCommand.LocationChangedAction = {{
    Invoke-VxAutoSync
}}

# Initialize on shell startup
Invoke-VxAutoSync

# VX prompt integration (optional)
function Get-VxPrompt {{
    if ($env:VX_PROJECT_ROOT) {{
        return "[vx]"
    }}
    return ""
}}

# Uncomment to add VX info to prompt
# function prompt {{
#     "$(Get-VxPrompt)PS $($executionContext.SessionState.Path.CurrentLocation)$('>' * ($nestedPromptLevel + 1)) "
# }}
"#,
        vx_home = vx_home
    );
}

fn print_cmd_init() {
    let vx_home = "%USERPROFILE%\\.vx".to_string();

    println!(
        r#"@echo off
REM VX Shell Integration for CMD
REM Add this to your startup script or run it manually

REM Set VX environment variables
set VX_HOME={vx_home}
set VX_SHELL=cmd

REM Add VX bin directory to PATH if not already present
echo %PATH% | find /i "%VX_HOME%\bin" >nul
if errorlevel 1 (
    set PATH=%VX_HOME%\bin;%PATH%
)

echo VX Shell integration initialized for CMD
echo Use 'vx --help' to get started
"#,
        vx_home = vx_home
    );
}

fn print_bash_completion() {
    println!(
        r#"# VX Bash Completion
# Source this file or add it to /etc/bash_completion.d/

_vx_completion() {{
    local cur prev words cword
    _init_completion || return

    case $prev in
        install|remove|switch|fetch)
            # Complete with available tools
            COMPREPLY=($(compgen -W "node npm npx go cargo uv uvx python" -- "$cur"))
            return
            ;;
        --template)
            # Complete with available templates
            COMPREPLY=($(compgen -W "node python rust go fullstack minimal" -- "$cur"))
            return
            ;;
        --format)
            # Complete with output formats
            COMPREPLY=($(compgen -W "table json yaml" -- "$cur"))
            return
            ;;
        --category)
            # Complete with categories
            COMPREPLY=($(compgen -W "javascript python rust go utility" -- "$cur"))
            return
            ;;
    esac

    case ${{words[1]}} in
        venv)
            case ${{words[2]}} in
                activate|remove|use)
                    # Complete with venv names
                    local venvs=$(vx venv list --names-only 2>/dev/null || echo "")
                    COMPREPLY=($(compgen -W "$venvs" -- "$cur"))
                    return
                    ;;
                *)
                    COMPREPLY=($(compgen -W "create list activate remove current" -- "$cur"))
                    return
                    ;;
            esac
            ;;
        config)
            COMPREPLY=($(compgen -W "show set get reset edit" -- "$cur"))
            return
            ;;
        *)
            # Complete with main commands
            COMPREPLY=($(compgen -W "install remove list update search sync init cleanup stats venv config global plugin shell-init completion version help" -- "$cur"))
            return
            ;;
    esac
}}

complete -F _vx_completion vx
"#
    );
}

fn print_zsh_completion() {
    println!(
        r#"#compdef vx

# VX Zsh Completion

_vx() {{
    local context state line
    typeset -A opt_args

    _arguments -C \
        '1: :_vx_commands' \
        '*::arg:->args'

    case $state in
        args)
            case $words[1] in
                install|remove|switch|fetch)
                    _arguments \
                        '*:tools:(node npm npx go cargo uv uvx python)'
                    ;;
                venv)
                    case $words[2] in
                        activate|remove|use)
                            _arguments \
                                '*:venv:_vx_venvs'
                            ;;
                        *)
                            _arguments \
                                '1:subcommand:(create list activate remove current)'
                            ;;
                    esac
                    ;;
                config)
                    _arguments \
                        '1:subcommand:(show set get reset edit)'
                    ;;
            esac
            ;;
    esac
}}

_vx_commands() {{
    local commands
    commands=(
        'install:Install a tool'
        'remove:Remove a tool'
        'list:List installed tools'
        'update:Update tools'
        'search:Search available tools'
        'sync:Sync project tools'
        'init:Initialize project'
        'cleanup:Clean up cache and orphaned files'
        'stats:Show statistics'
        'venv:Virtual environment management'
        'config:Configuration management'
        'global:Global tool management'
        'plugin:Plugin management'
        'shell-init:Generate shell initialization script'
        'completion:Generate shell completion script'
        'version:Show version information'
        'help:Show help'
    )
    _describe 'commands' commands
}}

_vx_venvs() {{
    local venvs
    venvs=($(vx venv list --names-only 2>/dev/null || echo ""))
    _describe 'virtual environments' venvs
}}

_vx "$@"
"#
    );
}

fn print_fish_completion() {
    println!(
        r#"# VX Fish Completion

# Main commands
complete -c vx -f -n '__fish_use_subcommand' -a 'install' -d 'Install a tool'
complete -c vx -f -n '__fish_use_subcommand' -a 'remove' -d 'Remove a tool'
complete -c vx -f -n '__fish_use_subcommand' -a 'list' -d 'List installed tools'
complete -c vx -f -n '__fish_use_subcommand' -a 'update' -d 'Update tools'
complete -c vx -f -n '__fish_use_subcommand' -a 'search' -d 'Search available tools'
complete -c vx -f -n '__fish_use_subcommand' -a 'sync' -d 'Sync project tools'
complete -c vx -f -n '__fish_use_subcommand' -a 'init' -d 'Initialize project'
complete -c vx -f -n '__fish_use_subcommand' -a 'cleanup' -d 'Clean up cache and orphaned files'
complete -c vx -f -n '__fish_use_subcommand' -a 'stats' -d 'Show statistics'
complete -c vx -f -n '__fish_use_subcommand' -a 'venv' -d 'Virtual environment management'
complete -c vx -f -n '__fish_use_subcommand' -a 'config' -d 'Configuration management'
complete -c vx -f -n '__fish_use_subcommand' -a 'global' -d 'Global tool management'
complete -c vx -f -n '__fish_use_subcommand' -a 'plugin' -d 'Plugin management'
complete -c vx -f -n '__fish_use_subcommand' -a 'shell-init' -d 'Generate shell initialization script'
complete -c vx -f -n '__fish_use_subcommand' -a 'completion' -d 'Generate shell completion script'
complete -c vx -f -n '__fish_use_subcommand' -a 'version' -d 'Show version information'
complete -c vx -f -n '__fish_use_subcommand' -a 'help' -d 'Show help'

# Tool names for install/remove/switch/fetch
complete -c vx -f -n '__fish_seen_subcommand_from install remove switch fetch' -a 'node npm npx go cargo uv uvx python'

# Venv subcommands
complete -c vx -f -n '__fish_seen_subcommand_from venv; and not __fish_seen_subcommand_from create list activate remove current' -a 'create' -d 'Create virtual environment'
complete -c vx -f -n '__fish_seen_subcommand_from venv; and not __fish_seen_subcommand_from create list activate remove current' -a 'list' -d 'List virtual environments'
complete -c vx -f -n '__fish_seen_subcommand_from venv; and not __fish_seen_subcommand_from create list activate remove current' -a 'activate' -d 'Activate virtual environment'
complete -c vx -f -n '__fish_seen_subcommand_from venv; and not __fish_seen_subcommand_from create list activate remove current' -a 'remove' -d 'Remove virtual environment'
complete -c vx -f -n '__fish_seen_subcommand_from venv; and not __fish_seen_subcommand_from create list activate remove current' -a 'current' -d 'Show current virtual environment'

# Config subcommands
complete -c vx -f -n '__fish_seen_subcommand_from config; and not __fish_seen_subcommand_from show set get reset edit' -a 'show' -d 'Show configuration'
complete -c vx -f -n '__fish_seen_subcommand_from config; and not __fish_seen_subcommand_from show set get reset edit' -a 'set' -d 'Set configuration value'
complete -c vx -f -n '__fish_seen_subcommand_from config; and not __fish_seen_subcommand_from show set get reset edit' -a 'get' -d 'Get configuration value'
complete -c vx -f -n '__fish_seen_subcommand_from config; and not __fish_seen_subcommand_from show set get reset edit' -a 'reset' -d 'Reset configuration'
complete -c vx -f -n '__fish_seen_subcommand_from config; and not __fish_seen_subcommand_from show set get reset edit' -a 'edit' -d 'Edit configuration'

# Shell types for completion and shell-init
complete -c vx -f -n '__fish_seen_subcommand_from completion shell-init' -a 'bash zsh fish powershell'

# Common options
complete -c vx -l help -d 'Show help'
complete -c vx -l version -d 'Show version'
complete -c vx -l verbose -d 'Verbose output'
complete -c vx -l dry-run -d 'Preview operations'
complete -c vx -l force -d 'Force operation'
"#
    );
}

fn print_powershell_completion() {
    println!(
        r#"# VX PowerShell Completion

Register-ArgumentCompleter -Native -CommandName vx -ScriptBlock {{
    param($wordToComplete, $commandAst, $cursorPosition)

    $commands = @(
        'install', 'remove', 'list', 'update', 'search', 'sync', 'init', 'cleanup', 'stats',
        'venv', 'config', 'global', 'plugin', 'shell-init', 'completion', 'version', 'help'
    )

    $tools = @('node', 'npm', 'npx', 'go', 'cargo', 'uv', 'uvx', 'python')
    $shells = @('bash', 'zsh', 'fish', 'powershell')
    $formats = @('table', 'json', 'yaml')
    $templates = @('node', 'python', 'rust', 'go', 'fullstack', 'minimal')

    $tokens = $commandAst.CommandElements
    $command = $tokens[1].Value

    switch ($command) {{
        {{ $_ -in @('install', 'remove', 'switch', 'fetch') }} {{
            $tools | Where-Object {{ $_ -like "$wordToComplete*" }}
        }}
        'venv' {{
            if ($tokens.Count -eq 2) {{
                @('create', 'list', 'activate', 'remove', 'current') | Where-Object {{ $_ -like "$wordToComplete*" }}
            }}
        }}
        'config' {{
            if ($tokens.Count -eq 2) {{
                @('show', 'set', 'get', 'reset', 'edit') | Where-Object {{ $_ -like "$wordToComplete*" }}
            }}
        }}
        {{ $_ -in @('completion', 'shell-init') }} {{
            $shells | Where-Object {{ $_ -like "$wordToComplete*" }}
        }}
        default {{
            if ($tokens.Count -eq 1) {{
                $commands | Where-Object {{ $_ -like "$wordToComplete*" }}
            }}
        }}
    }}
}}
"#
    );
}
