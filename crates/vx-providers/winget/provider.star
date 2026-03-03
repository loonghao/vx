# provider.star - winget provider
#
# Windows Package Manager - Official package manager for Windows
# Inheritance pattern: Level 2 (custom download_url for msixbundle)
#   - fetch_versions: inherited from github.star
#   - download_url:   custom (msixbundle from GitHub releases)
#
# winget is Windows-only. Pre-installed on Windows 11, available via
# App Installer on Windows 10.

load("@vx//stdlib:github.star", "make_fetch_versions", "github_asset_url")
load("@vx//stdlib:provider.star", "runtime_def", "github_permissions", "irm_install")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "winget"
description = "Windows Package Manager - Official package manager for Windows"
homepage    = "https://learn.microsoft.com/windows/package-manager/"
repository  = "https://github.com/microsoft/winget-cli"
license     = "MIT"
ecosystem   = "system"

# ---------------------------------------------------------------------------
# Platform constraint: Windows-only
# ---------------------------------------------------------------------------

def supported_platforms():
    return [
        {"os": "windows", "arch": "x64"},
        {"os": "windows", "arch": "arm64"},
    ]

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    runtime_def("winget",
        aliases     = ["winget-cli"],
        description = "Windows Package Manager command-line tool",
    ),
]
# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = github_permissions(exec_cmds = ["powershell", "pwsh"])

# ---------------------------------------------------------------------------
# fetch_versions — inherited from GitHub releases
# ---------------------------------------------------------------------------

fetch_versions = make_fetch_versions("microsoft", "winget-cli")

# ---------------------------------------------------------------------------
# download_url — msixbundle from GitHub releases
#
# winget releases include a .msixbundle file that can be installed via
# Add-AppxPackage in PowerShell.
# ---------------------------------------------------------------------------

def download_url(ctx, version):
    """Build the winget msixbundle download URL.

    Args:
        ctx:     Provider context
        version: Version string, e.g. "1.9.25200"

    Returns:
        Download URL string, or None if not Windows
    """
    os = ctx.platform.os
    if os != "windows":
        return None

    # winget releases: Microsoft.DesktopAppInstaller_{version}_8wekyb3d8bbwe.msixbundle
    asset = "Microsoft.DesktopAppInstaller_{}_8wekyb3d8bbwe.msixbundle".format(version)
    return github_asset_url("microsoft", "winget-cli", "v" + version, asset)

# ---------------------------------------------------------------------------
# script_install — PowerShell irm | iex (modern winget install)
# ---------------------------------------------------------------------------

# winget is pre-installed on Windows 11; this handles Windows 10 installs.
script_install = irm_install("https://aka.ms/getwinget")

# ---------------------------------------------------------------------------
# Path queries (RFC-0037)
# ---------------------------------------------------------------------------

def store_root(ctx):
    """Return the vx store root directory for winget.

    winget is a Windows system tool; it is not directly installed by vx.
    """
    return ctx.vx_home + "/store/winget"

def get_execute_path(_ctx, _version):
    """Return the executable path for winget (Windows-only)."""
    return "C:/Users/*/AppData/Local/Microsoft/WindowsApps/winget.exe"

def post_install(_ctx, _version):
    """No post-install actions needed — system package via msixbundle."""
    return None

# ---------------------------------------------------------------------------
# environment
# ---------------------------------------------------------------------------

def environment(_ctx, _version):
    return []
