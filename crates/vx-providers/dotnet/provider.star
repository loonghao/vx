# provider.star - .NET SDK provider
#
# .NET SDK downloads from Microsoft CDN
# RID: win-x64, osx-x64, linux-x64, etc.
#
# Uses stdlib templates from @vx//stdlib:provider.star

load("@vx//stdlib:provider.star",
     "runtime_def", "github_permissions", "dep_def",
     "archive_layout", "path_fns")
load("@vx//stdlib:env.star",    "env_prepend", "env_set")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "dotnet"
description = ".NET SDK - Developer platform for building apps"
homepage    = "https://dotnet.microsoft.com"
repository  = "https://github.com/dotnet/sdk"
license     = "MIT"
ecosystem   = "dotnet"

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    runtime_def("dotnet",
        aliases         = ["dotnet-sdk", "dotnet-cli"],
        version_pattern = "\\d+\\.\\d+\\.\\d+",
        test_commands = [
            {"command": "{executable} --version", "name": "version_check"},
        ],
    ),
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = github_permissions(extra_hosts = ["dotnetcli.azureedge.net"])

# ---------------------------------------------------------------------------
# fetch_versions — from dotnetcli.azureedge.net
# Note: For now we use an empty list; users should specify version explicitly
# ---------------------------------------------------------------------------

def fetch_versions(_ctx):
    return []

# ---------------------------------------------------------------------------
# Platform helpers
# .NET uses Runtime Identifiers (RIDs)
# ---------------------------------------------------------------------------

_DOTNET_RIDS = {
    "windows/x64":   "win-x64",
    "windows/x86":   "win-x86",
    "windows/arm64": "win-arm64",
    "macos/x64":     "osx-x64",
    "macos/arm64":   "osx-arm64",
    "linux/x64":     "linux-x64",
    "linux/arm64":   "linux-arm64",
    "linux/armv7":   "linux-arm",
}

def _dotnet_rid(ctx):
    return _DOTNET_RIDS.get("{}/{}".format(ctx.platform.os, ctx.platform.arch))

# ---------------------------------------------------------------------------
# download_url — Microsoft CDN
# ---------------------------------------------------------------------------

def download_url(ctx, version):
    rid = _dotnet_rid(ctx)
    if not rid:
        return None
    ext = "zip" if ctx.platform.os == "windows" else "tar.gz"
    filename = "dotnet-sdk-{}-{}.{}".format(version, rid, ext)
    return "https://dotnetcli.azureedge.net/dotnet/Sdk/{}/{}".format(version, filename)

# ---------------------------------------------------------------------------
# install_layout — flat layout
# ---------------------------------------------------------------------------

install_layout   = archive_layout("dotnet")
_paths           = path_fns("dotnet")
store_root       = _paths["store_root"]
get_execute_path = _paths["get_execute_path"]

def post_install(_ctx, _version):
    return None

def environment(ctx, _version):
    return [
        env_set("DOTNET_ROOT", ctx.install_dir),
        env_prepend("PATH", ctx.install_dir),
    ]

# ---------------------------------------------------------------------------
# deps
# ---------------------------------------------------------------------------

def deps(_ctx, _version):
    return [
        dep_def("git",   optional = True,
                reason = "Git is recommended for NuGet package sources"),
        dep_def("nuget", optional = True,
                reason = "NuGet CLI for advanced package management"),
    ]
