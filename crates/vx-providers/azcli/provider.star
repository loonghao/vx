# provider.star - Azure CLI provider
#
# Version source: https://github.com/Azure/azure-cli/releases
#
# Azure CLI uses platform-specific installers (.msi on Windows).
# vx prefers system package managers; Linux supports direct tar.gz download.
#
# Inheritance pattern: Level 2 (custom download_url + system_install)

load("@vx//stdlib:github.star", "make_fetch_versions")
load("@vx//stdlib:install.star", "set_permissions")
load("@vx//stdlib:env.star", "env_prepend")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "azcli"
description = "Azure CLI - Command-line interface for Microsoft Azure"
homepage    = "https://docs.microsoft.com/cli/azure/"
repository  = "https://github.com/Azure/azure-cli"
license     = "MIT"
ecosystem   = "cloud"
aliases     = ["azure-cli"]

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
        "test_commands": [
            {"command": "{executable} --version", "name": "version_check", "expected_output": "azure-cli"},
        ],
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
    os   = ctx.platform.os
    arch = ctx.platform.arch

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

def install_layout(_ctx, _version):
    return {
        "type":             "archive",
        "strip_prefix":     "",
        "executable_paths": ["bin/az"],
    }

# ---------------------------------------------------------------------------
# environment
# ---------------------------------------------------------------------------

def environment(ctx, _version):
    return [env_prepend("PATH", ctx.install_dir + "/bin")]

# ---------------------------------------------------------------------------
# system_install — preferred on Windows and macOS
# ---------------------------------------------------------------------------

def system_install(ctx):
    os = ctx.platform.os
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
# post_extract — set executable permissions on Linux
#
# Azure CLI Linux tar.gz extracts to bin/az.
# The binary needs +x permissions on Linux.
# ---------------------------------------------------------------------------

def post_extract(ctx, version, install_dir):
    """Set executable permissions on the Azure CLI binary after extraction.

    The Azure CLI Linux tar.gz places the main executable at bin/az.
    On Linux we need to ensure it has execute permissions.

    Args:
        ctx:         Provider context
        version:     Installed version string
        install_dir: Path to the installation directory

    Returns:
        List of post-extract actions
    """
    os = ctx.platform.os
    if os == "linux" or os == "macos":
        return [
            set_permissions("bin/az", "755"),
        ]
    return []

# ---------------------------------------------------------------------------
# deps
# ---------------------------------------------------------------------------

def deps(_ctx, _version):
    return []


# ---------------------------------------------------------------------------
# Path queries (RFC 0037)
# ---------------------------------------------------------------------------

def store_root(ctx):
    """Return the vx store root directory for azcli."""
    return ctx.vx_home + "/store/azcli"

def get_execute_path(ctx, version):
    """Return the executable path for the given version."""
    os = ctx.platform.os
    if os == "windows":
        return ctx.install_dir + "/az.exe"
    else:
        return ctx.install_dir + "/az"

def post_install(_ctx, _version):
    """Post-install hook (no-op for azcli)."""
    return None
