# provider.star - Azure CLI provider
#
# Version source: https://github.com/Azure/azure-cli/releases
#
# Azure CLI uses platform-specific installers (.msi on Windows).
# vx prefers system package managers; Linux supports direct tar.gz download.
#
# Inheritance pattern: Level 2 (custom download_url + system_install)

load("@vx//stdlib:github.star", "make_fetch_versions")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------

def name():
    return "azcli"

def description():
    return "Azure CLI - Command-line interface for Microsoft Azure"

def homepage():
    return "https://docs.microsoft.com/cli/azure/"

def repository():
    return "https://github.com/Azure/azure-cli"

def license():
    return "MIT"

def ecosystem():
    return "cloud"

def aliases():
    return ["azure-cli"]

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    {
        "name":        "az",
        "executable":  "az",
        "description": "Azure Command Line Interface",
        "aliases":     ["azcli", "azure-cli"],
        "priority":    100,
    },
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = {
    "http": ["api.github.com", "github.com", "aka.ms"],
    "fs":   [],
    "exec": [],
}

# ---------------------------------------------------------------------------
# fetch_versions — Azure/azure-cli GitHub releases
# ---------------------------------------------------------------------------

fetch_versions = make_fetch_versions("Azure", "azure-cli")

# ---------------------------------------------------------------------------
# download_url — Linux only (Windows/macOS use system install)
# ---------------------------------------------------------------------------

def download_url(ctx, version):
    """Build Azure CLI download URL.

    Linux: official tar.gz from GitHub releases
    Windows/macOS: use system_install (MSI not supported by vx-installer)
    """
    os   = ctx["platform"]["os"]
    arch = ctx["platform"]["arch"]

    if os == "linux":
        # Azure CLI Linux release: azure-cli-{version}-1.el9.x86_64.rpm or tar.gz
        # Use the GitHub release tar.gz
        asset = "azure-cli-{}.tar.gz".format(version)
        return "https://github.com/Azure/azure-cli/releases/download/{}/{}".format(
            version, asset
        )

    # Windows/macOS: no portable archive, use system_install
    return None

# ---------------------------------------------------------------------------
# install_layout
# ---------------------------------------------------------------------------

def install_layout(ctx, version):
    return {
        "type":             "archive",
        "strip_prefix":     "",
        "executable_paths": ["bin/az"],
    }

# ---------------------------------------------------------------------------
# environment
# ---------------------------------------------------------------------------

def environment(ctx, version, install_dir):
    return {"PATH": install_dir + "/bin"}

# ---------------------------------------------------------------------------
# system_install — preferred on Windows and macOS
# ---------------------------------------------------------------------------

def system_install(ctx):
    os = ctx["platform"]["os"]
    if os == "windows":
        return {
            "strategies": [
                {"manager": "winget", "package": "Microsoft.AzureCLI", "priority": 100},
                {"manager": "choco",  "package": "azure-cli",           "priority": 80},
            ],
        }
    elif os == "macos":
        return {
            "strategies": [
                {"manager": "brew", "package": "azure-cli", "priority": 90},
            ],
        }
    elif os == "linux":
        return {
            "strategies": [
                {"manager": "brew", "package": "azure-cli", "priority": 70},
            ],
        }
    return {}

# ---------------------------------------------------------------------------
# deps
# ---------------------------------------------------------------------------

def deps(ctx, version):
    return []
