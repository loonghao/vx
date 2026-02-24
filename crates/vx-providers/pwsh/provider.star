# provider.star - pwsh (PowerShell) provider
#
# PowerShell: Cross-platform command-line shell and scripting language
# Inheritance pattern: Level 2 (custom download_url for PowerShell's naming)
#   - fetch_versions: inherited from github.star
#   - download_url:   custom (PowerShell-{version}-{platform}.{ext})
#
# pwsh releases: https://github.com/PowerShell/PowerShell/releases
# Asset format: PowerShell-{version}-{os}-{arch}.{ext}

load("@vx//stdlib:github.star", "make_fetch_versions", "github_asset_url")

load("@vx//stdlib:env.star", "env_prepend")
# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "pwsh"
description = "Cross-platform command-line shell and scripting language"
homepage    = "https://docs.microsoft.com/en-us/powershell/"
repository  = "https://github.com/PowerShell/PowerShell"
license     = "MIT"
ecosystem   = "system"

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    {
        "name":        "pwsh",
        "executable":  "pwsh",
        "description": "PowerShell 7+ (cross-platform)",
        "aliases":     ["powershell", "ps"],
        "priority":    100,
        "test_commands": [
            {"command": "{executable} -Command \"$PSVersionTable.PSVersion\"", "name": "version_check", "expected_output": "\\d+\\.\\d+"},
        ],
    },
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = {
    "http": ["api.github.com", "github.com"],
    "fs":   [],
    "exec": [],
}

# ---------------------------------------------------------------------------
# fetch_versions — inherited
# ---------------------------------------------------------------------------

fetch_versions = make_fetch_versions("PowerShell", "PowerShell")

# ---------------------------------------------------------------------------
# download_url — custom
#
# PowerShell asset naming: PowerShell-{version}-{os}-{arch}.{ext}
#   PowerShell-7.5.0-win-x64.zip
#   PowerShell-7.5.0-osx-x64.tar.gz / PowerShell-7.5.0-osx-arm64.tar.gz
#   PowerShell-7.5.0-linux-x64.tar.gz / PowerShell-7.5.0-linux-arm64.tar.gz
# ---------------------------------------------------------------------------

def _pwsh_platform(ctx):
    """Map platform to PowerShell's naming convention."""
    os   = ctx.platform.os
    arch = ctx.platform.arch

    platform_map = {
        "windows/x64":   ("win",   "x64",   "zip"),
        "windows/arm64": ("win",   "arm64", "zip"),
        "macos/x64":     ("osx",   "x64",   "tar.gz"),
        "macos/arm64":   ("osx",   "arm64", "tar.gz"),
        "linux/x64":     ("linux", "x64",   "tar.gz"),
        "linux/arm64":   ("linux", "arm64", "tar.gz"),
    }
    return platform_map.get("{}/{}".format(os, arch))

def download_url(ctx, version):
    """Build the PowerShell download URL.

    Args:
        ctx:     Provider context
        version: Version string, e.g. "7.5.0"

    Returns:
        Download URL string, or None if platform is unsupported
    """
    platform = _pwsh_platform(ctx)
    if not platform:
        return None

    ps_os, ps_arch, ext = platform
    asset = "PowerShell-{}-{}-{}.{}".format(version, ps_os, ps_arch, ext)
    tag = "v{}".format(version)
    return github_asset_url("PowerShell", "PowerShell", tag, asset)

# ---------------------------------------------------------------------------
# install_layout
# ---------------------------------------------------------------------------

def install_layout(ctx, _version):
    os = ctx.platform.os
    exe = "pwsh.exe" if os == "windows" else "pwsh"
    return {
        "type":             "archive",
        "strip_prefix":     "",
        "executable_paths": [exe, "pwsh"],
    }

# ---------------------------------------------------------------------------
# Path queries (RFC-0037)
# ---------------------------------------------------------------------------

def store_root(ctx):
    """Return the vx store root directory for pwsh."""
    return ctx.vx_home + "/store/pwsh"

def get_execute_path(ctx, version):
    """Return the executable path for the given version."""
    os = ctx.platform.os
    exe = "pwsh.exe" if os == "windows" else "pwsh"
    return ctx.install_dir + "/" + exe

def post_install(_ctx, _version):
    """No post-install steps needed for pwsh."""
    return None

# ---------------------------------------------------------------------------
# environment
# ---------------------------------------------------------------------------

def environment(ctx, _version):
    return [env_prepend("PATH", ctx.install_dir)]
