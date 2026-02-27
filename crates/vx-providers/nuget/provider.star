# provider.star - nuget provider
#
# NuGet: The package manager for .NET
# Inheritance pattern: Level 3 (custom fetch + download, Windows-only binary)
#   - fetch_versions: custom (GitHub releases)
#   - download_url:   custom (direct binary from nuget.org, Windows-only)
#
# nuget.exe is Windows-only; on macOS/Linux use `dotnet nuget`
# Download: https://dist.nuget.org/win-x86-commandline/v{version}/nuget.exe

load("@vx//stdlib:provider.star",
     "runtime_def", "path_fns", "path_env_fns",
     "github_permissions")
load("@vx//stdlib:github.star", "make_fetch_versions")

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
        aliases         = ["nuget-cli"],
        description     = "NuGet command-line tool",
        version_cmd     = "{executable} help",
        version_pattern = "NuGet"),
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = github_permissions(extra_hosts = ["dist.nuget.org"])

# ---------------------------------------------------------------------------
# fetch_versions — GitHub releases
# ---------------------------------------------------------------------------

fetch_versions = make_fetch_versions("NuGet", "NuGet.Client")

# ---------------------------------------------------------------------------
# download_url — nuget.org CDN, Windows-only
#
# nuget.exe is Windows-only. On macOS/Linux, use `dotnet nuget` instead.
# URL: https://dist.nuget.org/win-x86-commandline/v{version}/nuget.exe
# ---------------------------------------------------------------------------

def download_url(ctx, version):
    if ctx.platform.os != "windows":
        return None
    return "https://dist.nuget.org/win-x86-commandline/v{}/nuget.exe".format(version)

# ---------------------------------------------------------------------------
# install_layout — single binary
# ---------------------------------------------------------------------------

def install_layout(_ctx, _version):
    return {
        "type":        "binary",
        "target_name": "nuget.exe",
        "target_dir":  "bin",
    }

# ---------------------------------------------------------------------------
# Path queries + environment (RFC-0037)
# ---------------------------------------------------------------------------

paths = path_fns("nuget", executable = "nuget")
env_fns = path_env_fns()

store_root       = paths["store_root"]
get_execute_path = paths["get_execute_path"]
environment      = env_fns["environment"]
post_install     = env_fns["post_install"]

def deps(_ctx, _version):
    return []
