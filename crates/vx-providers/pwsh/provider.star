# provider.star - pwsh (PowerShell) provider
#
# PowerShell archives are used on macOS/Linux.
# Windows prefers the system-installed PowerShell package.

load("@vx//stdlib:provider.star",
     "runtime_def", "github_permissions",
     "system_install_strategies", "winget_install", "choco_install",
     "brew_install", "apt_install")
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
    runtime_def("pwsh",
        aliases = ["powershell", "ps"],
        system_paths = [
            "C:/Program Files/PowerShell/*/pwsh.exe",
            "C:/Program Files (x86)/PowerShell/*/pwsh.exe",
            "/usr/local/bin/pwsh",
            "/usr/bin/pwsh",
        ],
        test_commands = [
            {"command": "{executable} -Command \"$PSVersionTable.PSVersion\"",
             "name": "version_check", "expected_output": "\\d+\\.\\d+"},
        ],
    ),
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = github_permissions(exec_cmds = ["winget", "choco"])

# ---------------------------------------------------------------------------
# fetch_versions
# ---------------------------------------------------------------------------

fetch_versions = make_fetch_versions("PowerShell", "PowerShell")

# ---------------------------------------------------------------------------
# Platform helpers
# ---------------------------------------------------------------------------

_PWSH_PLATFORMS = {
    "windows/x64":   ("win",   "x64",   "zip"),
    "windows/arm64": ("win",   "arm64", "zip"),
    "macos/x64":     ("osx",   "x64",   "tar.gz"),
    "macos/arm64":   ("osx",   "arm64", "tar.gz"),
    "linux/x64":     ("linux", "x64",   "tar.gz"),
    "linux/arm64":   ("linux", "arm64", "tar.gz"),
}


def download_url(ctx, version):
    if ctx.platform.os == "windows":
        return None
    platform = _PWSH_PLATFORMS.get("{}/{}".format(ctx.platform.os, ctx.platform.arch))
    if not platform:
        return None
    ps_os, ps_arch, ext = platform[0], platform[1], platform[2]
    asset = "PowerShell-{}-{}-{}.{}".format(version, ps_os, ps_arch, ext)
    return github_asset_url("PowerShell", "PowerShell", "v" + version, asset)

# ---------------------------------------------------------------------------
# install_layout
# ---------------------------------------------------------------------------

def install_layout(ctx, _version):
    exe = "pwsh.exe" if ctx.platform.os == "windows" else "pwsh"
    return {
        "type":             "archive",
        "strip_prefix":     "",
        "executable_paths": [exe, "pwsh"],
    }

# ---------------------------------------------------------------------------
# system_install — Windows package managers
# ---------------------------------------------------------------------------

system_install = system_install_strategies([
    winget_install("Microsoft.PowerShell", priority = 90),
    choco_install("powershell-core", priority = 80),
    brew_install("powershell", priority = 70),
    apt_install("powershell", priority = 60),
])

# ---------------------------------------------------------------------------
# Path queries + environment
# ---------------------------------------------------------------------------

def store_root(ctx):
    return ctx.vx_home + "/store/pwsh"


def get_execute_path(ctx, _version):
    exe = "pwsh.exe" if ctx.platform.os == "windows" else "pwsh"
    return ctx.install_dir + "/" + exe


def post_install(_ctx, _version):
    return None


def environment(ctx, _version):
    return [env_prepend("PATH", ctx.install_dir)]


def deps(_ctx, _version):
    return []
