# provider.star - choco provider
#
# Chocolatey - The package manager for Windows
# Inheritance pattern: Level 2 (custom download_url + script install)
#   - fetch_versions: inherited from github.star
#   - download_url:   None (installed via PowerShell script)
#
# Chocolatey is Windows-only. Installed via PowerShell script.

load("@vx//stdlib:github.star", "make_fetch_versions")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------

def name():
    return "choco"

def description():
    return "The package manager for Windows"

def homepage():
    return "https://chocolatey.org"

def repository():
    return "https://github.com/chocolatey/choco"

def license():
    return "Apache-2.0"

def ecosystem():
    return "system"

# ---------------------------------------------------------------------------
# Platform constraint: Windows-only
# ---------------------------------------------------------------------------

def supported_platforms():
    return [
        {"os": "windows", "arch": "x64"},
        {"os": "windows", "arch": "x86"},
        {"os": "windows", "arch": "arm64"},
    ]

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    {
        "name":        "choco",
        "executable":  "choco",
        "description": "Chocolatey package manager",
        "aliases":     ["chocolatey"],
        "priority":    100,
        "system_paths": [
            "C:/ProgramData/chocolatey/bin/choco.exe",
            "C:/ProgramData/chocolatey/choco.exe",
        ],
    },
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = {
    "http": ["api.github.com", "github.com", "community.chocolatey.org"],
    "fs":   ["C:/ProgramData/chocolatey"],
    "exec": ["powershell", "pwsh"],
}

# ---------------------------------------------------------------------------
# fetch_versions — inherited from GitHub releases
# ---------------------------------------------------------------------------

fetch_versions = make_fetch_versions("chocolatey", "choco")

# ---------------------------------------------------------------------------
# download_url — None (installed via PowerShell script)
# ---------------------------------------------------------------------------

def download_url(ctx, version):
    """Chocolatey is installed via PowerShell script, not direct download."""
    return None

# ---------------------------------------------------------------------------
# script_install — PowerShell installation
# ---------------------------------------------------------------------------

def script_install(ctx):
    """Return the PowerShell install command for Chocolatey."""
    return {
        "command": "Set-ExecutionPolicy Bypass -Scope Process -Force; [System.Net.ServicePointManager]::SecurityProtocol = [System.Net.ServicePointManager]::SecurityProtocol -bor 3072; iex ((New-Object System.Net.WebClient).DownloadString('https://community.chocolatey.org/install.ps1'))",
        "shell": "powershell",
    }

# ---------------------------------------------------------------------------
# environment
# ---------------------------------------------------------------------------

def environment(ctx, version, install_dir):
    return {
        "PATH":            "C:/ProgramData/chocolatey/bin",
        "ChocolateyInstall": "C:/ProgramData/chocolatey",
    }
