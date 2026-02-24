# @vx//stdlib:script_install.star
# Script-based install helpers for vx provider scripts
#
# This module provides helpers for declaring script_install functions that
# run shell scripts to install tools (curl | bash, PowerShell irm | iex, etc.).
#
# ┌─────────────────────────────────────────────────────────────────────────┐
# │  curl_bash_install()      curl | bash pattern (Unix)                    │
# │  curl_sh_install()        curl | sh pattern (Unix, POSIX)               │
# │  irm_iex_install()        PowerShell iex(irm) pattern (Windows)         │
# │  irm_install()            PowerShell irm | iex modern pattern (Windows) │
# │  platform_script_install() Dispatch unix/windows script by OS           │
# └─────────────────────────────────────────────────────────────────────────┘

# ---------------------------------------------------------------------------
# Unix script install helpers
# ---------------------------------------------------------------------------

def curl_bash_install(url, post_install_cmds = None):
    """Build a script_install function using the classic curl | bash pattern.

    This is the most common Unix install pattern:
        /bin/bash -c "$(curl -fsSL <url>)"

    Used by: Homebrew, Rust (rustup), nvm, oh-my-zsh, etc.

    Args:
        url:               URL of the install script
        post_install_cmds: Optional list of shell commands to run after
                           the install script completes (e.g. shellenv evals)

    Returns:
        A function: script_install(ctx) -> dict

    Example:
        # Homebrew
        script_install = curl_bash_install(
            "https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh",
            post_install_cmds = [
                'eval "$(/opt/homebrew/bin/brew shellenv)"',
                'eval "$(/usr/local/bin/brew shellenv)"',
            ],
        )

        # Rust / rustup
        script_install = curl_bash_install(
            "https://sh.rustup.rs",
        )
    """
    def _script_install(_ctx):
        result = {
            "command": '/bin/bash -c "$(curl -fsSL ' + url + ')"',
            "shell":   "bash",
        }
        if post_install_cmds != None:
            result["post_install"] = post_install_cmds
        return result
    return _script_install

def curl_sh_install(url, post_install_cmds = None):
    """Build a script_install function using the curl | sh pattern.

    Simpler variant of curl_bash_install that uses /bin/sh instead of bash.
    Used by tools that want POSIX-compatible install scripts.

    Args:
        url:               URL of the install script
        post_install_cmds: Optional list of shell commands to run after

    Returns:
        A function: script_install(ctx) -> dict

    Example:
        # Generic POSIX install
        script_install = curl_sh_install(
            "https://example.com/install.sh",
        )
    """
    def _script_install(_ctx):
        result = {
            "command": 'curl -fsSL ' + url + ' | sh',
            "shell":   "sh",
        }
        if post_install_cmds != None:
            result["post_install"] = post_install_cmds
        return result
    return _script_install

# ---------------------------------------------------------------------------
# Windows PowerShell script install helpers
# ---------------------------------------------------------------------------

def irm_iex_install(url, env_vars = None, pre_commands = None,
                    post_install_cmds = None):
    """Build a script_install function using PowerShell irm | iex pattern.

    This is the standard Windows PowerShell install pattern:
        iex ((New-Object System.Net.WebClient).DownloadString('<url>'))

    Used by: Chocolatey, Scoop, oh-my-posh, etc.

    Args:
        url:               URL of the PowerShell install script
        env_vars:          Dict of environment variables to set before running
                           the script (e.g. {"ChocolateyInstall": install_dir})
                           Values may contain the special token "{install_dir}"
                           which will be replaced with ctx.install_dir at
                           runtime.
        pre_commands:      Extra PowerShell commands to prepend (list of str)
        post_install_cmds: Extra PowerShell commands to append (list of str)

    Returns:
        A function: script_install(ctx) -> dict

    Example:
        # Chocolatey (sets CHOCOLATEY_INSTALL to vx store path)
        script_install = irm_iex_install(
            "https://community.chocolatey.org/install.ps1",
            env_vars = {"ChocolateyInstall": "{install_dir}"},
        )

        # Scoop
        script_install = irm_iex_install(
            "https://get.scoop.sh",
        )

        # oh-my-posh
        script_install = irm_iex_install(
            "https://ohmyposh.dev/install.ps1",
        )
    """
    def _script_install(ctx):
        parts = []

        # Security protocol (required for older PowerShell / TLS 1.2)
        parts.append(
            "[System.Net.ServicePointManager]::SecurityProtocol = " +
            "[System.Net.ServicePointManager]::SecurityProtocol -bor 3072"
        )
        parts.append("Set-ExecutionPolicy Bypass -Scope Process -Force")

        # Environment variables
        if env_vars != None:
            for k, v in env_vars.items():
                resolved = v.replace("{install_dir}", ctx.install_dir)
                parts.append("$env:" + k + " = '" + resolved + "'")

        # Extra pre-commands
        if pre_commands != None:
            for cmd in pre_commands:
                parts.append(cmd)

        # The actual install
        parts.append("iex ((New-Object System.Net.WebClient).DownloadString('" + url + "'))")

        # Post-install commands
        if post_install_cmds != None:
            for cmd in post_install_cmds:
                parts.append(cmd)

        return {
            "command": "; ".join(parts),
            "shell":   "powershell",
        }
    return _script_install

def irm_install(url, env_vars = None, post_install_cmds = None):
    """Build a script_install function using the modern PowerShell irm | iex pattern.

    Uses the modern `irm` (Invoke-RestMethod) shorthand instead of
    New-Object WebClient. Requires PowerShell 5+ / pwsh.

    Args:
        url:               URL of the PowerShell install script
        env_vars:          Dict of environment variables to set before running
        post_install_cmds: Extra PowerShell commands to append

    Returns:
        A function: script_install(ctx) -> dict

    Example:
        # Scoop (modern)
        script_install = irm_install("https://get.scoop.sh")

        # winget (modern)
        script_install = irm_install("https://aka.ms/getwinget")
    """
    def _script_install(ctx):
        parts = []
        parts.append("Set-ExecutionPolicy Bypass -Scope Process -Force")

        if env_vars != None:
            for k, v in env_vars.items():
                resolved = v.replace("{install_dir}", ctx.install_dir)
                parts.append("$env:" + k + " = '" + resolved + "'")

        parts.append("irm '" + url + "' | iex")

        if post_install_cmds != None:
            for cmd in post_install_cmds:
                parts.append(cmd)

        return {
            "command": "; ".join(parts),
            "shell":   "powershell",
        }
    return _script_install

# ---------------------------------------------------------------------------
# Cross-platform script install dispatcher
# ---------------------------------------------------------------------------

def platform_script_install(unix = None, windows = None):
    """Build a script_install function that dispatches by OS.

    Use this when a tool has different install scripts for Unix vs Windows.

    Args:
        unix:    script_install function (or result dict) for macOS/Linux
                 Typically built with curl_bash_install() or curl_sh_install()
        windows: script_install function (or result dict) for Windows
                 Typically built with irm_iex_install() or irm_install()

    Returns:
        A function: script_install(ctx) -> dict

    Example:
        # A tool with both Unix and Windows install scripts
        script_install = platform_script_install(
            unix    = curl_bash_install("https://example.com/install.sh"),
            windows = irm_iex_install("https://example.com/install.ps1"),
        )
    """
    def _script_install(ctx):
        os = ctx.platform.os
        if os == "windows" and windows != None:
            if type(windows) == "function":
                return windows(ctx)
            return windows
        elif os != "windows" and unix != None:
            if type(unix) == "function":
                return unix(ctx)
            return unix
        return {}
    return _script_install
