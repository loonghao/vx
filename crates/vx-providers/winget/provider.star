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

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------

def name():
    return "winget"

def description():
    return "Windows Package Manager - Official package manager for Windows"

def homepage():
    return "https://learn.microsoft.com/windows/package-manager/"

def repository():
    return "https://github.com/microsoft/winget-cli"

def license():
    return "MIT"

def ecosystem():
    return "system"

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
    {
        "name":        "winget",
        "executable":  "winget",
        "description": "Windows Package Manager command-line tool",
        "aliases":     ["winget-cli"],
        "priority":    100,
        "system_paths": [
            "C:/Users/*/AppData/Local/Microsoft/WindowsApps/winget.exe",
        ],
    },
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = {
    "http": ["api.github.com", "github.com"],
    "fs":   [],
    "exec": ["powershell", "pwsh"],
}

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
    os = ctx["platform"]["os"]
    if os != "windows":
        return None

    # winget releases: Microsoft.DesktopAppInstaller_{version}_8wekyb3d8bbwe.msixbundle
    asset = "Microsoft.DesktopAppInstaller_{}_8wekyb3d8bbwe.msixbundle".format(version)
    return github_asset_url("microsoft", "winget-cli", "v" + version, asset)

# ---------------------------------------------------------------------------
# script_install — PowerShell installation
# ---------------------------------------------------------------------------

def script_install(ctx):
    """Return the PowerShell install command for winget."""
    return {
        "command": """
$progressPreference = 'silentlyContinue'
$latestWingetMsixBundleUri = $(Invoke-RestMethod https://api.github.com/repos/microsoft/winget-cli/releases/latest).assets.browser_download_url | Where-Object {$_.EndsWith(".msixbundle")}
$latestWingetMsixBundle = $latestWingetMsixBundleUri.Split("/")[-1]
Invoke-WebRequest -Uri $latestWingetMsixBundleUri -OutFile "./$latestWingetMsixBundle"
Add-AppxPackage -Path "./$latestWingetMsixBundle"
""",
        "shell": "powershell",
    }

# ---------------------------------------------------------------------------
# environment
# ---------------------------------------------------------------------------

def environment(ctx, version, install_dir):
    return {}
