# provider.star - .NET SDK provider
#
# Version source: Microsoft dotnet-releases API
# Downloads from Microsoft CDN (dotnetcli.azureedge.net)
#
# Uses stdlib templates from @vx//stdlib:provider.star

load("@vx//stdlib:provider.star",
     "runtime_def", "fetch_versions_from_api", "dep_def",
     "system_permissions")
load("@vx//stdlib:env.star", "env_set", "env_prepend")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "dotnet"
description = ".NET SDK - Free, cross-platform, open-source developer platform for C#, F#, and VB.NET"
homepage    = "https://dotnet.microsoft.com"
repository  = "https://github.com/dotnet/sdk"
license     = "MIT"
ecosystem   = "dotnet"
aliases     = ["dotnet-sdk"]

# Supported package prefixes for ecosystem:package syntax (RFC 0027)
# Enables `vx dotnet-tool:<package>` for .NET global tool installation
package_prefixes = ["dotnet-tool", "dotnet"]

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    runtime_def("dotnet",
        aliases = ["dotnet-sdk"],
        test_commands = [
            {"command": "{executable} --version", "name": "version_check",
             "expected_output": "^\\d+\\.\\d+"},
            {"command": "{executable} --info", "name": "info_check",
             "expected_output": "Runtime Environment"},
        ],
    ),
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = system_permissions(
    extra_hosts = ["dotnetcli.blob.core.windows.net", "dotnetcli.azureedge.net",
                   "builds.dotnet.microsoft.com"],
)

# ---------------------------------------------------------------------------
# fetch_versions — Microsoft dotnet-releases API
# ---------------------------------------------------------------------------

fetch_versions = fetch_versions_from_api(
    "https://dotnetcli.blob.core.windows.net/dotnet/release-metadata/releases-index.json",
    "dotnet_releases",
)

# ---------------------------------------------------------------------------
# Platform helpers — .NET Runtime Identifiers (RIDs)
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

def install_layout(ctx, _version):
    exe = "dotnet.exe" if ctx.platform.os == "windows" else "dotnet"
    return {
        "type":             "archive",
        "strip_prefix":     "",
        "executable_paths": [exe],
    }

# ---------------------------------------------------------------------------
# Path queries + environment
# ---------------------------------------------------------------------------

def store_root(ctx):
    return ctx.vx_home + "/store/dotnet"

def get_execute_path(ctx, _version):
    exe = "dotnet.exe" if ctx.platform.os == "windows" else "dotnet"
    return ctx.install_dir + "/" + exe

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
