# provider.star - choco provider
#
# Chocolatey - The package manager for Windows
# Inheritance pattern: Level 2 (custom download_url + script install)
#   - fetch_versions: inherited from github.star
#   - download_url:   None (installed via PowerShell script to vx store)
#
# Chocolatey is Windows-only.
# We install it to the vx store path (~/.vx/store/choco/<version>/) using
# the official install script with a custom CHOCOLATEY_INSTALL env var,
# which Chocolatey respects as the installation directory.

load("@vx//stdlib:github.star", "make_fetch_versions")
load("@vx//stdlib:env.star", "env_set", "env_prepend")
load("@vx//stdlib:test.star", "cmd", "check_path", "check_env")
load("@vx//stdlib:provider.star", "system_permissions", "irm_iex_install")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "choco"
description = "The package manager for Windows"
homepage    = "https://chocolatey.org"
repository  = "https://github.com/chocolatey/choco"
license     = "Apache-2.0"
ecosystem   = "system"

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
        "test_commands": [
            cmd("{executable} --version",
                name="version_check",
                expected_output="^\\d+\\.\\d+"),
            check_path("{install_dir}/choco.exe",
                       name="binary_exists"),
            check_env("ChocolateyInstall",
                      name="install_dir_set",
                      expected_output=".+"),
        ],
    },
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = system_permissions(
    exec_cmds   = ["powershell", "pwsh"],
    extra_hosts = ["api.github.com", "github.com", "community.chocolatey.org"],
)

# ---------------------------------------------------------------------------
# fetch_versions — inherited from GitHub releases
# ---------------------------------------------------------------------------

fetch_versions = make_fetch_versions("chocolatey", "choco")

# ---------------------------------------------------------------------------
# download_url — None (installed via PowerShell script)
# ---------------------------------------------------------------------------

def download_url(_ctx, _version):
    """Chocolatey is installed via PowerShell script, not direct download."""
    return None

# ---------------------------------------------------------------------------
# script_install — PowerShell iex(irm) with custom install dir
# ---------------------------------------------------------------------------

# Chocolatey respects CHOCOLATEY_INSTALL as the installation directory.
# We set it to the vx store path so choco is fully managed by vx.
script_install = irm_iex_install(
    "https://community.chocolatey.org/install.ps1",
    env_vars = {"ChocolateyInstall": "{install_dir}"},
)

# ---------------------------------------------------------------------------
# environment
# ---------------------------------------------------------------------------

def environment(ctx, version):
    install_dir = ctx.vx_home + "/store/choco/" + version
    return [
        env_set("ChocolateyInstall", install_dir),
        env_prepend("PATH", install_dir + "/bin"),
    ]

# ---------------------------------------------------------------------------
# Path queries (RFC 0037)
# ---------------------------------------------------------------------------

def store_root(ctx):
    """Return the vx store root directory for choco."""
    return ctx.vx_home + "/store/choco"

def get_execute_path(ctx, version):
    """Return the executable path for the given version."""
    os = ctx.platform.os
    if os == "windows":
        return ctx.vx_home + "/store/choco/" + version + "/choco.exe"
    else:
        return ctx.vx_home + "/store/choco/" + version + "/choco"

def post_install(_ctx, _version):
    """Post-install hook (no-op for choco)."""
    return None
