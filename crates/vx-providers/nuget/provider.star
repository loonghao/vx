# provider.star - nuget provider
#
# NuGet: The package manager for .NET
# nuget.exe is Windows-only; on macOS/Linux use `dotnet nuget` instead.
# Download: https://dist.nuget.org/win-x86-commandline/v{version}/nuget.exe
#
# Uses runtime_def + github_permissions from @vx//stdlib:provider.star

load("@vx//stdlib:provider.star",
     "runtime_def", "github_permissions",
     "system_install_strategies", "winget_install", "choco_install")
load("@vx//stdlib:env.star", "env_prepend")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "nuget"
description = "NuGet - The package manager for .NET"
homepage    = "https://www.nuget.org/"
repository  = "https://github.com/NuGet/NuGet.Client"
license     = "Apache-2.0"
ecosystem   = "dotnet"

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    runtime_def("nuget",
        aliases             = ["nuget-cli"],
        description         = "NuGet command-line tool",
        platform_constraint = {"os": ["windows"]},
        system_paths        = [
            "C:/Program Files/NuGet/nuget.exe",
            "C:/ProgramData/chocolatey/bin/nuget.exe",
        ],
        version_cmd         = "{executable} help",
        version_pattern     = "NuGet",
    ),
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = github_permissions(
    extra_hosts = ["dist.nuget.org"],
    exec_cmds   = ["winget", "choco"],
)

# ---------------------------------------------------------------------------
# fetch_versions — dist.nuget.org has a stable latest channel
# ---------------------------------------------------------------------------

def fetch_versions(_ctx):
    return [{"version": "latest", "lts": True, "prerelease": False}]

# ---------------------------------------------------------------------------
# download_url — nuget.org CDN, Windows-only
# ---------------------------------------------------------------------------

def download_url(ctx, version):
    if ctx.platform.os != "windows":
        return None
    if version == "latest":
        return "https://dist.nuget.org/win-x86-commandline/latest/nuget.exe"
    return "https://dist.nuget.org/win-x86-commandline/v{}/nuget.exe".format(version)

# ---------------------------------------------------------------------------
# install_layout — single binary
# ---------------------------------------------------------------------------

def install_layout(_ctx, _version):
    return {
        "type":             "binary",
        "target_name":      "nuget.exe",
        "target_dir":       "bin",
        "executable_paths": ["bin/nuget.exe", "nuget.exe", "nuget"],
    }

# ---------------------------------------------------------------------------
# system_install — Windows package managers
# ---------------------------------------------------------------------------

system_install = system_install_strategies([
    winget_install("Microsoft.NuGet", priority = 90),
    choco_install("nuget.commandline", priority = 80),
])

# ---------------------------------------------------------------------------
# Path queries + environment
# ---------------------------------------------------------------------------

def store_root(ctx):
    return ctx.vx_home + "/store/nuget"


def get_execute_path(ctx, _version):
    return ctx.install_dir + "/bin/nuget.exe"


def environment(ctx, _version):
    return [env_prepend("PATH", ctx.install_dir + "/bin")]


def post_install(_ctx, _version):
    return None


def deps(_ctx, _version):
    return []
